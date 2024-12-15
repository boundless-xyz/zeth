// Copyright 2024 RISC Zero, Inc.
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

use crate::driver::PreflightDriver;
use crate::provider::db::ProviderDB;
use crate::provider::get_proofs;
use crate::provider::query::{AccountRangeQuery, BlockQuery, ProofQuery, StorageRangeQuery};
use crate::trie::TrieOrphan;
use alloy::network::Network;
use alloy::primitives::map::HashMap;
use alloy::primitives::{Address, B256, U256};
use alloy::rpc::types::EIP1186AccountProofResponse;
use log::{error, warn};
use reth_primitives::revm_primitives::{Account, AccountInfo, Bytecode};
use reth_revm::db::states::StateChangeset;
use reth_revm::db::CacheDB;
use reth_revm::{Database, DatabaseCommit, DatabaseRef};
use std::cell::{Ref, RefCell};
use std::marker::PhantomData;
use std::ops::DerefMut;
use zeth_core::db::update::Update;
use zeth_core::driver::CoreDriver;
use zeth_core::rescue::{Recoverable, Rescued};

/// Wraps a [`Database`] to provide a [`DatabaseRef`] implementation.
#[derive(Clone, Debug, Default)]
pub struct MutDB<T> {
    pub db: RefCell<T>,
}

impl<T: Database> MutDB<T> {
    pub fn new(db: T) -> Self {
        Self {
            db: RefCell::new(db),
        }
    }

    pub fn borrow_db(&self) -> Ref<T> {
        self.db.borrow()
    }
}

impl<T: Database> DatabaseRef for MutDB<T> {
    type Error = <T as Database>::Error;

    fn basic_ref(&self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        self.db.borrow_mut().basic(address)
    }

    fn code_by_hash_ref(&self, code_hash: B256) -> Result<Bytecode, Self::Error> {
        self.db.borrow_mut().code_by_hash(code_hash)
    }

    fn storage_ref(&self, address: Address, index: U256) -> Result<U256, Self::Error> {
        self.db.borrow_mut().storage(address, index)
    }

    fn block_hash_ref(&self, number: u64) -> Result<B256, Self::Error> {
        self.db.borrow_mut().block_hash(number)
    }
}

pub type PrePostDB<N, R, P> = CacheDB<MutDB<CacheDB<MutDB<ProviderDB<N, R, P>>>>>;

#[derive(Clone)]
pub struct PreflightDB<N: Network, R: CoreDriver, P: PreflightDriver<R, N>> {
    pub inner: PrePostDB<N, R, P>,
    pub driver: PhantomData<R>,
}

impl<N: Network, R: CoreDriver, P: PreflightDriver<R, N>> Recoverable for PreflightDB<N, R, P>
where
    R: Clone,
    P: Clone,
{
    fn rescue(&mut self) -> Option<Self> {
        Some(self.clone())
    }
}

impl<N: Network, R: CoreDriver, P: PreflightDriver<R, N>> From<ProviderDB<N, R, P>>
    for PreflightDB<N, R, P>
{
    fn from(value: ProviderDB<N, R, P>) -> Self {
        Self {
            inner: CacheDB::new(MutDB::new(CacheDB::new(MutDB::new(value)))),
            driver: PhantomData,
        }
    }
}

impl<N: Network, R: CoreDriver, P: PreflightDriver<R, N>> From<PrePostDB<N, R, P>>
    for PreflightDB<N, R, P>
{
    fn from(value: PrePostDB<N, R, P>) -> Self {
        Self {
            inner: value,
            driver: PhantomData,
        }
    }
}

impl<N: Network, R: CoreDriver, P: PreflightDriver<R, N>> From<Rescued<PrePostDB<N, R, P>>>
    for PreflightDB<N, R, P>
{
    fn from(value: Rescued<PrePostDB<N, R, P>>) -> Self {
        value.lock().unwrap().take().unwrap().into()
    }
}

impl<N: Network, R: CoreDriver, P: PreflightDriver<R, N>> PreflightDB<N, R, P> {
    pub fn clear(&mut self) -> anyhow::Result<()> {
        let cleared = Self::from(self.inner.db.borrow_db().db.borrow_db().clone());
        drop(core::mem::replace(self, cleared));
        Ok(())
    }

    pub fn save_provider(&mut self) -> anyhow::Result<()> {
        self.inner.db.db.borrow_mut().db.db.borrow().save_provider()
    }

