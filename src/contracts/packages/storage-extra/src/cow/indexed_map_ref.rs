/// Another varaint of cw-storage-plus IndexedMap. Using reference of indexes instead of owned to
/// avoid cloning/rebuilding as accessor and remove trait bound of new constructor to make it constant.
///
/// Modified from:
/// https://github.com/CosmWasm/cw-plus/blob/v0.9.1/packages/storage-plus/src/indexed_map.rs
use cosmwasm_std::{StdError, StdResult, Storage};
use cw_storage_plus::{IndexList, Map, Path, Prefix, Prefixer, PrimaryKey};
use serde::{de::DeserializeOwned, Serialize};

pub struct IndexedMapRef<'a, K, T, I> {
    pk_namespace: &'a [u8],
    primary: Map<'a, K, T>,
    idx: &'a I,
}

impl<'a, K, T, I> IndexedMapRef<'a, K, T, I> {
    pub const fn new(pk_namespace: &'a str, indexes: &'a I) -> Self {
        Self {
            pk_namespace: pk_namespace.as_bytes(),
            primary: Map::new(pk_namespace),
            idx: indexes,
        }
    }
}

impl<'a, K, T, I> IndexedMapRef<'a, K, T, I>
where
    K: PrimaryKey<'a>,
    T: Serialize + DeserializeOwned + Clone,
    I: IndexList<T>,
{
    pub fn key(&self, k: K) -> Path<T> {
        self.primary.key(k)
    }

    pub fn save(&self, store: &mut dyn Storage, key: K, data: &T) -> StdResult<()> {
        let old_data = self.may_load(store, key.clone())?;
        self.replace(store, key, Some(data), old_data.as_ref())
    }

    pub fn remove(&self, store: &mut dyn Storage, key: K) -> StdResult<()> {
        let old_data = self.may_load(store, key.clone())?;
        self.replace(store, key, None, old_data.as_ref())
    }

    pub fn replace(
        &self,
        store: &mut dyn Storage,
        key: K,
        data: Option<&T>,
        old_data: Option<&T>,
    ) -> StdResult<()> {
        let pk = key.joined_key();
        if let Some(old) = old_data {
            for index in self.idx.get_indexes() {
                index.remove(store, &pk, old)?;
            }
        }
        if let Some(updated) = data {
            for index in self.idx.get_indexes() {
                index.save(store, &pk, updated)?;
            }
            self.primary.save(store, key, updated)?;
        } else {
            self.primary.remove(store, key);
        }
        Ok(())
    }

    pub fn update<A, E>(&self, store: &mut dyn Storage, key: K, action: A) -> Result<T, E>
    where
        A: FnOnce(Option<T>) -> Result<T, E>,
        E: From<StdError>,
    {
        let input = self.may_load(store, key.clone())?;
        let old_val = input.clone();
        let output = action(input)?;
        self.replace(store, key, Some(&output), old_val.as_ref())?;
        Ok(output)
    }

    pub fn load(&self, store: &dyn Storage, key: K) -> StdResult<T> {
        self.primary.load(store, key)
    }

    pub fn may_load(&self, store: &dyn Storage, key: K) -> StdResult<Option<T>> {
        self.primary.may_load(store, key)
    }

    pub fn prefix(&self, p: K::Prefix) -> Prefix<T> {
        Prefix::new(self.pk_namespace, &p.prefix())
    }

    pub fn sub_prefix(&self, p: K::SubPrefix) -> Prefix<T> {
        Prefix::new(self.pk_namespace, &p.prefix())
    }
}
