use candid::Principal;
use ic_stable_structures::memory_manager::MemoryId;

use super::{
    cell_api::{CellStorage, CellStorageRef},
    state::{PROXY_CANISTER, PROXY_CANISTER_MEMORY_ID},
};

pub struct ProxyCanisterStorage;

impl CellStorage<Principal> for ProxyCanisterStorage {
    const NAME: &'static str = "proxy";

    fn storage() -> CellStorageRef<Principal> {
        &PROXY_CANISTER
    }

    fn memory_id() -> MemoryId {
        PROXY_CANISTER_MEMORY_ID
    }
}
