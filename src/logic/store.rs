use std::{cell::RefCell, collections::HashSet, convert::TryFrom};

use candid::{Encode, Nat, Principal};
use ic_cdk::{
    api::{
        call::{self, RejectionCode},
        management_canister::{
            main::{
                create_canister, install_code, CanisterInstallMode, CreateCanisterArgument,
                InstallCodeArgument,
            },
            provisional::CanisterSettings,
        },
    },
    caller, id,
};
use ic_ledger_types::{Memo, Tokens};
use ic_stable_structures::{
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
    {DefaultMemoryImpl, StableBTreeMap},
};

use crate::rust_declarations::types::{Status, WalletData};

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


    pub static ENTRIES: RefCell<StableBTreeMap<Principal, WalletData, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))),
        )
    );

    pub static SPAWN_STATUS: RefCell<StableBTreeMap<u64, Status, Memory>> = RefCell::new(
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

    pub fn get_wallets() -> Vec<(Principal, WalletData)> {
        ENTRIES.with(|e| e.borrow().iter().collect())
    }

    pub fn get_wallet(principal: &Principal) -> Option<WalletData> {
        ENTRIES.with(|e| e.borrow().get(principal).clone())
    }

    pub async fn spawn_canister(cycles: Nat) -> Result<Principal, String> {
        let args = CreateCanisterArgument {
            settings: Some(CanisterSettings {
                controllers: Some(vec![id()]),
                compute_allocation: None,
                memory_allocation: None,
                freezing_threshold: None,
                reserved_cycles_limit: None,
                wasm_memory_limit: None,
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
        let wallet_wasm = include_bytes!("../../wasm/wallet.wasm.gz");

        let args = InstallCodeArgument {
            mode: CanisterInstallMode::Install,
            canister_id,
            wasm_module: wallet_wasm.to_vec(),
            arg: Encode!((&whitelist)).unwrap(),
        };
        let result = install_code(args).await;

        match result {
            Ok(()) => Ok(canister_id),
            Err((_, err)) => Err(err),
        }
    }

    pub fn save_wallet(
        canister_id: Principal,
        icp_transfer_blockheight: u64,
        cmc_transfer_block_height: u64,
    ) {
        let wallet = WalletData::new(icp_transfer_blockheight, cmc_transfer_block_height);
        ENTRIES.with(|e| e.borrow_mut().insert(canister_id, wallet));
    }

    pub fn get_spawn(blockheight: u64) -> Result<Status, String> {
        SPAWN_STATUS.with(|s| {
            s.borrow()
                .get(&blockheight)
                .ok_or_else(|| "Error: blockheight not found".to_string())
        })
    }

    pub fn get_spawns() -> Vec<(u64, Status)> {
        SPAWN_STATUS.with(|s| s.borrow().iter().collect())
    }

    pub fn save_status(block_index: u64, status: Status) {
        SPAWN_STATUS.with(|s| s.borrow_mut().insert(block_index, status));
    }

    pub fn validate_whitelist(whitelist: &Vec<Principal>) -> Result<(), String> {
        if whitelist.len() < 2 {
            return Err("Whitelist must have at least 2 principals".to_string());
        }

        // check for duplicate principals
        let mut seen = HashSet::new();

        for principal in whitelist.iter() {
            if seen.contains(principal) {
                return Err(format!("Duplicate principal: {}", principal));
            }
            seen.insert(principal);
        }

        Ok(())
    }

    pub async fn transfer_ownership(
        canister_id: Principal,
        new_owner: Principal,
    ) -> Result<(), String> {
        // Get the wallet
        let mut wallet = ENTRIES.with(|e| {
            e.borrow()
                .get(&canister_id)
                .ok_or_else(|| format!("Error: Canister not found: {}", canister_id))
        })?;

        // Check if the caller is the owner
        if !wallet.is_owner(caller()) {
            return Err("Error: Caller is not the owner".to_string());
        }

        // Call the canister to set the new owner
        let call_result: Result<(Result<Principal, String>,), (RejectionCode, String)> =
            call::call(canister_id, "set_owner", (new_owner,)).await;

        // Check if the call was successful
        let call_result = call_result.map_err(|(_, err)| err)?.0;

        // get the wallet canister id from the result
        let wallet_canister_id = call_result?;

        ENTRIES.with(|e| {
            e.borrow_mut()
                .insert(wallet_canister_id, wallet.set_owner(new_owner));
        });

        Ok(())
    }
}
