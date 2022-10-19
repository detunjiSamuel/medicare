use cosmwasm_std::{StdResult, Storage};
use cw_storage_plus::{Index, Map, Prefix, Prefixer, PrimaryKey};
use serde::{de::DeserializeOwned, Serialize};
use std::borrow::Cow;

use super::helpers::{deserialize_multi_kv, DeserializeFn};

#[derive(Clone)]
pub struct CustomDeseMultiIndex<'a, K, T> {
    pub(crate) idx_namespace: Cow<'a, str>,
    pub(crate) pk_namespace: Cow<'a, str>,
    idx_fn: fn(&T, Vec<u8>) -> K,
    dese_fn: Option<DeserializeFn<T>>,
}

impl<'a, K, T> CustomDeseMultiIndex<'a, K, T> {
    pub const fn new_ref(
        idx_fn: fn(&T, Vec<u8>) -> K,
        dese_fn: Option<DeserializeFn<T>>,
        pk_namespace: &'a str,
        idx_namespace: &'a str,
    ) -> Self {
        Self {
            idx_fn,
            dese_fn,
            idx_namespace: Cow::Borrowed(idx_namespace),
            pk_namespace: Cow::Borrowed(pk_namespace),
        }
    }

    pub const fn new_owned(
        idx_fn: fn(&T, Vec<u8>) -> K,
        dese_fn: Option<DeserializeFn<T>>,
        pk_namespace: String,
        idx_namespace: String,
    ) -> Self {
        Self {
            idx_fn,
            dese_fn,
            idx_namespace: Cow::Owned(idx_namespace),
            pk_namespace: Cow::Owned(pk_namespace),
        }
    }
}

impl<'a, K, T> Index<T> for CustomDeseMultiIndex<'a, K, T>
where
    T: Serialize + DeserializeOwned + Clone,
    K: for<'key> PrimaryKey<'key>,
{
    fn save(&self, store: &mut dyn Storage, pk: &[u8], data: &T) -> StdResult<()> {
        let idx = (self.idx_fn)(data, pk.to_vec());
        self.idx_map().save(store, idx, &(pk.len() as u32))
    }

    fn remove(&self, store: &mut dyn Storage, pk: &[u8], old_data: &T) -> StdResult<()> {
        let idx = (self.idx_fn)(old_data, pk.to_vec());
        self.idx_map().remove(store, idx);
        Ok(())
    }
}

impl<'a, K, T> CustomDeseMultiIndex<'a, K, T>
where
    T: Serialize + DeserializeOwned + Clone,
    K: for<'key> PrimaryKey<'key>,
{
    fn idx_map(&self) -> Map<'_, K, u32> {
        Map::new(&self.idx_namespace)
    }

    pub fn prefix(&self, p: <K as PrimaryKey<'_>>::Prefix) -> Prefix<T> {
        Prefix::with_deserialization_function(
            self.idx_namespace.as_bytes(),
            &p.prefix(),
            self.pk_namespace.as_bytes(),
            match self.dese_fn {
                Some(f) => f,
                None => deserialize_multi_kv,
            },
        )
    }

    pub fn sub_prefix(&self, p: <K as PrimaryKey<'_>>::SubPrefix) -> Prefix<T> {
        Prefix::with_deserialization_function(
            self.idx_namespace.as_bytes(),
            &p.prefix(),
            self.pk_namespace.as_bytes(),
            match self.dese_fn {
                Some(f) => f,
                None => deserialize_multi_kv,
            },
        )
    }

    pub fn index_key(&self, k: K) -> Vec<u8> {
        k.joined_key()
    }
}

#[cfg(test)]
mod test {
    use cosmwasm_std::{testing::MockStorage, Order, Uint128};
    use cw_storage_plus::{Index, IndexList, IndexedMap, MultiIndex, PrimaryKey, U128Key, U64Key};
    use serde::{Deserialize, Serialize};

    use crate::cow::deserialize_multi_kv_custom_pk;

    use super::CustomDeseMultiIndex;

    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq, PartialOrd)]
    struct Test {
        id: u64,
        val: Uint128,
    }

    struct TestIndexes<'a> {
        val: CustomDeseMultiIndex<'a, (U128Key, Vec<u8>), Test>,
        val_n: MultiIndex<'a, (U128Key, Vec<u8>), Test>,
    }

    impl IndexList<Test> for TestIndexes<'_> {
        fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<Test>> + '_> {
            let v: Vec<&dyn Index<Test>> = vec![&self.val, &self.val_n];
            Box::new(v.into_iter())
        }
    }

    fn idm<'a>() -> IndexedMap<'a, U64Key, Test, TestIndexes<'a>> {
        IndexedMap::new(
            "test",
            TestIndexes {
                val: CustomDeseMultiIndex::new_ref(
                    |t, _| {
                        (
                            t.val.u128().into(),
                            U64Key::new(u64::max_value() - t.id).joined_key(),
                        )
                    },
                    Some(|s, pk, kv| {
                        deserialize_multi_kv_custom_pk(s, pk, kv, |old_kv| {
                            U64Key::new(
                                u64::max_value()
                                    - u64::from_be_bytes(old_kv.as_slice().try_into().unwrap()),
                            )
                            .joined_key()
                        })
                    }),
                    "test",
                    "test__val",
                ),
                val_n: MultiIndex::new(|t, k| (t.val.u128().into(), k), "test", "test__val_n"),
            },
        )
    }

    #[test]
    fn correct_namespace() {
        let idm = idm();

        assert_eq!(idm.idx.val.pk_namespace, "test");
        assert_eq!(idm.idx.val.idx_namespace, "test__val");
    }

    #[test]
    fn correctly_dese() {
        let mut storage = MockStorage::new();
        idm()
            .save(
                &mut storage,
                0.into(),
                &Test {
                    id: 0,
                    val: Uint128::from(100u64),
                },
            )
            .unwrap();

        let v = idm()
            .idx
            .val
            .sub_prefix(())
            .range(&storage, None, None, Order::Ascending)
            .map(|e| e.unwrap().1.id)
            .collect::<Vec<_>>();

        assert_eq!(v, vec![0]);
    }

    #[test]
    fn index_correctly_use_dese_fn() {
        let mut storage = MockStorage::new();
        idm()
            .save(
                &mut storage,
                0.into(),
                &Test {
                    id: 0,
                    val: Uint128::from(100u64),
                },
            )
            .unwrap();

        idm()
            .save(
                &mut storage,
                1.into(),
                &Test {
                    id: 1,
                    val: Uint128::from(100u64),
                },
            )
            .unwrap();

        idm()
            .save(
                &mut storage,
                2.into(),
                &Test {
                    id: 2,
                    val: Uint128::from(200u64),
                },
            )
            .unwrap();

        idm()
            .save(
                &mut storage,
                3.into(),
                &Test {
                    id: 3,
                    val: Uint128::from(100u64),
                },
            )
            .unwrap();

        let v = idm()
            .idx
            .val
            .sub_prefix(())
            .range(&storage, None, None, Order::Descending)
            .map(|e| e.unwrap().1.id)
            .collect::<Vec<_>>();

        // custom
        // val: Descending, id: Ascending
        assert_eq!(v, vec![2, 0, 1, 3]);

        let vn = idm()
            .idx
            .val_n
            .sub_prefix(())
            .range(&storage, None, None, Order::Descending)
            .map(|e| e.unwrap().1.id)
            .collect::<Vec<_>>();

        // normal
        // val: Descending, id: Descending
        assert_eq!(vn, vec![2, 3, 1, 0]);
    }
}
