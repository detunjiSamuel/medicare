use cosmwasm_std::{StdError, StdResult, Storage};
use cw_storage_plus::Item;
use serde::{de::DeserializeOwned, Serialize};
use std::{borrow::Cow, marker::PhantomData};

#[derive(Debug, Clone)]
pub struct ItemCow<'a, T> {
    pub(crate) namespace: Cow<'a, str>,
    data_type: PhantomData<T>,
}

impl<'a, 'k, T> ItemCow<'a, T>
where
    'k: 'a,
{
    pub const fn new_owned(namespace: String) -> Self {
        Self {
            namespace: Cow::Owned(namespace),
            data_type: PhantomData,
        }
    }

    pub const fn new_ref(namespace: &'k str) -> Self {
        Self {
            namespace: Cow::Borrowed(namespace),
            data_type: PhantomData,
        }
    }
}

impl<'a, T> ItemCow<'a, T>
where
    T: Serialize + DeserializeOwned,
{
    pub fn item(&self) -> Item<T> {
        Item::new(&self.namespace)
    }

    pub fn save(&self, store: &mut dyn Storage, data: &T) -> StdResult<()> {
        self.item().save(store, data)
    }

    pub fn remove(&self, store: &mut dyn Storage) {
        self.item().remove(store)
    }

    pub fn load(&self, store: &dyn Storage) -> StdResult<T> {
        self.item().load(store)
    }

    pub fn may_load(&self, store: &dyn Storage) -> StdResult<Option<T>> {
        self.item().may_load(store)
    }

    pub fn update<A, E>(&self, store: &mut dyn Storage, action: A) -> Result<T, E>
    where
        A: FnOnce(T) -> Result<T, E>,
        E: From<StdError>,
    {
        self.item().update(store, action)
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
        let item: ItemCow<Addr> = ItemCow::new_owned(String::from("Something brah"));

        item.item().save(&mut storage, &a).unwrap();

        assert_eq!(item.item().load(&storage).unwrap(), a);
    }

    #[test]
    fn new_ref() {
        let a = Addr::unchecked("g");
        let mut storage = MockStorage::new();
        const ITEM: ItemCow<Addr> = ItemCow::new_ref("Anotherthing");

        ITEM.item().save(&mut storage, &a).unwrap();

        assert_eq!(ITEM.item().load(&storage).unwrap(), a);
    }
}
