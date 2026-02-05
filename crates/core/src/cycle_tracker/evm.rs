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

use super::tracer::CycleTracer;
use reth_errors::BlockExecutionError;
use reth_evm::{
    ConfigureEvm, Database, EvmEnvFor, EvmFactory, ExecutionCtxFor, OnStateHook,
    block::{BlockExecutionResult, BlockExecutorFactory, BlockExecutorFor},
    execute::{BlockExecutor, Executor},
    revm::{
        Inspector,
        context::{ContextTr, JournalTr},
        database::{State, states::bundle_state::BundleRetention},
        interpreter::{
            CallInputs, CallOutcome, FrameInput, Interpreter, InterpreterAction,
            interpreter_types::Jumps,
        },
        primitives::hardfork::SpecId,
    },
};
use reth_evm_ethereum::EthEvmConfig;
use reth_primitives_traits::{
    BlockTy, HeaderTy, NodePrimitives, RecoveredBlock, SealedBlock, SealedHeader,
};
use std::fmt::Debug;

/// A wrapper around [`EthEvmConfig`] that enables cycle tracking during execution.
#[derive(Debug, Clone)]
pub struct CycleTrackerEvmConfig<ChainSpec, EvmF>(EthEvmConfig<ChainSpec, EvmF>);

impl<ChainSpec, EvmF> CycleTrackerEvmConfig<ChainSpec, EvmF> {
    /// Creates a new cycle tracking configuration wrapping the provided inner config.
    pub fn new(config: EthEvmConfig<ChainSpec, EvmF>) -> Self {
        Self(config)
    }
}

impl<ChainSpec, EvmF> ConfigureEvm for CycleTrackerEvmConfig<ChainSpec, EvmF>
where
    EthEvmConfig<ChainSpec, EvmF>: ConfigureEvm<
        BlockExecutorFactory: BlockExecutorFactory<EvmFactory: EvmFactory<Spec = SpecId>>,
    >,
    ChainSpec: Clone + Debug,
    EvmF: Clone + Debug,
{
    type Primitives = <EthEvmConfig<ChainSpec, EvmF> as ConfigureEvm>::Primitives;
    type Error = <EthEvmConfig<ChainSpec, EvmF> as ConfigureEvm>::Error;
    type NextBlockEnvCtx = <EthEvmConfig<ChainSpec, EvmF> as ConfigureEvm>::NextBlockEnvCtx;
    type BlockExecutorFactory =
        <EthEvmConfig<ChainSpec, EvmF> as ConfigureEvm>::BlockExecutorFactory;
    type BlockAssembler = <EthEvmConfig<ChainSpec, EvmF> as ConfigureEvm>::BlockAssembler;

    fn block_executor_factory(&self) -> &Self::BlockExecutorFactory {
        self.0.block_executor_factory()
    }

    fn block_assembler(&self) -> &Self::BlockAssembler {
        self.0.block_assembler()
    }

    fn evm_env(&self, header: &HeaderTy<Self::Primitives>) -> Result<EvmEnvFor<Self>, Self::Error> {
        self.0.evm_env(header)
    }

    fn next_evm_env(
        &self,
        parent: &HeaderTy<Self::Primitives>,
        attributes: &Self::NextBlockEnvCtx,
    ) -> Result<EvmEnvFor<Self>, Self::Error> {
        self.0.next_evm_env(parent, attributes)
    }

    fn context_for_block<'a>(
        &self,
        block: &'a SealedBlock<BlockTy<Self::Primitives>>,
    ) -> Result<ExecutionCtxFor<'a, Self>, Self::Error> {
        self.0.context_for_block(block)
    }

    fn context_for_next_block(
        &self,
        parent: &SealedHeader<HeaderTy<Self::Primitives>>,
        attributes: Self::NextBlockEnvCtx,
    ) -> Result<ExecutionCtxFor<'_, Self>, Self::Error> {
        self.0.context_for_next_block(parent, attributes)
    }

    fn executor<DB: Database>(
        &self,
        db: DB,
    ) -> impl Executor<DB, Primitives = Self::Primitives, Error = BlockExecutionError> {
        // override the default implementation to execute with cycle tracking
        CycleTrackerBlockExecutor::new(self, db)
    }
}

struct CycleTrackerBlockExecutor<F, DB> {
    factory: F,
    db: State<DB>,
}

impl<F, DB: Database> CycleTrackerBlockExecutor<F, DB> {
    pub(crate) fn new(factory: F, db: DB) -> Self {
        let db =
            State::builder().with_database(db).with_bundle_update().without_state_clear().build();
        Self { factory, db }
    }
}

