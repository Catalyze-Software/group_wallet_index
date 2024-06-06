use candid::Principal;
use ic_stable_structures::memory_manager::MemoryId;

use crate::types::wallet_data::WalletData;

use super::{
    state::{StaticStorageRef, MULTISIGS, MULTISIGS_MEMORY_ID},
    storage_api::{Storage, StorageInsertableByKey, StorageQueryable, StorageUpdateable},
};

pub struct MultisigStorage;

impl Storage<Principal, WalletData> for MultisigStorage {
    const NAME: &'static str = "multisigs";

    fn storage() -> StaticStorageRef<Principal, WalletData> {
        &MULTISIGS
    }

    fn memory_id() -> MemoryId {
        MULTISIGS_MEMORY_ID
    }
}

impl StorageQueryable<Principal, WalletData> for MultisigStorage {}
impl StorageInsertableByKey<Principal, WalletData> for MultisigStorage {}
impl StorageUpdateable<Principal, WalletData> for MultisigStorage {}
