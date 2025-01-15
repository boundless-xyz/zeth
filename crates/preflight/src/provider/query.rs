// Copyright 2023, 2024 RISC Zero, Inc.
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

use alloy::primitives::{Address, B256, U256};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub struct AccountQuery {
    pub block_no: u64,
    pub address: Address,
}

#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
pub struct BlockQuery {
    pub block_no: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
pub struct UncleQuery {
    pub block_no: u64,
    pub uncle_index: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
pub struct ProofQuery {
    pub block_no: u64,
    pub address: Address,
    pub indices: BTreeSet<B256>,
}

#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
pub struct StorageQuery {
    pub block_no: u64,
    pub address: Address,
    pub index: U256,
}

#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
pub struct PreimageQuery {
    pub digest: B256,
}

#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
pub struct AccountRangeQuery {
    pub block_no: u64,
    pub start: B256,
    pub max_results: u64,
    pub no_code: bool,
    pub no_storage: bool,
    pub incompletes: bool,
}

impl AccountRangeQuery {
    pub fn new(block_no: u64, start: B256) -> Self {
        Self {
            block_no,
            start,
            max_results: 1,
            no_code: true,
            no_storage: true,
            incompletes: true,
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountRangeQueryResponse {
    pub root: B256,
    pub accounts: HashMap<Address, AccountRangeQueryResponseEntry>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub next: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountRangeQueryResponseEntry {
    pub balance: U256,
    pub nonce: U256,
    pub root: B256,
    pub code_hash: B256,
    pub address: Address,
    pub key: B256,
}

#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
pub struct StorageRangeQuery {
    pub block_no: u64,
    pub tx_index: u64,
    pub address: Address,
    pub start: B256,
    pub max_results: u64,
}

impl StorageRangeQuery {
    pub fn new(block_no: u64, address: Address, start: B256) -> Self {
        Self {
            block_no,
            tx_index: 0,
            address,
            start,
            max_results: 1,
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageRangeQueryResponse {
    pub storage: HashMap<B256, StorageRangeQueryResponseEntry>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub next_key: Option<B256>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct StorageRangeQueryResponseEntry {
    pub key: U256,
    pub value: U256,
}
