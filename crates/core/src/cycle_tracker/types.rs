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

use alloy_primitives::Address;
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, fmt::Display};

/// The file descriptor used for trace communication.
///
/// This must be configured on the Host via [`risc0_zkvm::ExecutorEnvBuilder::write_fd`] and is used
/// by the Guest to stream trace events.
pub const CYCLE_TRACKER_FD: u32 = 0x10;

/// Identifier for a traced code section.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TraceId<'a> {
    // Custom name
    #[serde(borrow)]
    Custom(Cow<'a, str>),
    // Opcode execution
    Opcode(u8),
    // Precompile invocation
    Precompile(Address),
}

impl Display for TraceId<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TraceId::Custom(s) => write!(f, "[ FN] {s}"),
            TraceId::Opcode(op) => write!(f, "[ OP] 0x{op:02x}"),
            TraceId::Precompile(addr) => write!(f, "[PRE] {addr}"),
        }
    }
}

impl<'a> TraceId<'a> {
    /// Converts a borrowed `TraceId` into an owned one.
    pub fn into_owned(self) -> TraceId<'static> {
        match self {
            TraceId::Custom(s) => TraceId::Custom(Cow::Owned(s.into_owned())),
            TraceId::Opcode(op) => TraceId::Opcode(op),
            TraceId::Precompile(addr) => TraceId::Precompile(addr),
        }
    }
}

/// Helper trait for ergonomic trace ID construction.
pub trait IntoTraceId<'a> {
    fn into_trace_id(self) -> TraceId<'a>;
}

impl<'a> IntoTraceId<'a> for TraceId<'a> {
    fn into_trace_id(self) -> TraceId<'a> {
        self
    }
}

impl<'a> IntoTraceId<'a> for &TraceId<'a> {
    fn into_trace_id(self) -> TraceId<'a> {
        self.clone()
    }
}

impl<'a> IntoTraceId<'a> for &'a str {
    fn into_trace_id(self) -> TraceId<'a> {
        TraceId::Custom(Cow::Borrowed(self))
    }
}

impl<'a> IntoTraceId<'a> for String {
    fn into_trace_id(self) -> TraceId<'a> {
        TraceId::Custom(Cow::Owned(self))
    }
}

impl<'a> IntoTraceId<'a> for u8 {
    fn into_trace_id(self) -> TraceId<'a> {
        TraceId::Opcode(self)
    }
}

impl<'a> IntoTraceId<'a> for Address {
    fn into_trace_id(self) -> TraceId<'a> {
        TraceId::Precompile(self)
    }
}

/// Internal event type for wire protocol.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum EventKind {
    /// Marks the start of a traced scope (e.g., entering an opcode or function).
    Enter,
    /// Marks the end of a traced scope that was previously opened with [`EventKind::Enter`].
    Exit,
    /// Represents a complete, atomic execution of a scope (Enter + Exit).
    Complete,
}

/// The packet sent over the trace file descriptor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct TraceFdEvent<'a> {
    pub kind: EventKind,
    #[serde(borrow)]
    pub id: TraceId<'a>,
    pub cycles: u64,
    pub gas: u64,
}
