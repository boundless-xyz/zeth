// Copyright 2023 RISC Zero, Inc.
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

use anyhow::Result;
use revm::{Database, DatabaseCommit};
use zeth_primitives::{
    block::Header,
    transactions::{ethereum::EthereumTxEssence, optimism::OptimismTxEssence, TxEssence},
    trie::MptNode,
};

use crate::{
    builder::{
        execute::{ethereum::EthTxExecStrategy, optimism::OpTxExecStrategy, TxExecStrategy},
        finalize::{BlockFinalizeStrategy, MemDbBlockFinalizeStrategy},
        initialize::{DbInitStrategy, MemDbInitStrategy},
        prepare::{EthHeaderPrepStrategy, HeaderPrepStrategy},
    },
    consts::ChainSpec,
    input::Input,
    mem_db::MemDb,
};

mod execute;
mod finalize;
mod initialize;
mod prepare;

/// A generic builder for building a block.
#[derive(Clone, Debug)]
pub struct BlockBuilder<'a, D, E: TxEssence> {
    pub(crate) chain_spec: &'a ChainSpec,
    pub(crate) input: Input<E>,
    pub(crate) db: Option<D>,
    pub(crate) header: Option<Header>,
}

impl<D, E> BlockBuilder<'_, D, E>
where
    D: Database + DatabaseCommit,
    <D as Database>::Error: core::fmt::Debug,
    E: TxEssence,
{
    /// Creates a new block builder.
    pub fn new(chain_spec: &ChainSpec, input: Input<E>) -> BlockBuilder<'_, D, E> {
        BlockBuilder {
            chain_spec,
            db: None,
            header: None,
            input,
        }
    }

    /// Sets the database instead of initializing it from the input.
    pub fn with_db(mut self, db: D) -> Self {
        self.db = Some(db);
        self
    }

    /// Initializes the database from the input.
    pub fn initialize_database<T: DbInitStrategy<D>>(self) -> Result<Self> {
        T::initialize_database(self)
    }

    /// Initializes the header. This must be called before executing transactions.
    pub fn prepare_header<T: HeaderPrepStrategy>(self) -> Result<Self> {
        T::prepare_header(self)
    }

    /// Executes all input transactions.
    pub fn execute_transactions<T: TxExecStrategy<E>>(self) -> Result<Self> {
        T::execute_transactions(self)
    }

    /// Finalizes the block building and returns the header and the state trie.
    pub fn finalize<T: BlockFinalizeStrategy<D>>(self) -> Result<(Header, MptNode)> {
        T::finalize(self)
    }

    /// Returns a reference to the database.
    pub fn db(&self) -> Option<&D> {
        self.db.as_ref()
    }

    /// Returns a mutable reference to the database.
    pub fn mut_db(&mut self) -> Option<&mut D> {
        self.db.as_mut()
    }
}

/// A bundle of strategies for building a block using [BlockBuilder].
pub trait BlockBuilderStrategy {
    type TxEssence: TxEssence;

    type DbInitStrategy: DbInitStrategy<MemDb>;
    type HeaderPrepStrategy: HeaderPrepStrategy;
    type TxExecStrategy: TxExecStrategy<Self::TxEssence>;
    type BlockFinalizeStrategy: BlockFinalizeStrategy<MemDb>;

    /// Builds a block from the given input.
    fn build_from(
        chain_spec: &ChainSpec,
        input: Input<Self::TxEssence>,
    ) -> Result<(Header, MptNode)> {
        BlockBuilder::<MemDb, Self::TxEssence>::new(chain_spec, input)
            .initialize_database::<Self::DbInitStrategy>()?
            .prepare_header::<Self::HeaderPrepStrategy>()?
            .execute_transactions::<Self::TxExecStrategy>()?
            .finalize::<Self::BlockFinalizeStrategy>()
    }
}

/// The [BlockBuilderStrategy] for building an Ethereum block.
pub struct EthereumStrategy {}

impl BlockBuilderStrategy for EthereumStrategy {
    type TxEssence = EthereumTxEssence;
    type DbInitStrategy = MemDbInitStrategy;
    type HeaderPrepStrategy = EthHeaderPrepStrategy;
    type TxExecStrategy = EthTxExecStrategy;
    type BlockFinalizeStrategy = MemDbBlockFinalizeStrategy;
}

/// The [BlockBuilderStrategy] for building an Optimism block.
pub struct OptimismStrategy {}

impl BlockBuilderStrategy for OptimismStrategy {
    type TxEssence = OptimismTxEssence;
    type DbInitStrategy = MemDbInitStrategy;
    type HeaderPrepStrategy = EthHeaderPrepStrategy;
    type TxExecStrategy = OpTxExecStrategy;
    type BlockFinalizeStrategy = MemDbBlockFinalizeStrategy;
}
