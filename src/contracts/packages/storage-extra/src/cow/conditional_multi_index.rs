use cosmwasm_std::{StdResult, Storage};
use cw_storage_plus::{Index, Map, Prefix, Prefixer, PrimaryKey};
use serde::{de::DeserializeOwned, Serialize};
use std::borrow::Cow;

use super::{helpers::deserialize_multi_kv, DeserializeFn};

#[derive(Clone)]
pub struct ConditionalMultiIndex<'a, K, T> {
    pub(crate) idx_namespace: Cow<'a, str>,
    pub(crate) pk_namespace: Cow<'a, str>,
    idx_fn: fn(&T, Vec<u8>) -> K,
    cond_fn: fn(&T) -> bool,
    dese_fn: Option<DeserializeFn<T>>,
}

impl<'a, K, T> ConditionalMultiIndex<'a, K, T> {
    /// Only if result of `cond_fn` is `true`, data will be added to this `ConditionalMultiIndex`.
    ///
    /// Result of `cond_fn` **must be constant**, otherwise might raise unexpected behavior.
    pub const fn new_ref(
        idx_fn: fn(&T, Vec<u8>) -> K,
        cond_fn: fn(&T) -> bool,
        dese_fn: Option<DeserializeFn<T>>,
        pk_namespace: &'a str,
        idx_namespace: &'a str,
    ) -> Self {
        Self {
            idx_fn,
            cond_fn,
            dese_fn,
            idx_namespace: Cow::Borrowed(idx_namespace),
            pk_namespace: Cow::Borrowed(pk_namespace),
        }
    }

    /// Only if result of `cond_fn` is `true`, data will be added to this `ConditionalMultiIndex`.
    ///
    /// Result of `cond_fn` **must be constant**, otherwise might raise unexpected behavior.
    pub const fn new_owned(
        idx_fn: fn(&T, Vec<u8>) -> K,
        cond_fn: fn(&T) -> bool,
        dese_fn: Option<DeserializeFn<T>>,
        pk_namespace: String,
        idx_namespace: String,
    ) -> Self {
        Self {
            idx_fn,
            cond_fn,
            dese_fn,
            idx_namespace: Cow::Owned(idx_namespace),
            pk_namespace: Cow::Owned(pk_namespace),
        }
    }
}

impl<'a, K, T> Index<T> for ConditionalMultiIndex<'a, K, T>
where
    T: Serialize + DeserializeOwned + Clone,
    K: for<'key> PrimaryKey<'key>,
{
    fn save(&self, store: &mut dyn Storage, pk: &[u8], data: &T) -> StdResult<()> {
        if (self.cond_fn)(data) {
            let idx = (self.idx_fn)(data, pk.to_vec());
            self.idx_map().save(store, idx, &(pk.len() as u32))?;
        }

        Ok(())
    }

    fn remove(&self, store: &mut dyn Storage, pk: &[u8], old_data: &T) -> StdResult<()> {
        if (self.cond_fn)(old_data) {
            let idx = (self.idx_fn)(old_data, pk.to_vec());
            self.idx_map().remove(store, idx);
        };

        Ok(())
    }
}

impl<'a, K, T> ConditionalMultiIndex<'a, K, T>
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
    use cosmwasm_std::{testing::MockStorage, Uint128};
    use cw_storage_plus::{Index, IndexList, IndexedMap, MultiIndex, PrimaryKey, U128Key, U64Key};
    use serde::{Deserialize, Serialize};

    use crate::cow::deserialize_multi_kv_custom_pk;

    use super::ConditionalMultiIndex;

    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq, PartialOrd)]
    struct Test {
        id: u64,
        val: Uint128,
    }

    struct TestIndexes<'a> {
        val: ConditionalMultiIndex<'a, (U128Key, Vec<u8>), Test>,
        val_inv: ConditionalMultiIndex<'a, (U128Key, Vec<u8>), Test>,
        val_n: MultiIndex<'a, (U128Key, Vec<u8>), Test>,
    }

    impl IndexList<Test> for TestIndexes<'_> {
        fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<Test>> + '_> {
            let v: Vec<&dyn Index<Test>> = vec![&self.val, &self.val_n, &self.val_inv];
            Box::new(v.into_iter())
        }
    }

    fn idm<'a>() -> IndexedMap<'a, U64Key, Test, TestIndexes<'a>> {
        IndexedMap::new(
            "test",
            TestIndexes {
                val: ConditionalMultiIndex::new_ref(
                    |t, k| (t.val.u128().into(), k),
                    // only add to val if t.val > 100
                    |t| t.val.u128() > 100,
                    None,
                    "test",
                    "test__val",
                ),
                val_inv: ConditionalMultiIndex::new_ref(
                    |t, _| {
                        (
                            t.val.u128().into(),
                            U64Key::new(u64::max_value() - t.id).joined_key(),
                        )
                    },
                    // only add to val if t.val > 100
                    |t| t.val.u128() > 100,
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
                    "test__inv",
                ),
                val_n: MultiIndex::new(|t, k| (t.val.u128().into(), k), "test", "test__normal"),
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
    fn correctly_add_to_index() {
        let mut storage = MockStorage::new();
        idm()
            .save(
                &mut storage,
                0.into(),
                &Test {
                    id: 0,
                    val: Uint128::from(101u64),
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
                    val: Uint128::from(101u64),
                },
            )
            .unwrap();

        let v = idm()
            .idx
            .val
            .sub_prefix(())
            .range(&storage, None, None, cosmwasm_std::Order::Descending)
            .map(|e| e.map(|(_, i)| (i.id, i.val.u128())).unwrap())
            .collect::<Vec<_>>();

        assert_eq!(v, vec![(2, 101), (0, 101)]);

        let v_n = idm()
            .idx
            .val_n
            .sub_prefix(())
            .range(&storage, None, None, cosmwasm_std::Order::Descending)
            .map(|e| e.map(|(_, i)| (i.id, i.val.u128())).unwrap())
            .collect::<Vec<_>>();

        assert_eq!(v_n, vec![(2, 101), (0, 101), (1, 100),]);
    }

    #[test]
    fn correctly_add_to_index_custom_dese() {
        let mut storage = MockStorage::new();
        idm()
            .save(
                &mut storage,
                0.into(),
                &Test {
                    id: 0,
                    val: Uint128::from(101u64),
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
                    val: Uint128::from(101u64),
                },
            )
            .unwrap();

        let v_inv = idm()
            .idx
            .val_inv
            .sub_prefix(())
            .range(&storage, None, None, cosmwasm_std::Order::Descending)
            .map(|e| e.map(|(_, i)| (i.id, i.val.u128())).unwrap())
            .collect::<Vec<_>>();

        assert_eq!(v_inv, vec![(0, 101), (2, 101)]);

        let v_n = idm()
            .idx
            .val_n
            .sub_prefix(())
            .range(&storage, None, None, cosmwasm_std::Order::Descending)
            .map(|e| e.map(|(_, i)| (i.id, i.val.u128())).unwrap())
            .collect::<Vec<_>>();

        assert_eq!(v_n, vec![(2, 101), (0, 101), (1, 100),]);
    }
}
