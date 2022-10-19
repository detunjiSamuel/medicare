use cosmwasm_std::testing::MockStorage;
use cw_storage_plus::Map;
use tw_storage_macros::StorageKey;

#[derive(Clone, Copy, StorageKey)]
enum TestEnum {
    G,
    F,
}

#[test]
fn compile_and_works() {
    let mut storage = MockStorage::new();

    let map: Map<TestEnum, u64> = Map::new("great!");
    let map_2: Map<&TestEnum, u64> = Map::new("good!");

    map.save(&mut storage, TestEnum::G, &4).unwrap();
    map_2.save(&mut storage, &TestEnum::F, &4).unwrap();

    assert_eq!(map.load(&storage, TestEnum::G).unwrap(), 4);
    assert_eq!(map_2.load(&storage, &TestEnum::F).unwrap(), 4);
}
