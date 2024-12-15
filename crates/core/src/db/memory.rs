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

use crate::db::unreachable::UnreachableDB;
use crate::db::update::Update;
use crate::rescue::Recoverable;
use alloy_primitives::map::HashSet;
use alloy_primitives::{B256, U256};
use reth_primitives::revm_primitives::AccountInfo;
use reth_revm::db::states::{PlainStorageChangeset, StateChangeset};
use reth_revm::db::{AccountState, CacheDB};

pub type MemoryDB = CacheDB<UnreachableDB>;

impl<DB: Default> Recoverable for CacheDB<DB> {
    fn rescue(&mut self) -> Option<Self> {
        Some(core::mem::take(self))
    }
}

impl<DB> Update for CacheDB<DB> {
    fn apply_changeset(&mut self, changeset: StateChangeset) -> anyhow::Result<()> {
        // Update accounts in state trie
        let mut was_destroyed = HashSet::new();
        for (address, account_info) in changeset.accounts {
            let db_account = self.accounts.get_mut(&address).unwrap();
            let Some(info) = account_info else {
                db_account.storage.clear();
                db_account.account_state = AccountState::NotExisting;
                db_account.info = AccountInfo::default();
                was_destroyed.insert(address);
                continue;
            };
            if info.code_hash != db_account.info.code_hash {
                db_account.info = info;
            } else {
                db_account.info.balance = info.balance;
                db_account.info.nonce = info.nonce;
            }
        }
        // Update account storages
        for PlainStorageChangeset {
            address,
            wipe_storage,
            storage,
        } in changeset.storage
        {
            if was_destroyed.contains(&address) {
                continue;
            }
            let db_account = self.accounts.get_mut(&address).unwrap();
            if wipe_storage {
                db_account.storage.clear();
                db_account.account_state = AccountState::StorageCleared;
            }
            for (key, val) in storage {
                db_account.storage.insert(key, val);
            }
        }
        Ok(())
    }
    fn insert_block_hash(&mut self, block_number: U256, block_hash: B256) -> anyhow::Result<()> {
        self.block_hashes.insert(block_number, block_hash);
        Ok(())
    }
}
