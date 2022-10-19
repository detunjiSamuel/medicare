use cosmwasm_std::{testing::MockStorage, Addr};
use cw_storage_plus::{IndexedMap, MultiIndex, UniqueIndex};
use serde::{Deserialize, Serialize};
use tw_storage_macros::index_list_impl;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
struct TestStruct {
    id: u64,
    id2: u32,
    addr: Addr,
}

#[index_list_impl(TestStruct)]
struct TestIndexes<'a> {
    id: MultiIndex<'a, u32, TestStruct, u64>,
    addr: UniqueIndex<'a, Addr, TestStruct>,
}

#[test]
fn compile() {
    let _: IndexedMap<u64, TestStruct, TestIndexes> = IndexedMap::new(
        "t",
        TestIndexes {
            id: MultiIndex::new(|t| t.id2, "t", "t_2"),
            addr: UniqueIndex::new(|t| t.addr.clone(), "t_addr"),
        },
    );
}

#[test]
fn works() {
    let mut storage = MockStorage::new();
    let idm: IndexedMap<u64, TestStruct, TestIndexes> = IndexedMap::new(
        "t",
        TestIndexes {
            id: MultiIndex::new(|t| t.id2, "t", "t_2"),
            addr: UniqueIndex::new(|t| t.addr.clone(), "t_addr"),
        },
    );

    idm.save(
        &mut storage,
        0,
        &TestStruct {
            id: 0,
            id2: 100,
            addr: Addr::unchecked("1"),
        },
    )
    .unwrap();

    assert_eq!(
        idm.load(&storage, 0).unwrap(),
        TestStruct {
            id: 0,
            id2: 100,
            addr: Addr::unchecked("1"),
        }
    );
}
