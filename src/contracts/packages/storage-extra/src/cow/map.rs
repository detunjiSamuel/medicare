use cosmwasm_std::{StdError, StdResult, Storage};
use cw_storage_plus::{Map, Prefix, PrimaryKey};
use serde::{de::DeserializeOwned, Serialize};
use std::{borrow::Cow, marker::PhantomData};

#[derive(Debug, Clone)]
pub struct MapCow<'a, K, T> {
    pub(crate) namespace: Cow<'a, str>,
    key_type: PhantomData<K>,
    data_type: PhantomData<T>,
}

impl<'a, 'k, K, T> MapCow<'a, K, T>
where
    'k: 'a,
{
    pub const fn new_owned(namespace: String) -> Self {
        Self {
            namespace: Cow::Owned(namespace),
            key_type: PhantomData,
            data_type: PhantomData,
        }
    }

    pub const fn new_ref(namespace: &'k str) -> Self {
        Self {
            namespace: Cow::Borrowed(namespace),
            key_type: PhantomData,
            data_type: PhantomData,
        }
    }
}

impl<'a, 'key, K, T> MapCow<'a, K, T>
where
    T: Serialize + DeserializeOwned,
    K: PrimaryKey<'key>,
    'key: 'a,
{
    pub fn map(&self) -> Map<K, T> {
        Map::new(&self.namespace)
    }

    pub fn prefix(&'key self, p: K::Prefix) -> Prefix<T> {
        self.map().prefix(p)
    }

    pub fn sub_prefix(&'key self, p: K::SubPrefix) -> Prefix<T> {
        self.map().sub_prefix(p)
    }

    pub fn save(&'key self, store: &mut dyn Storage, k: K, data: &T) -> StdResult<()> {
        self.map().save(store, k, data)
    }

    pub fn remove(&'key self, store: &mut dyn Storage, k: K) {
        self.map().remove(store, k)
    }

    pub fn load(&'key self, store: &dyn Storage, k: K) -> StdResult<T> {
        self.map().load(store, k)
    }

    pub fn may_load(&'key self, store: &dyn Storage, k: K) -> StdResult<Option<T>> {
        self.map().may_load(store, k)
    }

    pub fn has(&'key self, store: &dyn Storage, k: K) -> bool {
        self.map().has(store, k)
    }

    pub fn update<A, E>(&'key self, store: &mut dyn Storage, k: K, action: A) -> Result<T, E>
    where
        A: FnOnce(Option<T>) -> Result<T, E>,
        E: From<StdError>,
    {
        self.map().update(store, k, action)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use cosmwasm_std::{testing::MockStorage, Addr};

    #[test]
    fn new_owned() {
        let a = Addr::unchecked("g");
        let mut storage = MockStorage::new();
        let addr_owned: MapCow<&Addr, u64> = MapCow::new_owned(String::from("g"));

        //addr_owned.map().save(&mut storage, &a, &1).unwrap();
        addr_owned.save(&mut storage, &a, &1).unwrap();
        assert_eq!(addr_owned.map().load(&storage, &a).unwrap(), 1);
    }

    #[test]
    fn new_ref() {
        let a = Addr::unchecked("g");
        let mut storage = MockStorage::new();
        const ADDR_REF: MapCow<&Addr, u64> = MapCow::new_ref("g");

        ADDR_REF.map().save(&mut storage, &a, &1).unwrap();
        assert_eq!(ADDR_REF.map().load(&storage, &a).unwrap(), 1);
    }
}
