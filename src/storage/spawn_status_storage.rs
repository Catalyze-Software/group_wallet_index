use ic_stable_structures::memory_manager::MemoryId;

use crate::types::spawn_status::SpawnStatus;

use super::{
    state::{StaticStorageRef, SPAWN_STATUS, SPAWN_STATUS_MEMORY_ID},
    storage_api::{Storage, StorageInsertableByKey, StorageQueryable, StorageUpdateable},
};

pub struct SpawnStatusStorage;

impl Storage<u64, SpawnStatus> for SpawnStatusStorage {
    const NAME: &'static str = "spawn_status";

    fn storage() -> StaticStorageRef<u64, SpawnStatus> {
        &SPAWN_STATUS
    }

    fn memory_id() -> MemoryId {
        SPAWN_STATUS_MEMORY_ID
    }
}

impl StorageQueryable<u64, SpawnStatus> for SpawnStatusStorage {}
impl StorageInsertableByKey<u64, SpawnStatus> for SpawnStatusStorage {}
impl StorageUpdateable<u64, SpawnStatus> for SpawnStatusStorage {}
