use cosmwasm_std::{Pair, StdError, StdResult, Storage};
use cw_storage_plus::{Index, IndexList, MultiIndex, Path, Prefix, PrimaryKey, UniqueIndex};
use serde::{de::DeserializeOwned, Serialize};
use std::{borrow::Cow, marker::PhantomData};

use super::indexed_map_ref::IndexedMapRef;

#[derive(Debug, Clone)]
pub struct IndexedMapCow<'a, K, T, I> {
    pub(crate) pk_namespace: Cow<'a, str>,
    pub index: I,
    key_type: PhantomData<K>,
    data_type: PhantomData<T>,
}

impl<'k, K, T, I> IndexedMapCow<'k, K, T, I> {
    pub const fn new_ref(pk_namespace: &'k str, index: I) -> Self {
        Self {
            pk_namespace: Cow::Borrowed(pk_namespace),
            key_type: PhantomData,
            data_type: PhantomData,
            index,
        }
    }

    pub const fn new_owned(pk_namespace: String, index: I) -> Self {
        Self {
            pk_namespace: Cow::Owned(pk_namespace),
            key_type: PhantomData,
            data_type: PhantomData,
            index,
        }
    }
}

impl<'a, K, T, I> IndexedMapCow<'a, K, T, I>
where
    K: PrimaryKey<'a>,
    T: Serialize + DeserializeOwned + Clone,
    I: IndexList<T>,
{
    pub fn indexed_map(&'a self) -> IndexedMapRef<'a, K, T, I> {
        IndexedMapRef::new(&self.pk_namespace, &self.index)
    }

    pub fn key(&'a self, k: K) -> Path<T> {
        self.indexed_map().key(k)
    }

    pub fn save(&'a self, store: &mut dyn Storage, key: K, data: &T) -> StdResult<()> {
        self.indexed_map().save(store, key, data)
    }

    pub fn remove(&'a self, store: &mut dyn Storage, key: K) -> StdResult<()> {
        self.indexed_map().remove(store, key)
    }

    pub fn replace(
        &'a self,
        store: &mut dyn Storage,
        key: K,
        data: Option<&T>,
        old_data: Option<&T>,
    ) -> StdResult<()> {
        self.indexed_map().replace(store, key, data, old_data)
    }

    pub fn update<A, E>(&'a self, store: &mut dyn Storage, key: K, action: A) -> Result<T, E>
    where
        A: FnOnce(Option<T>) -> Result<T, E>,
        E: From<StdError>,
    {
        self.indexed_map().update(store, key, action)
    }

    pub fn load(&'a self, store: &dyn Storage, key: K) -> StdResult<T> {
        self.indexed_map().load(store, key)
    }

    pub fn may_load(&'a self, store: &dyn Storage, key: K) -> StdResult<Option<T>> {
        self.indexed_map().may_load(store, key)
    }

    pub fn prefix(&'a self, p: K::Prefix) -> Prefix<T> {
        self.indexed_map().prefix(p)
    }

    pub fn sub_prefix(&'a self, p: K::SubPrefix) -> Prefix<T> {
        self.indexed_map().sub_prefix(p)
    }
}

#[derive(Clone)]
pub struct MultiIndexCow<'a, K, T> {
    pub(crate) pk_namespace: Cow<'a, str>,
    pub(crate) idx_namespace: Cow<'a, str>,
    idx_fn: fn(&T, Vec<u8>) -> K,
}

impl<'k, K, T> MultiIndexCow<'k, K, T> {
    pub const fn new_ref(
        pk_namespace: &'k str,
        idx_namespace: &'k str,
        idx_fn: fn(&T, Vec<u8>) -> K,
    ) -> Self {
        Self {
            idx_fn,
            pk_namespace: Cow::Borrowed(pk_namespace),
            idx_namespace: Cow::Borrowed(idx_namespace),
        }
    }

    pub const fn new_owned(
        pk_namespace: String,
        idx_namespace: String,
        idx_fn: fn(&T, Vec<u8>) -> K,
    ) -> Self {
        Self {
            idx_fn,
            pk_namespace: Cow::Owned(pk_namespace),
            idx_namespace: Cow::Owned(idx_namespace),
        }
    }
}

impl<K, T> MultiIndexCow<'_, K, T>
where
    T: Serialize + DeserializeOwned + Clone,
    K: for<'key> PrimaryKey<'key>,
{
    pub fn multi_index(&self) -> MultiIndex<K, T> {
        MultiIndex::new(self.idx_fn, &self.pk_namespace, &self.idx_namespace)
    }

    pub fn prefix<'key>(&'key self, p: <K as PrimaryKey<'key>>::Prefix) -> Prefix<T> {
        self.multi_index().prefix(p)
    }

    pub fn sub_prefix<'key>(&'key self, p: <K as PrimaryKey<'key>>::SubPrefix) -> Prefix<T> {
        self.multi_index().sub_prefix(p)
    }

    pub fn index_key(&self, k: K) -> Vec<u8> {
        self.multi_index().index_key(k)
    }
}