    pub fn advance_provider_block(&mut self) -> anyhow::Result<()> {
        self.inner
            .db
            .db
            .borrow_mut()
            .db
            .db
            .borrow_mut()
            .advance_provider_block()
    }

    pub fn apply_changeset(&mut self, state_changeset: StateChangeset) -> anyhow::Result<()> {
        self.inner.apply_changeset(state_changeset)
    }

    pub fn sanity_check(&mut self, state_changeset: StateChangeset) -> anyhow::Result<()> {
        // storage sanity check
        let initial_db = &self.inner.db;
        let mut provider_db = initial_db.db.borrow().db.db.borrow().clone();
        provider_db.block_no += 1;
        for (address, db_account) in &self.inner.accounts {
            use reth_revm::Database;
            let provider_info = provider_db.basic(*address)?.unwrap_or_default();
            if db_account.info != provider_info {
                error!("State difference for account {address}:");
                if db_account.info.balance != provider_info.balance {
                    error!(
                        "Calculated balance is {} while provider reports balance is {}",
                        db_account.info.balance, provider_info.balance
                    );
                }
                if db_account.info.nonce != provider_info.nonce {
                    error!(
                        "Calculated nonce is {} while provider reports nonce is {}",
                        db_account.info.nonce, provider_info.nonce
                    );
                }
                if db_account.info.code_hash != provider_info.code_hash {
                    error!(
                        "Calculated code_hash is {} while provider reports code_hash is {}",
                        db_account.info.code_hash, provider_info.code_hash
                    );
                }
                if let Some((_, info)) = state_changeset
                    .accounts
                    .iter()
                    .find(|(addr, _)| addr == address)
                {
                    error!("Info of account was to be updated to {info:?}");
                } else {
                    error!("No update was scheduled for this account.")
                }
            }
        }
        Ok(())
    }

    pub fn get_initial_proofs(
        &mut self,
    ) -> anyhow::Result<HashMap<Address, EIP1186AccountProofResponse>> {
        let initial_db = self.inner.db.borrow_db();
        let storage_keys = enumerate_storage_keys(&initial_db);
        let block_no = initial_db.db.borrow_db().block_no;
        let res = get_proofs(
            initial_db.db.borrow_db().provider.borrow_mut().deref_mut(),
            block_no,
            storage_keys,
        )?;
        Ok(res)
    }

    pub fn get_latest_proofs(
        &mut self,
    ) -> anyhow::Result<HashMap<Address, EIP1186AccountProofResponse>> {
        // get initial keys
        let initial_db = self.inner.db.borrow_db();
        let mut storage_keys = enumerate_storage_keys(&initial_db);
        // merge initial keys with latest db storage keys
        for (address, mut indices) in enumerate_storage_keys(&self.inner) {
            match storage_keys.get_mut(&address) {
                Some(initial_indices) => initial_indices.append(&mut indices),
                None => {
                    storage_keys.insert(address, indices);
                }
            }
        }
        // return proofs as of next block
        let block_no = initial_db.db.borrow_db().block_no + 1;
        let res = get_proofs(
            initial_db.db.borrow_db().provider.borrow_mut().deref_mut(),
            block_no,
            storage_keys,
        )?;
        Ok(res)
    }

    pub fn get_ancestor_headers(&mut self) -> anyhow::Result<Vec<N::HeaderResponse>> {
        let initial_db = self.inner.db.db.borrow_mut();
        let db_block_number = initial_db.db.borrow_db().block_no;
        let earliest_block = initial_db
            .block_hashes
            .keys()
            .min()
            .copied()
            .map(|v| v.to())
            .unwrap_or(db_block_number);
        let provider_db = initial_db.db.borrow_db();
        let mut provider = provider_db.provider.borrow_mut();
        let headers = (earliest_block..db_block_number)
            .rev()
            .map(|block_no| {
                P::derive_header_response(
                    provider
                        .get_full_block(&BlockQuery { block_no })
                        .expect("Failed to retrieve ancestor block"),
                )
            })
            .collect();
        Ok(headers)
    }

    pub fn resolve_orphans(
        &mut self,
        block_count: u64,
        state_orphans: &[TrieOrphan],
        storage_orphans: &[(Address, TrieOrphan)],
    ) -> Vec<EIP1186AccountProofResponse> {
        let state_resolves = self.resolve_state_orphans(state_orphans, block_count);
        let storage_resolves = self.resolve_storage_orphans(storage_orphans, block_count);
        state_resolves.into_iter().chain(storage_resolves).collect()
    }

