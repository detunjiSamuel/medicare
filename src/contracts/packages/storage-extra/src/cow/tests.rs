use cosmwasm_std::{testing::MockStorage, Addr, Order};
use cw_storage_plus::{Bound, Index, IndexList, U64Key};
use serde::{Deserialize, Serialize};

use super::*;

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

struct ItemMapAccessor<'a, 'key> {
    item: ItemCow<'a, u64>,
    map: MapCow<'a, &'key Addr, u64>,
    indexed_map: IndexedMapCow<'a, U64Key, ToIndex, ToIndexList<'a>>,
}

impl ItemMapAccessor<'_, '_> {
    fn new(primary_ns: &str) -> Self {
        Self {
            item: ItemCow::new_owned(format!("{}-item", primary_ns)),
            map: MapCow::new_owned(format!("{}-map", primary_ns)),
            indexed_map: IndexedMapCow::new_owned(
                format!("{}-idm", primary_ns),
                ToIndexList {
                    count: MultiIndexCow::new_owned(
                        format!("{}-idm", primary_ns),
                        format!("{}-idm-count", primary_ns),
                        |e, k| (e.count.into(), k),
                    ),
                    address: UniqueIndexCow::new_owned(format!("{}-idm-addr", primary_ns), |e| {
                        e.address.clone()
                    }),
                },
            ),
        }
    }
}

#[test]
fn correct_namespace() {
    let it = ItemMapAccessor::new("primary");
    assert_eq!(it.item.namespace, "primary-item");
    assert_eq!(it.map.namespace, "primary-map");
    assert_eq!(it.indexed_map.pk_namespace, "primary-idm");
    assert_eq!(it.indexed_map.index.count.pk_namespace, "primary-idm");
    assert_eq!(
        it.indexed_map.index.count.idx_namespace,
        "primary-idm-count"
    );
    assert_eq!(
        it.indexed_map.index.address.idx_namespace,
        "primary-idm-addr"
    );
}

#[test]
fn ensure_uninit() {
    let storage = MockStorage::new();
    let it = ItemMapAccessor::new("primary");

    assert_eq!(it.item.may_load(&storage).unwrap(), None);
    assert_eq!(
        it.map
            .prefix(())
            .range(&storage, None, None, Order::Ascending)
            .collect::<Vec<_>>(),
        vec![]
    );
    assert_eq!(
        it.indexed_map
            .prefix(())
            .range(&storage, None, None, Order::Ascending)
            .collect::<Vec<_>>(),
        vec![]
    );
}

#[test]
fn item_works() {
    let mut storage = MockStorage::new();
    let it = ItemMapAccessor::new("primary");

    assert_eq!(it.item.may_load(&storage).unwrap(), None);

    it.item.save(&mut storage, &1).unwrap();

    assert_eq!(it.item.may_load(&storage).unwrap(), Some(1));

    it.item.save(&mut storage, &555).unwrap();

    assert_eq!(it.item.may_load(&storage).unwrap(), Some(555));
}

#[test]
fn map_works() {
    let mut storage = MockStorage::new();
    let it = ItemMapAccessor::new("primary");

    let a = Addr::unchecked("a");
    let b = Addr::unchecked("b");

    assert_eq!(
        it.map
            .prefix(())
            .range(&storage, None, None, Order::Ascending)
            .map(|e| e
                .map(|res| (Addr::unchecked(String::from_utf8(res.0).unwrap()), res.1))
                .unwrap())
            .collect::<Vec<_>>(),
        vec![]
    );

    it.map.save(&mut storage, &a, &1).unwrap();

    assert_eq!(
        it.map
            .prefix(())
            .range(&storage, None, None, Order::Ascending)
            .map(|e| e
                .map(|res| (Addr::unchecked(String::from_utf8(res.0).unwrap()), res.1))
                .unwrap())
            .collect::<Vec<_>>(),
        vec![(a.clone(), 1)]
    );

    it.map.save(&mut storage, &b, &2).unwrap();

    assert_eq!(
        it.map
            .prefix(())
            .range(&storage, None, None, Order::Ascending)
            .map(|e| e
                .map(|res| (Addr::unchecked(String::from_utf8(res.0).unwrap()), res.1))
                .unwrap())
            .collect::<Vec<_>>(),
        vec![(a.clone(), 1), (b.clone(), 2)]
    );

    it.map.remove(&mut storage, &a);

    assert_eq!(
        it.map
            .prefix(())
            .range(&storage, None, None, Order::Ascending)
            .map(|e| e
                .map(|res| (Addr::unchecked(String::from_utf8(res.0).unwrap()), res.1))
                .unwrap())
            .collect::<Vec<_>>(),
        vec![(b, 2)]
    );
}