impl<F, DB> CycleTrackerBlockExecutor<F, DB>
where
    F: ConfigureEvm<
        BlockExecutorFactory: BlockExecutorFactory<EvmFactory: EvmFactory<Spec = SpecId>>,
    >,
    DB: Database,
{
    /// Creates a strategy for execution of a given block with the inspector.
    fn executor_for_block<'a>(
        &'a mut self,
        block: &'a SealedBlock<<<F as ConfigureEvm>::Primitives as NodePrimitives>::Block>,
    ) -> Result<
        impl BlockExecutorFor<
            'a,
            <F as ConfigureEvm>::BlockExecutorFactory,
            DB,
            CycleTrackerInspector<'a>,
        >,
        <F as ConfigureEvm>::Error,
    > {
        let evm_env = self.factory.evm_env(block.header())?;

        let inspector = CycleTrackerInspector::default();
        let evm = self.factory.evm_with_env_and_inspector(&mut self.db, evm_env, inspector);

        let ctx = self.factory.context_for_block(block)?;
        Ok(self.factory.create_executor(evm, ctx))
    }
}

impl<F, DB> Executor<DB> for CycleTrackerBlockExecutor<F, DB>
where
    F: ConfigureEvm<
        BlockExecutorFactory: BlockExecutorFactory<EvmFactory: EvmFactory<Spec = SpecId>>,
    >,
    DB: Database,
{
    type Primitives = F::Primitives;
    type Error = BlockExecutionError;

    fn execute_one(
        &mut self,
        block: &RecoveredBlock<<Self::Primitives as NodePrimitives>::Block>,
    ) -> Result<BlockExecutionResult<<Self::Primitives as NodePrimitives>::Receipt>, Self::Error>
    {
        let result = self
            .executor_for_block(block)
            .map_err(BlockExecutionError::other)?
            .execute_block(block.transactions_recovered())?;

        self.db.merge_transitions(BundleRetention::Reverts);

        Ok(result)
    }

    fn execute_one_with_state_hook<H>(
        &mut self,
        _: &RecoveredBlock<<Self::Primitives as NodePrimitives>::Block>,
        _: H,
    ) -> Result<BlockExecutionResult<<Self::Primitives as NodePrimitives>::Receipt>, Self::Error>
    where
        H: OnStateHook + 'static,
    {
        unimplemented!()
    }

    fn into_state(self) -> State<DB> {
        self.db
    }

    fn size_hint(&self) -> usize {
        self.db.bundle_state.size_hint()
    }
}

#[derive(Clone, Debug, Default)]
struct CycleTrackerInspector<'a> {
    // last opcode executed
    last_opcode: Option<u8>,
    // since cycle tracking is performance-critical, we use a local tracer instead of a global one
    tracer: CycleTracer<'a>,
}

impl<CTX: ContextTr> Inspector<CTX> for CycleTrackerInspector<'_> {
    #[inline]
    fn initialize_interp(&mut self, _interp: &mut Interpreter, _context: &mut CTX) {
        // make sure the last opcode is reset, even if the interpreter is reused
        self.last_opcode = None;
    }

    #[inline]
    fn step(&mut self, interp: &mut Interpreter, _context: &mut CTX) {
        let opcode = interp.bytecode.opcode();

        // keep track of the last opcode executed
        self.last_opcode = Some(opcode);
        self.tracer.enter_with_gas(opcode, interp.gas.spent())
    }

    #[inline]
    fn step_end(&mut self, interp: &mut Interpreter, _context: &mut CTX) {
        if let Some(opcode) = self.last_opcode.take() {
            let mut gas = interp.gas.spent();

            // Calls and creations include the gas limit in the gas cost. We need to subtract this
            // amount because we want to track how much gas the opcode itself consumes.
            if let Some(InterpreterAction::NewFrame(frame)) = &interp.bytecode.action {
                let gas_limit = match frame {
                    FrameInput::Empty => 0,
                    FrameInput::Call(input) => input.gas_limit,
                    FrameInput::Create(input) => input.gas_limit(),
                };
                gas -= gas_limit;
            }
            self.tracer.exit_with_gas(opcode, gas)
        }
    }

    #[inline]
    fn call(&mut self, context: &mut CTX, inputs: &mut CallInputs) -> Option<CallOutcome> {
        if context.journal_ref().precompile_addresses().contains(&inputs.bytecode_address) {
            self.tracer.enter_with_gas(inputs.bytecode_address, 0);
        }
        None
    }

    #[inline]
    fn call_end(&mut self, context: &mut CTX, inputs: &CallInputs, outcome: &mut CallOutcome) {
        if context.journal_ref().precompile_addresses().contains(&inputs.bytecode_address) {
            self.tracer.exit_with_gas(inputs.bytecode_address, outcome.result.gas.spent());
        }
    }
}
