use ic_stable_structures::{memory_manager::MemoryId, StableBTreeMap, Storable};

use crate::types::error::Error;

use super::state::{StaticStorageRef, MEMORY_MANAGER};

pub trait Storage<K: Storable + Ord + Clone, V: Storable + Clone> {
    const NAME: &'static str;

    fn memory_id() -> MemoryId;
    fn storage() -> StaticStorageRef<K, V>;
}

pub trait StorageQueryable<K: 'static + Storable + Ord + Clone, V: 'static + Storable + Clone>:
    Storage<K, V>
{
    /// Get a single entity by key
    /// # Arguments
    /// * `key` - The key of the entity to get
    /// # Returns
    /// * `Result<(K, V), ApiError>` - The entity if found, otherwise an error
    fn get(key: K) -> Result<(K, V), Error> {
        Self::storage().with(|data| {
            data.borrow()
                .get(&key)
                .ok_or(
                    Error::not_found()
                        .add_method_name("get")
                        .add_info(Self::NAME),
                )
                .map(|value| (key, value))
        })
    }

    fn get_opt(key: K) -> Option<(K, V)> {
        Self::storage().with(|data| data.borrow().get(&key).map(|value| (key, value)))
    }

    /// Get multiple entities by key
    /// # Arguments
    /// * `keys` - The keys of the entities to get
    /// # Returns
    /// * `Vec<(K, V)>` - The entities if found, otherwise an empty vector
    fn get_many(keys: Vec<K>) -> Vec<(K, V)> {
        Self::storage().with(|data| {
            let mut entities = Vec::new();
            for key in keys {
                if let Some(value) = data.borrow().get(&key) {
                    entities.push((key, value));
                }
            }
            entities
        })
    }

    /// Get all entities by key
    /// # Returns
    /// * `Vec<(K, V)>` - The entities if found, otherwise an empty vector
    fn get_all() -> Vec<(K, V)> {
        Self::storage().with(|data| data.borrow().iter().collect())
    }

    /// Find a single entity by filter
    /// # Arguments
    /// * `filter` - The filter to apply
    /// # Returns
    /// * `Option<(K, V)>` - The entity if found, otherwise None
    fn find<F>(filter: F) -> Option<(K, V)>
    where
        F: Fn(&K, &V) -> bool,
    {
        Self::storage().with(|data| data.borrow().iter().find(|(id, value)| filter(id, value)))
    }

    /// Find all entities by filter
    /// # Arguments
    /// * `filter` - The filter to apply
    /// # Returns
    /// * `Vec<(K, V)>` - The entities if found, otherwise an empty vector
    fn filter<F>(filter: F) -> Vec<(K, V)>
    where
        F: Fn(&K, &V) -> bool,
    {
        Self::storage().with(|data| {
            data.borrow()
                .iter()
                .filter(|(id, value)| filter(id, value))
                .collect()
        })
    }

    /// Check if an entity exists by key
    /// # Arguments
    /// * `key` - The key of the entity to check
    /// # Returns
    /// * `bool` - True if the entity exists, otherwise false
    fn contains_key(key: K) -> bool {
        Self::storage().with(|data| data.borrow().contains_key(&key))
    }
}

pub trait StorageInsertable<V: 'static + Storable + Clone>: Storage<u64, V> {
    /// Insert a single entity
    /// # Arguments
    /// * `value` - The entity to insert
    /// # Returns
    /// * `Result<(u64, V), ApiError>` - The inserted entity if successful, otherwise an error
    /// # Note
    /// Does check if a entity with the same key already exists, if so returns an error
    fn insert(value: V) -> Result<(u64, V), Error> {
        Self::storage().with(|data| {
            let key = data
                .borrow()
                .last_key_value()
                .map(|(k, _)| k + 1)
                .unwrap_or_else(|| 1);

            if data.borrow().contains_key(&key) {
                return Err(Error::duplicate()
                    .add_method_name("insert")
                    .add_info(Self::NAME)
                    .add_message("Key already exists"));
            }

            data.borrow_mut().insert(key, value.clone());
            Ok((key, value))
        })
    }
}

pub trait StorageInsertableByKey<K: 'static + Storable + Ord + Clone, V: 'static + Storable + Clone>:
    Storage<K, V>
{
    /// Insert a single entity by key
    /// # Arguments
    /// * `key` - The entity as key of the entity to insert
    /// * `value` - The entity to insert
    /// # Returns
    /// * `Result<(K, V), ApiError>` - The inserted entity if successful, otherwise an error
    /// # Note
    /// Does check if a entity with the same key already exists, if so returns an error
    fn insert_by_key(key: K, value: V) -> Result<(K, V), Error> {
        Self::storage().with(|data| {
            if data.borrow().contains_key(&key) {
                return Err(Error::duplicate()
                    .add_method_name("insert_by_key")
                    .add_info(Self::NAME)
                    .add_message("Key already exists"));
            }

            data.borrow_mut().insert(key.clone(), value.clone());
            Ok((key, value))
        })
    }
}

pub trait StorageUpdateable<K: 'static + Storable + Ord + Clone, V: 'static + Storable + Clone>:
    Storage<K, V> + StorageQueryable<K, V>
{
    /// Update a single entity by key
    /// # Arguments
    /// * `key` - The key of the entity to update
    /// * `value` - The entity to update
    /// # Returns
    /// * `Result<(K, V), ApiError>` - The updated entity if successful, otherwise an error
    /// # Note
    /// Does check if a entity with the same key already exists, if not returns an error
    fn update(key: K, value: V) -> Result<(K, V), Error> {
        Self::storage().with(|data| {
            if !data.borrow().contains_key(&key) {
                return Err(Error::not_found()
                    .add_method_name("update")
                    .add_info(Self::NAME)
                    .add_message("Key does not exist"));
            }

            data.borrow_mut().insert(key.clone(), value.clone());
            Ok((key, value))
        })
    }

    fn upsert(key: K, value: V) -> Result<(K, V), Error> {
        Self::storage().with(|data| {
            data.borrow_mut().insert(key.clone(), value.clone());
            Ok((key, value))
        })
    }

    /// Remove a single entity by key
    /// # Arguments
    /// * `key` - The key of the entity to remove
    /// # Returns
    /// * `bool` - True if the entity was removed, otherwise false
    fn remove(key: K) -> Result<(), Error> {
        Self::storage().with(|data| {
            if !data.borrow().contains_key(&key) {
                return Err(Error::not_found()
                    .add_method_name("remove")
                    .add_info(Self::NAME)
                    .add_message("Key does not exist"));
            }
            data.borrow_mut().remove(&key);
            Ok(())
        })
    }

    /// Remove a entities by keys
    /// # Arguments
    /// * `keys` - The keys of the entities to remove
    fn remove_many(keys: Vec<K>) {
        Self::storage().with(|data| {
            for key in keys {
                data.borrow_mut().remove(&key);
            }
        })
    }

    /// Clear all entities
    fn clear() {
        Self::storage().with(|n| {
            n.replace(StableBTreeMap::new(
                MEMORY_MANAGER.with(|m| m.borrow().get(Self::memory_id())),
            ))
        });
    }
}
