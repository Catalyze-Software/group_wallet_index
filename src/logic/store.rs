use std::{collections::HashSet, convert::TryFrom};

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

use crate::{
    storage::{
        cell_api::CellStorage,
        multisig_storage::MultisigStorage,
        multisig_wasm_storage::MultisigWasmStorage,
        proxy_storage::ProxyCanisterStorage,
        spawn_status_storage::SpawnStatusStorage,
        storage_api::{StorageInsertableByKey, StorageQueryable, StorageUpdateable},
    },
    types::{
        error::Error, result::CanisterResult, spawn_status::SpawnStatus, wallet_data::WalletData,
    },
};

pub struct Store;

impl Store {
    pub fn get_cycles() -> u64 {
        ic_cdk::api::canister_balance()
    }

    pub fn get_wallets() -> Vec<(Principal, WalletData)> {
        MultisigStorage::get_all()
    }

    pub fn get_wallet(principal: Principal) -> CanisterResult<(Principal, WalletData)> {
        MultisigStorage::get(principal)
    }

    pub fn _test_add_wallet(principal: Principal) -> CanisterResult<(Principal, WalletData)> {
        MultisigStorage::insert_by_key(principal, WalletData::new(0, 0, 0))
    }

    pub async fn spawn_canister(cycles: Nat) -> CanisterResult<Principal> {
        let args = CreateCanisterArgument {
            settings: Some(CanisterSettings {
                controllers: Some(vec![id()]),
                compute_allocation: None,
                memory_allocation: None,
                freezing_threshold: None,
                reserved_cycles_limit: None,
                wasm_memory_limit: None,
                log_visibility: None,
            }),
        };

        create_canister(args, TryFrom::try_from(cycles.0).unwrap())
            .await
            .map(|(result,)| result.canister_id)
            .map_err(|(_, err)| Error::internal().add_message(err.as_str()))
    }

    pub async fn install_canister(
        canister_id: Principal,
        whitelist: Vec<Principal>,
        group_id: u64,
    ) -> CanisterResult<Principal> {
        let wallet_wasm = MultisigWasmStorage::get()?;

        let proxy = ProxyCanisterStorage::get()?;

        let args = InstallCodeArgument {
            mode: CanisterInstallMode::Install,
            canister_id,
            wasm_module: wallet_wasm.to_vec(),
            arg: Encode!(&caller(), &whitelist, &proxy, &group_id).unwrap(),
        };
        install_code(args)
            .await
            .map_err(|(_, err)| Error::internal().add_message(err.as_str()))
            .map(|_| canister_id)
    }

    pub fn save_wallet(
        canister_id: Principal,
        icp_transfer_blockheight: u64,
        cmc_transfer_block_height: u64,
        group_id: u64,
    ) -> CanisterResult<(Principal, WalletData)> {
        let wallet = WalletData::new(
            icp_transfer_blockheight,
            cmc_transfer_block_height,
            group_id,
        );

        MultisigStorage::insert_by_key(canister_id, wallet)
    }

    pub fn get_spawn(blockheight: u64) -> CanisterResult<(u64, SpawnStatus)> {
        SpawnStatusStorage::get(blockheight)
    }

    pub fn get_spawns() -> Vec<(u64, SpawnStatus)> {
        SpawnStatusStorage::get_all()
    }

    pub fn save_status(
        blockheight: u64,
        status: SpawnStatus,
    ) -> CanisterResult<(u64, SpawnStatus)> {
        SpawnStatusStorage::insert_by_key(blockheight, status)
    }

    pub fn update_status(
        blockheight: u64,
        status: SpawnStatus,
    ) -> CanisterResult<(u64, SpawnStatus)> {
        SpawnStatusStorage::update(blockheight, status)
    }

    pub fn validate_whitelist(whitelist: &[Principal]) -> CanisterResult<()> {
        if whitelist.len() < 2 {
            return Err(
                Error::bad_request().add_message("Whitelist must have at least 2 principals")
            );
        }

        // check for duplicate principals
        let mut seen = HashSet::new();

        for principal in whitelist.iter() {
            if seen.contains(principal) {
                return Err(Error::bad_request()
                    .add_message(format!("Duplicate principal: {}", principal).as_str()));
            }
            seen.insert(principal);
        }

        Ok(())
    }

    pub async fn transfer_ownership(
        canister_id: Principal,
        new_owner: Principal,
    ) -> CanisterResult<(Principal, WalletData)> {
        // Get the wallet
        let (_, mut wallet) = MultisigStorage::get(canister_id)?;

        // Check if the caller is the owner
        if !wallet.is_owner(caller()) {
            return Err(Error::unauthorized().add_message("Caller is not the owner"));
        }

        // Call the canister to set the new owner
        let call_result: Result<(Result<Principal, String>,), (RejectionCode, String)> =
            call::call(canister_id, "set_owner", (new_owner,)).await;

        // Check if the call was successful
        let wallet_canister_id = call_result
            .map_err(|(_, err)| Error::internal().add_message(err.as_str()))?
            .0
            .map_err(|err| Error::internal().add_message(err.as_str()))?;

        MultisigStorage::update(wallet_canister_id, wallet.set_owner(new_owner))
    }
}