#[test]
fn indexed_map_works() {
    let mut storage = MockStorage::new();
    let it = ItemMapAccessor::new("primary");

    let a = Addr::unchecked("a");
    let b = Addr::unchecked("b");

    let first = ToIndex {
        id: 0,
        count: 5,
        address: a.clone(),
    };

    let second = ToIndex {
        id: 1,
        count: 5,
        address: b.clone(),
    };

    assert_eq!(
        it.indexed_map
            .prefix(())
            .range(&storage, None, None, Order::Ascending)
            .map(|e| e.unwrap().1)
            .collect::<Vec<_>>(),
        vec![]
    );

    it.indexed_map
        .save(&mut storage, first.id.into(), &first)
        .unwrap();

    assert_eq!(
        it.indexed_map
            .prefix(())
            .range(&storage, None, None, Order::Ascending)
            .map(|e| e.unwrap().1)
            .collect::<Vec<_>>(),
        vec![first.clone()]
    );

    it.indexed_map
        .save(&mut storage, second.id.into(), &second)
        .unwrap();

    assert_eq!(
        it.indexed_map
            .prefix(())
            .range(&storage, None, None, Order::Ascending)
            .map(|e| e.unwrap().1)
            .collect::<Vec<_>>(),
        vec![first.clone(), second.clone()]
    );

    assert_eq!(
        it.indexed_map
            .index
            .count
            .prefix(5.into())
            .range(&storage, None, None, Order::Ascending)
            .map(|e| e.unwrap().1)
            .collect::<Vec<_>>(),
        vec![first.clone(), second.clone()]
    );

    assert_eq!(
        it.indexed_map
            .index
            .count
            .prefix(5.into())
            .range(
                &storage,
                Some(Bound::exclusive_int(0u64)),
                None,
                Order::Ascending
            )
            .map(|e| e.unwrap().1)
            .collect::<Vec<_>>(),
        vec![second.clone()]
    );

    assert_eq!(
        it.indexed_map
            .index
            .count
            .prefix(5.into())
            .range(
                &storage,
                None,
                Some(Bound::exclusive_int(1u64)),
                Order::Ascending,
            )
            .map(|e| e.unwrap().1)
            .collect::<Vec<_>>(),
        vec![first.clone()]
    );

    assert_eq!(
        it.indexed_map
            .index
            .address
            .item(&storage, a.clone())
            .unwrap()
            .unwrap()
            .1,
        first.clone()
    );

    assert_eq!(
        it.indexed_map
            .index
            .address
            .item(&storage, b.clone())
            .unwrap()
            .unwrap()
            .1,
        second.clone()
    );

    it.indexed_map
        .remove(&mut storage, first.id.into())
        .unwrap();

    assert_eq!(
        it.indexed_map
            .prefix(())
            .range(&storage, None, None, Order::Ascending)
            .map(|e| e.unwrap().1)
            .collect::<Vec<_>>(),
        vec![second.clone()]
    );

    assert_eq!(
        it.indexed_map
            .index
            .count
            .prefix(5.into())
            .range(&storage, None, None, Order::Ascending)
            .map(|e| e.unwrap().1)
            .collect::<Vec<_>>(),
        vec![second.clone()]
    );

    assert_eq!(
        it.indexed_map
            .index
            .address
            .item(&storage, a.clone())
            .unwrap(),
        None
    );
}