impl<K, T> Index<T> for MultiIndexCow<'_, K, T>
where
    T: Serialize + DeserializeOwned + Clone,
    K: for<'key> PrimaryKey<'key>,
{
    fn save(&self, store: &mut dyn Storage, pk: &[u8], data: &T) -> StdResult<()> {
        self.multi_index().save(store, pk, data)
    }

    fn remove(&self, store: &mut dyn Storage, pk: &[u8], old_data: &T) -> StdResult<()> {
        self.multi_index().remove(store, pk, old_data)
    }
}

#[derive(Clone)]
pub struct UniqueIndexCow<'a, K, T> {
    pub(crate) idx_namespace: Cow<'a, str>,
    idx_fn: fn(&T) -> K,
}

impl<'k, K, T> UniqueIndexCow<'k, K, T> {
    pub const fn new_ref(idx_namespace: &'k str, idx_fn: fn(&T) -> K) -> Self {
        Self {
            idx_fn,
            idx_namespace: Cow::Borrowed(idx_namespace),
        }
    }

    pub const fn new_owned(idx_namespace: String, idx_fn: fn(&T) -> K) -> Self {
        Self {
            idx_fn,
            idx_namespace: Cow::Owned(idx_namespace),
        }
    }
}

impl<K, T> UniqueIndexCow<'_, K, T>
where
    T: Serialize + DeserializeOwned + Clone,
    K: for<'key> PrimaryKey<'key>,
{
    pub fn unique_index(&self) -> UniqueIndex<K, T> {
        UniqueIndex::new(self.idx_fn, &self.idx_namespace)
    }

    pub fn index_key(&self, k: K) -> Vec<u8> {
        self.unique_index().index_key(k)
    }

    pub fn prefix<'key>(&'key self, p: <K as PrimaryKey<'key>>::Prefix) -> Prefix<T> {
        self.unique_index().prefix(p)
    }

    pub fn sub_prefix<'key>(&'key self, p: <K as PrimaryKey<'key>>::SubPrefix) -> Prefix<T> {
        self.unique_index().sub_prefix(p)
    }

    pub fn item(&self, store: &dyn Storage, idx: K) -> StdResult<Option<Pair<T>>> {
        self.unique_index().item(store, idx)
    }
}

