# Storage Extra

Additional feature implementation to access `cw-storage-plus` and `cosmwasm-storage`

## Cow

Cow implementation of storage accessor in `cw-storage-plus` to provide owned string variant of namespace, useful for constructing the accessor dynamically though `format!` macro.

Also implements new useful storage accessor, such as `ConditionalMultiIndex` and `CustomDeseMultiIndex`

Every struct will have access to `new_owned` and `new_ref` constant constructor function.

### ItemCow

Like `Item` from `cw-storage-plus` but in `Cow`.

```rust
let item: ItemCow<Addr> = ItemCow::new_owned(String::from("g"));
const ITEM: ItemCow<Addr> = ItemCow::new_ref("g");
```

### MapCow

Like `Map` from `cw-storage-plus` but in `Cow`.

```rust
let addr_owne: MapCow<&Addr, u64> = MapCow::new_owned(String::from("g"));
const ADDR_REF: MapCow<&Addr, u64> = MapCow::new_ref("g");
```

### IndexMapCow

Like `IndexedMap` from `cw-storage-plus` but in `Cow`. `Index` struct can be construct from normal `Index` trait, like `MultiIndex` and `UniqueIndex`.

### MultiIndexCow

Like `MultiIndex` from `cw-storage-plus` but in `Cow`. Also usable in normal `IndexedMap`.

### UniqueIndexCow

Like `UniqueIndex` from `cw-storage-plus` but in `Cow`. Also usable in normal `IndexedMap`.

```rust
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

const TO: IndexedMapCow<U64Key, ToIndex, ToIndexList> = IndexedMapCow::new_ref(
    "primary",
    ToIndexList {
        count: MultiIndexCow::new_ref("primary", "primary_count", |e, k| {
            (e.count.into(), k)
        }),
        address: UniqueIndexCow::new_ref("primary_address", |e| e.address.clone()),
    },
);

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
```

### CustomDeseMultiIndex

`MultiIndexCow` with customizable index to pk deserialize function. Also usable in normal `IndexedMap`.

Use `deserialize_multi_kv_custom_pk` helper function to map old kv to new kv, also works with a fully customed one.

Use `None` to operate as normal `MultiIndexCow`.

```rust
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, PartialOrd)]
struct Test {
    id: u64,
    val: Uint128,
}

struct TestIndexes<'a> {
    val: CustomDeseMultiIndex<'a, (U128Key, Vec<u8>), Test>,
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
        },
    )
}

```

### ConditionalMultiIndex

`CustomDeseMultiIndex` with addtional condition to save/remove from original indexed map. Useful for reducing composite key complexity. Also usable in normal `IndexedMap`.

`cond_fn` **must be constant**, otherwise might raise unexpected behavior.

```rust
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, PartialOrd)]
struct Test {
    id: u64,
    val: Uint128,
}

struct TestIndexes<'a> {
    val: ConditionalMultiIndex<'a, (U128Key, Vec<u8>), Test>,
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
        },
    )
}

```d

