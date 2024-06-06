use std::{cell::RefCell, thread::LocalKey};

use candid::Principal;
use ic_ledger_types::{Memo, Tokens};
use ic_stable_structures::{
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
    Cell, DefaultMemoryImpl, StableBTreeMap,
};

use crate::types::{spawn_status::SpawnStatus, wallet_data::WalletData};

pub type Memory = VirtualMemory<DefaultMemoryImpl>;
pub type StorageRef<K, V> = RefCell<StableBTreeMap<K, V, Memory>>;
pub type StaticStorageRef<K, V> = &'static LocalKey<StorageRef<K, V>>;

type MemoryManagerStorage = RefCell<MemoryManager<DefaultMemoryImpl>>;

pub static MEMO_TOP_UP_CANISTER: Memo = Memo(1347768404_u64);
pub static MEMO_CREATE_CANISTER: Memo = Memo(1095062083_u64);
pub static ICP_TRANSACTION_FEE: Tokens = Tokens::from_e8s(10000);
pub static MIN_CYCLES_FOR_SPINUP: u64 = 5_000_000_000_000;
pub static CATALYZE_E8S_FEE: Tokens = Tokens::from_e8s(10000000);
pub static CATALYZE_MULTI_SIG: &str = "fcygz-gqaaa-aaaap-abpaa-cai";

pub static MULTISIGS_MEMORY_ID: MemoryId = MemoryId::new(0);
pub static SPAWN_STATUS_MEMORY_ID: MemoryId = MemoryId::new(1);
pub static PROXY_CANISTER_MEMORY_ID: MemoryId = MemoryId::new(2);
pub static MULTISIG_WASM_MEMORY_ID: MemoryId = MemoryId::new(3);

thread_local! {
    pub static MEMORY_MANAGER: MemoryManagerStorage =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));


    pub static MULTISIGS: RefCell<StableBTreeMap<Principal, WalletData, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MULTISIGS_MEMORY_ID)),
        )
    );

    pub static SPAWN_STATUS: RefCell<StableBTreeMap<u64, SpawnStatus, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(SPAWN_STATUS_MEMORY_ID)),
        )
    );

    pub static PROXY_CANISTER: RefCell<Cell<Option<Principal>, Memory>> = RefCell::new(
        Cell::init(MEMORY_MANAGER.with(|p| p.borrow().get(PROXY_CANISTER_MEMORY_ID)), None)
            .expect("Failed to initialize proxy canister")
    );

    pub static MULTISIG_WASM: RefCell<Cell<Option<Vec<u8>>, Memory>> = RefCell::new(
        Cell::init(MEMORY_MANAGER.with(|p| p.borrow().get(MULTISIG_WASM_MEMORY_ID)), None)
            .expect("Failed to initialize proxy canister")
    );
}