impl<K, T> Index<T> for UniqueIndexCow<'_, K, T>
where
    T: Serialize + DeserializeOwned + Clone,
    K: for<'key> PrimaryKey<'key>,
{
    fn save(&self, store: &mut dyn Storage, pk: &[u8], data: &T) -> StdResult<()> {
        self.unique_index().save(store, pk, data)
    }

    fn remove(&self, store: &mut dyn Storage, pk: &[u8], old_data: &T) -> StdResult<()> {
        self.unique_index().remove(store, pk, old_data)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use cosmwasm_std::{testing::MockStorage, Addr, Order};
    use cw_storage_plus::U64Key;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
    struct ToIndex {
        id: u64,
        count: u64,
        address: Addr,
    }

    #[derive(Clone)]
    struct ToIndexList<'a> {
        count: MultiIndexCow<'a, (U64Key, Vec<u8>), ToIndex>,
        address: UniqueIndexCow<'a, Addr, ToIndex>,
    }

    impl IndexList<ToIndex> for ToIndexList<'_> {
        fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<ToIndex>> + '_> {
            let v: Vec<&dyn Index<ToIndex>> = vec![&self.count, &self.address];
            Box::new(v.into_iter())
        }
    }

    #[test]
    fn new_ref() {
        let mut storage = MockStorage::new();
        const TO: IndexedMapCow<U64Key, ToIndex, ToIndexList> = IndexedMapCow::new_ref(
            "primary",
            ToIndexList {
                count: MultiIndexCow::new_ref("primary", "primary_count", |e, k| {
                    (e.count.into(), k)
                }),
                address: UniqueIndexCow::new_ref("primary_address", |e| e.address.clone()),
            },
        );

        let first = ToIndex {
            id: 0,
            count: 5,
            address: Addr::unchecked("a"),
        };

        TO.save(&mut storage, first.id.into(), &first).unwrap();

        assert_eq!(
            TO.indexed_map().load(&storage, first.id.into()).unwrap(),
            first
        );

        let second = ToIndex {
            id: 1,
            count: 5,
            address: Addr::unchecked("b"),
        };

        TO.save(&mut storage, second.id.into(), &second).unwrap();

        assert_eq!(
            TO.index
                .count
                .prefix(5.into())
                .range(&storage, None, None, Order::Ascending)
                .map(|e| e.unwrap().1)
                .collect::<Vec<_>>(),
            vec![first.clone(), second.clone()]
        );

        assert_eq!(
            TO.index
                .address
                .item(&storage, Addr::unchecked("a"))
                .unwrap()
                .unwrap()
                .1,
            first
        );

        TO.remove(&mut storage, first.id.into()).unwrap();

        assert_eq!(
            TO.index
                .count
                .prefix(5.into())
                .range(&storage, None, None, Order::Ascending)
                .map(|e| e.unwrap().1)
                .collect::<Vec<_>>(),
            vec![second.clone()]
        );

        assert_eq!(
            TO.index
                .address
                .item(&storage, Addr::unchecked("a"))
                .unwrap(),
            None
        );
    }

    #[test]
    fn new_owned() {
        let mut storage = MockStorage::new();
        let to: IndexedMapCow<U64Key, ToIndex, ToIndexList> = IndexedMapCow::new_owned(
            "primary".to_string(),
            ToIndexList {
                count: MultiIndexCow::new_owned(
                    "primary".to_string(),
                    "primary_count".to_string(),
                    |e, k| (e.count.into(), k),
                ),
                address: UniqueIndexCow::new_owned("primary_address".to_string(), |e| {
                    e.address.clone()
                }),
            },
        );

        let first = ToIndex {
            id: 0,
            count: 5,
            address: Addr::unchecked("a"),
        };

        to.save(&mut storage, first.id.into(), &first).unwrap();

        assert_eq!(
            to.indexed_map().load(&storage, first.id.into()).unwrap(),
            first
        );

        let second = ToIndex {
            id: 1,
            count: 5,
            address: Addr::unchecked("b"),
        };

        to.save(&mut storage, second.id.into(), &second).unwrap();

        assert_eq!(
            to.index
                .count
                .prefix(5.into())
                .range(&storage, None, None, Order::Ascending)
                .map(|e| e.unwrap().1)
                .collect::<Vec<_>>(),
            vec![first.clone(), second.clone()]
        );

        assert_eq!(
            to.index
                .address
                .item(&storage, Addr::unchecked("a"))
                .unwrap()
                .unwrap()
                .1,
            first
        );

        to.remove(&mut storage, first.id.into()).unwrap();

        assert_eq!(
            to.index
                .count
                .prefix(5.into())
                .range(&storage, None, None, Order::Ascending)
                .map(|e| e.unwrap().1)
                .collect::<Vec<_>>(),
            vec![second.clone()]
        );

        assert_eq!(
            to.index
                .address
                .item(&storage, Addr::unchecked("a"))
                .unwrap(),
            None
        );
    }
}
