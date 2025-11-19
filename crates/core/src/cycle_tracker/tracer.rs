// Copyright 2025 RISC Zero, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use super::types::{EventKind, IntoTraceId, TraceFdEvent, TraceId};
use serde::Serialize;
use std::cell::RefCell;

#[cfg(target_os = "zkvm")]
mod platform {
    use crate::cycle_tracker::types;
    use risc0_zkvm::guest::env::{FdWriter, Write};

    pub use risc0_zkvm::guest::env::cycle_count;

    #[inline(always)]
    pub fn write_slice(buf: &[u8]) {
        FdWriter::new(types::CYCLE_TRACKER_FD, |_| {}).write_slice(buf);
    }
}

#[cfg(not(target_os = "zkvm"))]
mod platform {
    /// Returns 0 to prevent panics when instrumented code runs on the host.
    pub fn cycle_count() -> u64 {
        0
    }
    /// No-op on the host; trace data is discarded.
    pub fn write_slice(_: &[u8]) {}
}

/// Guest-side tracer for recording cycle counts.
///
/// This struct is responsible for serializing trace events using [postcard] with COBS framing and
/// writing them to the trace file descriptor.
///
/// # Optimization Strategy
///
/// To minimize I/O overhead, the tracer employs a "write-combining" strategy:
/// If `exit` is called immediately after `enter` for the same ID (a leaf span), the tracer
/// calculates the diff and writes a single event.
///
/// # Performance Note
///
/// For hot loops (like EVM opcode execution), instantiate this struct directly and keep it alive to
/// avoid the overhead of Thread Local Storage (TLS) associated with the global [`enter`] and
/// [`exit`] functions.
#[derive(Clone, Debug)]
pub struct CycleTracer<'a> {
    // use a Box to keep the struct itself small (stack-friendly)
    buf: Box<[u8]>,
    last_enter: Option<(TraceId<'a>, u64, u64)>,
}

impl<'a> Default for CycleTracer<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Drop for CycleTracer<'a> {
    fn drop(&mut self) {
        // ensure any pending "enter" is flushed if it is dropped before the corresponding "exit"
        if let Some((id, cycles, gas)) = self.last_enter.take() {
            self.send(EventKind::Enter, id, cycles, gas)
        }
    }
}

impl<'a> CycleTracer<'a> {
    /// Creates a new tracer with a default 48-byte serialization buffer.
    pub fn new() -> Self {
        Self { buf: Box::new([0u8; 48]), last_enter: None }
    }

    /// Records the start of a traced section.
    ///
    /// If a previous [`CycleTracer::enter`] is pending, it's flushed before recording the new one.
    /// This batching behavior minimizes syscalls.
    ///
    /// # Example
    /// ```rust
    /// # let mut tracer = zeth_core::cycle_tracker::guest::CycleTracer::new();
    /// tracer.enter("function_name");
    /// // ... work ...
    /// tracer.exit("function_name");
    /// ```
    #[inline(always)]
    pub fn enter(&mut self, id: impl IntoTraceId<'a>) {
        self.enter_with_gas(id, 0)
    }

    /// Records the start of a traced section with an associated gas metric.
    ///
    /// It captures the current cycle count and the provided `gas` value (typically cumulative gas
    /// spent). The `gas` value is stored and used later in [`CycleTracer::exit_with_gas`] to
    /// calculate the exact amount of gas consumed during this span (`end_gas - start_gas`).
    #[inline]
    pub fn enter_with_gas(&mut self, id: impl IntoTraceId<'a>, gas: u64) {
        let id = id.into_trace_id();
        let cycles = platform::cycle_count();

        // If we have a pending enter that wasn't closed, flush it now.
        // This handles nested calls like: enter(A) -> enter(B) -> exit(B) -> exit(A)
        if let Some((enter_id, enter_cycles, enter_gas)) =
            self.last_enter.replace((id, cycles, gas))
        {
            self.send(EventKind::Enter, enter_id, enter_cycles, enter_gas);
        }
    }

    /// Records the end of a traced section.
    #[inline(always)]
    pub fn exit(&mut self, id: impl IntoTraceId<'a>) {
        self.exit_with_gas(id, 0)
    }

    /// Records the end of a traced section with an associated gas metric.
    ///
    /// It captures the current cycle count and provided `gas` value (typically cumulative gas
    /// spent).If the pending enter event matches this exit (same ID), the tracer emits a single
    /// event containing the net cycles and gas used.
    pub fn exit_with_gas(&mut self, id: impl IntoTraceId<'a>, gas: u64) {
        let cycles = platform::cycle_count();
        let id = id.into_trace_id();
        match self.last_enter.take() {
            None => self.send(EventKind::Exit, id, cycles, gas),
            Some((enter_id, enter_cycles, enter_gas)) => {
                if enter_id == id {
                    self.send(EventKind::Complete, id, cycles - enter_cycles, gas - enter_gas);
                } else {
                    self.send(EventKind::Enter, enter_id, enter_cycles, enter_gas);
                    self.send(EventKind::Exit, id, cycles, gas);
                }
            }
        }
    }