    pub fn resolve_state_orphans(
        &mut self,
        state_orphans: &[TrieOrphan],
        block_count: u64,
    ) -> Vec<EIP1186AccountProofResponse> {
        let initial_db = self.inner.db.db.borrow_mut();
        let provider_db = initial_db.db.borrow_db();
        let mut provider = provider_db.provider.borrow_mut();
        let mut result = Vec::new();
        let block_no = initial_db.db.borrow_db().block_no + block_count - 1;
        for (start, digest) in state_orphans {
            if let Ok(next_account) = provider.get_next_account(&AccountRangeQuery {
                block_no,
                start: *start,
                max_results: 1,
                no_code: true,
                no_storage: true,
                incompletes: false,
            }) {
                if let Ok(proof) = provider.get_proof(&ProofQuery {
                    block_no,
                    address: next_account,
                    indices: Default::default(),
                }) {
                    result.push(proof);
                    continue;
                }
                continue;
            }
            warn!("state orphan {digest} not found");
        }
        result
    }

    pub fn resolve_storage_orphans(
        &mut self,
        storage_orphans: &[(Address, TrieOrphan)],
        block_count: u64,
    ) -> Vec<EIP1186AccountProofResponse> {
        let initial_db = self.inner.db.db.borrow_mut();
        let provider_db = initial_db.db.borrow_db();
        let mut provider = provider_db.provider.borrow_mut();
        let mut result = Vec::new();
        let block_no = initial_db.db.borrow_db().block_no + block_count - 1;
        for (address, (start, digest)) in storage_orphans {
            // if let Ok(val) = provider.get_preimage(&PreimageQuery { digest: *digest }) {
            //     continue;
            // }
            if let Ok(next_slot) = provider.get_next_slot(&StorageRangeQuery {
                block_no,
                tx_index: 0,
                address: *address,
                start: *start,
                max_results: 1,
            }) {
                if let Ok(proof) = provider.get_proof(&ProofQuery {
                    block_no,
                    address: *address,
                    indices: vec![next_slot]
                        .into_iter()
                        .map(|x| x.to_be_bytes().into())
                        .collect(),
                }) {
                    result.push(proof);
                    continue;
                }
            }
            warn!("storage orphan {address}/{digest} not found");
        }
        result
    }
}

pub fn enumerate_storage_keys<T>(db: &CacheDB<T>) -> HashMap<Address, Vec<U256>> {
    db.accounts
        .iter()
        .map(|(address, account)| (*address, account.storage.keys().cloned().collect()))
        .collect()
}

impl<N: Network, R: CoreDriver, P: PreflightDriver<R, N>> Database for PreflightDB<N, R, P> {
    type Error = <PrePostDB<N, R, P> as Database>::Error;

    fn basic(&mut self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        self.inner.basic(address)
    }

    fn code_by_hash(&mut self, code_hash: B256) -> Result<Bytecode, Self::Error> {
        self.inner.code_by_hash(code_hash)
    }

    fn storage(&mut self, address: Address, index: U256) -> Result<U256, Self::Error> {
        self.inner.storage(address, index)
    }

    fn block_hash(&mut self, number: u64) -> Result<B256, Self::Error> {
        self.inner.block_hash(number)
    }
}

impl<N: Network, R: CoreDriver, P: PreflightDriver<R, N>> DatabaseRef for PreflightDB<N, R, P> {
    type Error = <PrePostDB<N, R, P> as DatabaseRef>::Error;

    fn basic_ref(&self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        self.inner.basic_ref(address)
    }

    fn code_by_hash_ref(&self, code_hash: B256) -> Result<Bytecode, Self::Error> {
        self.inner.code_by_hash_ref(code_hash)
    }

    fn storage_ref(&self, address: Address, index: U256) -> Result<U256, Self::Error> {
        self.inner.storage_ref(address, index)
    }

    fn block_hash_ref(&self, number: u64) -> Result<B256, Self::Error> {
        self.inner.block_hash_ref(number)
    }
}

impl<N: Network, R: CoreDriver, P: PreflightDriver<R, N>> DatabaseCommit for PreflightDB<N, R, P> {
    fn commit(&mut self, changes: HashMap<Address, Account>) {
        self.inner.commit(changes)
    }
}
