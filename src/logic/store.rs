use std::{cell::RefCell, convert::TryFrom};

use candid::{Encode, Nat, Principal};
use ic_cdk::{
    api::management_canister::{
        main::{
            create_canister, install_code, CanisterInstallMode, CreateCanisterArgument,
            InstallCodeArgument,
        },
        provisional::CanisterSettings,
    },
    id,
};
use ic_ledger_types::{Memo, Tokens};
use ic_stable_structures::{
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
    {DefaultMemoryImpl, StableBTreeMap},
};

use crate::rust_declarations::types::{MultisigData, SpawnStatus};

type Memory = VirtualMemory<DefaultMemoryImpl>;

pub static MEMO_TOP_UP_CANISTER: Memo = Memo(1347768404_u64);
pub static MEMO_CREATE_CANISTER: Memo = Memo(1095062083_u64);
pub static ICP_TRANSACTION_FEE: Tokens = Tokens::from_e8s(10000);
pub static MIN_E8S_FOR_SPINUP: Tokens = Tokens::from_e8s(110000000);
pub static CATALYZE_E8S_FEE: Tokens = Tokens::from_e8s(10000000);
pub static CATALYZE_MULTI_SIG: &str = "fcygz-gqaaa-aaaap-abpaa-cai";

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));


    pub static ENTRIES: RefCell<StableBTreeMap<Principal, MultisigData, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))),
        )
    );

    pub static SPAWN_STATUS: RefCell<StableBTreeMap<u64, SpawnStatus, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3))),
        )
    );
}

pub struct Store;

impl Store {
    pub fn get_cycles() -> u64 {
        ic_cdk::api::canister_balance()
    }

    pub fn get_multisigs() -> Vec<(Principal, MultisigData)> {
        ENTRIES.with(|e| e.borrow().iter().collect())
    }

    pub async fn spawn_canister(cycles: Nat) -> Result<Principal, String> {
        let args = CreateCanisterArgument {
            settings: Some(CanisterSettings {
                controllers: Some(vec![id()]),
                compute_allocation: None,
                memory_allocation: None,
                freezing_threshold: None,
                reserved_cycles_limit: None,
            }),
        };

        let result = create_canister(args, TryFrom::try_from(cycles.0).unwrap()).await;
        match result {
            Ok((canister_record,)) => Ok(canister_record.canister_id),
            Err((_, err)) => Err(err),
        }
    }

    pub async fn install_canister(
        canister_id: Principal,
        whitelist: Vec<Principal>,
    ) -> Result<Principal, String> {
        let multisig_wasm = include_bytes!("../../wasm/multisig.wasm.gz");

        let args = InstallCodeArgument {
            mode: CanisterInstallMode::Install,
            canister_id,
            wasm_module: multisig_wasm.to_vec(),
            arg: Encode!((&whitelist)).unwrap(),
        };
        let result = install_code(args).await;

        match result {
            Ok(()) => Ok(canister_id),
            Err((_, err)) => Err(err),
        }
    }

    pub fn save_multisig(
        canister_id: Principal,
        icp_transfer_blockheight: u64,
        cmc_transfer_block_height: u64,
    ) {
        let multisig = MultisigData::new(icp_transfer_blockheight, cmc_transfer_block_height);
        ENTRIES.with(|e| e.borrow_mut().insert(canister_id, multisig));
    }

    pub fn get_spawn(blockheight: u64) -> Result<SpawnStatus, String> {
        SPAWN_STATUS.with(|s| {
            s.borrow()
                .get(&blockheight)
                .ok_or_else(|| "Error: blockheight not found".to_string())
        })
    }

    pub fn get_spawns() -> Vec<(u64, SpawnStatus)> {
        SPAWN_STATUS.with(|s| s.borrow().iter().collect())
    }

    pub fn save_spawn_status(block_index: u64, status: SpawnStatus) {
        SPAWN_STATUS.with(|s| s.borrow_mut().insert(block_index, status));
    }
}