    /// Sends the corresponding event via the file descriptor.
    /// # Panics
    /// It panics if serialization fails (OOM or internal error).
    fn send(&mut self, kind: EventKind, id: TraceId, cycles: u64, gas: u64) {
        let event = TraceFdEvent { kind, id, cycles, gas };
        let encoded = self.serialize(&event).expect("should serialize");
        platform::write_slice(encoded);
    }

    fn serialize<T: Serialize + ?Sized>(&mut self, value: &T) -> postcard::Result<&mut [u8]> {
        match postcard::to_slice_cobs(value, &mut self.buf) {
            Ok(encoded) => {
                let len = encoded.len();
                Ok(&mut self.buf[..len])
            }
            Err(postcard::Error::SerializeBufferFull) => {
                // if buf is not sufficient, allocate a new vec and use that as the new buf
                let mut buf = postcard::to_allocvec_cobs(value)?;
                let len = buf.len();
                // we have allocated with capacity, so we might as well use everything
                buf.resize(buf.capacity(), 0);
                self.buf = buf.into_boxed_slice();
                Ok(&mut self.buf[..len])
            }
            Err(err) => Err(err),
        }
    }
}

thread_local! {
    static GLOBAL_TRACER: RefCell<CycleTracer<'static>> =  RefCell::new(CycleTracer::new()) ;
}

/// Records the start of a traced section using the thread-local global tracer.
///
/// Use this for high-level logic. For tight loops, use [`CycleTracer`] directly.
pub fn enter(id: impl IntoTraceId<'static>) {
    GLOBAL_TRACER.with_borrow_mut(move |t| t.enter(id))
}

/// Records the end of a traced section using the thread-local global tracer.
///
/// Use this for high-level logic. For tight loops, use [`CycleTracer`] directly.
pub fn exit(id: impl IntoTraceId<'static>) {
    GLOBAL_TRACER.with_borrow_mut(move |t| t.exit(id))
}

/// Creates a RAII guard for a traced section using the global tracer.
///
/// # Example
/// ```rust
/// use zeth_core::cycle_tracker::guest::span;
/// pub fn process() {
///     let _outer = span("process");
///     {
///         let _inner = span("inner");
///     }
/// }
/// ```
pub fn span(id: impl IntoTraceId<'static>) -> Span {
    let id = id.into_trace_id();
    enter(&id);
    Span { id }
}

/// RAII guard that calls [`exit`] when dropped.
#[derive(Debug)]
#[must_use]
pub struct Span {
    id: TraceId<'static>,
}

impl Drop for Span {
    #[inline(always)]
    fn drop(&mut self) {
        exit(&self.id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::{Address, Bytes};

    #[test]
    fn serialize() {
        let mut tracer = CycleTracer::new();

        // Small event (should fit in 48 bytes)
        let event = TraceFdEvent {
            kind: EventKind::Complete,
            id: TraceId::Precompile(Address::repeat_byte(0xff)),
            cycles: u64::MAX,
            gas: u64::MAX,
        };
        let scratch_len = tracer.buf.len();
        let encoded = tracer.serialize(&event).unwrap();
        println!("encoded: {}", Bytes::copy_from_slice(encoded));
        assert_eq!(event, postcard::from_bytes_cobs(encoded).unwrap());
        assert_eq!(scratch_len, tracer.buf.len()); // must fit in original buffer

        let event =
            TraceFdEvent { kind: EventKind::Enter, id: TraceId::Opcode(0), cycles: 0, gas: 0 };
        let encoded = tracer.serialize(&event).unwrap();
        assert_eq!(event, postcard::from_bytes_cobs(encoded).unwrap());

        // Large event (should trigger resize)
        let event = TraceFdEvent {
            kind: EventKind::Enter,
            id: TraceId::Custom("x".repeat(100).into()),
            cycles: 0,
            gas: 0,
        };
        let encoded = tracer.serialize(&event).unwrap();
        assert_eq!(event, postcard::from_bytes_cobs(encoded).unwrap());
        assert!(scratch_len < tracer.buf.len());

        // repeat to make sure the new buffer can be reused
        let encoded = tracer.serialize(&event).unwrap();
        assert_eq!(event, postcard::from_bytes_cobs(encoded).unwrap());
    }
}
