use alloy_primitives::map::B256Set;
use alloy_primitives::B256;
use alloy_rlp::{Decodable, Encodable};
use risc0_ethereum_trie::{orphan, CachedTrie};
use serde::{Deserialize, Serialize};
use std::borrow::Borrow;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

#[derive(
    Debug,
    Clone,
    Eq,
    PartialEq,
    Deserialize,
    Serialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
pub struct MptNode<T: Decodable + Encodable> {
    inner: CachedTrie,
    phantom_data: PhantomData<T>,
}

impl<T: Decodable + Encodable> Default for MptNode<T> {
    fn default() -> Self {
        Self {
            inner: CachedTrie::default(),
            phantom_data: PhantomData,
        }
    }
}

impl<T: Decodable + Encodable> MptNode<T> {
    pub fn get_rlp(&self, key: impl AsRef<[u8]>) -> alloy_rlp::Result<Option<T>> {
        match self.inner.get(key) {
            None => Ok(None),
            Some(mut bytes) => Ok(Some(T::decode(&mut bytes)?)),
        }
    }

    pub fn insert_rlp<K, V>(&mut self, key: K, value: V)
    where
        K: AsRef<[u8]>,
        V: Borrow<T>,
    {
        self.inner.insert(key, alloy_rlp::encode(value.borrow()));
    }

    /// Tries to resolve the potential removal orphan corresponding to `key` from the given
    /// post-removal proof. If the orphan cannot be resolved from the proof alone, the `key`
    /// corresponding to the unresolved path is added to `unresolvable`.
    pub fn resolve_orphan<K, N>(
        &mut self,
        key: K,
        post_state_proof: impl IntoIterator<Item = N>,
        unresolvable: &mut B256Set,
    ) -> anyhow::Result<()>
    where
        K: AsRef<[u8]>,
        N: AsRef<[u8]>,
    {
        match self.inner.resolve_orphan(key, post_state_proof) {
            Ok(_) => {}
            Err(orphan::Error::Unresolvable(prefix)) => {
                // convert the unresolvable prefix nibbles into a B256 key with zero padding
                let key = B256::right_padding_from(&prefix.pack());
                unresolvable.insert(key);
            }
            Err(err) => return Err(err.into()),
        };

        Ok(())
    }

    #[inline]
    pub fn from_digest(digest: B256) -> Self {
        if digest == B256::ZERO {
            Self::default()
        } else {
            Self {
                inner: CachedTrie::from_digest(digest),
                phantom_data: PhantomData,
            }
        }
    }

    #[inline]
    pub fn from_rlp<N: AsRef<[u8]>>(nodes: impl IntoIterator<Item = N>) -> alloy_rlp::Result<Self> {
        Ok(Self {
            inner: CachedTrie::from_rlp(nodes)?,
            phantom_data: PhantomData,
        })
    }
}

impl<T: Decodable + Encodable> Deref for MptNode<T> {
    type Target = CachedTrie;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: Decodable + Encodable> DerefMut for MptNode<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
