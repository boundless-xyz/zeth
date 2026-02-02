// Copyright 2026 RISC Zero, Inc.
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

use super::types::{EventKind, TraceFdEvent, TraceId};
use alloy_primitives::bytes::{Buf, BytesMut};
use std::{fmt::Display, io, vec::Vec};

/// A frame in the call stack maintained by the trace processor.
#[derive(Debug, Clone)]
struct StackFrame {
    id: TraceId<'static>,
    start_cycles: u64,
    start_gas: u64,
}

/// Processes trace events and maintains a call stack.
///
/// Converts raw enter/exit/total events into structured [`TraceEvent`]s
/// with computed depths and durations.
#[derive(Debug, Clone)]
struct TraceProcessor<F = fn(TraceEvent)> {
    stack: Vec<StackFrame>,
    callback: F,
}

/// A processed trace event with call depth and cycle count.
#[derive(Clone, Debug)]
pub struct TraceEvent {
    /// The identifier for this traced section
    pub id: TraceId<'static>,
    pub depth: usize,
    pub cycles: u64,
    pub gas: u64,
}

impl Display for TraceEvent {
    /// Default callback that prints trace events with indentation.
    ///
    /// Output format:
    /// ```text
    /// initialization: 1234
    ///     [ OP] 0x01: 56
    ///     [PRE] 0x0000...0001: 789
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let indent = "  ".repeat(self.depth);
        writeln!(f, "{}{}: {} cycles", indent, self.id, self.cycles)
    }
}

impl<F: FnMut(TraceEvent)> TraceProcessor<F> {
    /// Creates a new processor with a custom callback.
    ///
    /// The callback is invoked for each completed trace event.
    fn new(callback: F) -> Self {
        Self { stack: Vec::new(), callback }
    }

    fn process(&mut self, event: TraceFdEvent) {
        match event.kind {
            EventKind::Complete => {
                let id = event.id.into_owned();
                let depth = self.stack.len();
                (self.callback)(TraceEvent { id, depth, cycles: event.cycles, gas: event.gas });
            }
            EventKind::Enter => {
                let id = event.id.into_owned();
                self.stack.push(StackFrame {
                    id,
                    start_cycles: event.cycles,
                    start_gas: event.gas,
                });
            }
            EventKind::Exit => {
                if let Some(frame) = self.stack.pop() {
                    let id = event.id.into_owned();

                    // Safety Check: Verify we are popping what we expect
                    if frame.id == id {
                        let cycles = event.cycles.saturating_sub(frame.start_cycles);
                        let gas = event.gas.saturating_sub(frame.start_gas);
                        (self.callback)(TraceEvent { id, depth: self.stack.len(), cycles, gas });
                    } else {
                        tracing::warn!("Trace Mismatch: Entered {:?} but exited {id:?}", frame.id);
                    }
                } else {
                    tracing::warn!("Stack Underflow: Exited {:?} without entering it", event.id);
                }
            }
        }
    }
}

/// Host-side collector that processes the trace stream.
///
/// Implements [`io::Write`] so it can be passed to RiscZero's `write_fd`.
/// Deserializes COBS-framed events and invokes the processor callback.
///
/// # Example
///
/// ```rust,no_run
/// # use zeth_core::cycle_tracker::{CYCLE_TRACKER_FD, TraceCollector};
/// # use risc0_zkvm::{ExecutorEnv, default_executor};
/// # const GUEST_ELF: &[u8] = &[];
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut collector = TraceCollector::new(|event| println!("{event}"));
///
/// let env = ExecutorEnv::builder().write_fd(CYCLE_TRACKER_FD, collector).build()?;
///
/// let exec = default_executor();
/// exec.execute(env, GUEST_ELF)?;
/// # Ok(()) }
/// ```
#[derive(Debug, Clone)]
pub struct TraceCollector<F = fn(TraceEvent)> {
    buffer: BytesMut,
    processor: TraceProcessor<F>,
}

impl<F: FnMut(TraceEvent)> TraceCollector<F> {
    /// Creates a new collector with a custom processor.
    ///
    /// Useful for collecting statistics or custom output formatting.
    ///
    /// # Example
    /// ```rust
    /// # use zeth_core::cycle_tracker::TraceCollector;
    /// let collector = TraceCollector::new(|event| {
    ///     if event.cycles > 1000 {
    ///         println!("Expensive: {} took {} cycles", event.id, event.cycles);
    ///     }
    /// });
    /// ```
    pub fn new(callback: F) -> Self {
        Self { buffer: BytesMut::default(), processor: TraceProcessor::new(callback) }
    }
}

impl<F: FnMut(TraceEvent)> io::Write for TraceCollector<F> {
    /// Buffers partial frames and deserializes complete ones (delimited by zero bytes).
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.buffer.extend_from_slice(buf);

        while let Some(zero_pos) = self.buffer.iter().position(|&b| b == 0) {
            let frame_len = zero_pos + 1;
            let frame = &mut self.buffer[..frame_len];

            match postcard::from_bytes_cobs::<TraceFdEvent>(frame) {
                Ok(event) => self.processor.process(event),
                Err(e) => return Err(io::Error::other(e)),
            }

            self.buffer.advance(frame_len);
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
