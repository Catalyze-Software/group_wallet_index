use candid::Principal;
use ic_cdk::{caller, id, query, update};

use crate::{
    logic::{
        cmc::CyclesManagementCanister,
        ledger::Ledger,
        store::{Store, MIN_E8S_FOR_SPINUP},
    },
    rust_declarations::types::{Status, WalletData},
};

#[query]
fn get_cycles() -> u64 {
    Store::get_cycles()
}

#[query]
fn get_spawns() -> Vec<(u64, Status)> {
    Store::get_spawns()
}

#[query]
fn get_spawn(blockheight: u64) -> Result<Status, String> {
    Store::get_spawn(blockheight)
}

#[query]
fn get_wallets() -> Vec<(Principal, WalletData)> {
    Store::get_wallets()
}

#[update(guard = "is_not_anonymous")]
async fn spawn_wallet(
    icp_transfer_blockheight: u64,
    whitelist: Vec<Principal>,
) -> Result<Principal, String> {
    Store::validate_whitelist(&whitelist)?;

    // check if spawn already exists
    if Store::get_spawn(icp_transfer_blockheight).is_ok() {
        return Err(format!(
            "Duplicate blockheight: {}",
            icp_transfer_blockheight
        ));
    }

    // initialize new spawn status tracker
    let mut spawn_status = Status::new(Some("Wallet spawn".to_string()));
    Store::save_status(icp_transfer_blockheight, spawn_status.clone());

    // validate ICP transaction
    let amount = Ledger::validate_transaction(caller(), icp_transfer_blockheight).await?;

    Store::save_status(
        icp_transfer_blockheight,
        spawn_status.transaction_valid(amount),
    );

    // if amount is less than minimum required, transfer ICP back to caller
    if amount < MIN_E8S_FOR_SPINUP {
        let transfer_back_blockheight = Ledger::transfer_icp_back_to_caller(amount).await?;

        Store::save_status(
            icp_transfer_blockheight,
            spawn_status.min_amount_error(transfer_back_blockheight),
        );

        return Err(format!(
            "Amount ({}) is less than {}, ICP transferred back: blockheight: {}",
            amount, MIN_E8S_FOR_SPINUP, transfer_back_blockheight
        ));
    }

    // transfer ICP to the cycles management canister
    let cmc_transfer_block_height = Ledger::transfer_icp_to_cmc(amount, id()).await?;

    Store::save_status(
        icp_transfer_blockheight,
        spawn_status.transferred_to_cmc(cmc_transfer_block_height),
    );

    // top up this canister with cycles
    let cycles = CyclesManagementCanister::top_up(cmc_transfer_block_height, id()).await?;

    Store::save_status(
        icp_transfer_blockheight,
        spawn_status.topped_up_self(cycles.clone()),
    );

    // spawn a new canister
    let canister_id = Store::spawn_canister(cycles).await?;

    Store::save_status(
        icp_transfer_blockheight,
        spawn_status.canister_spawned(canister_id),
    );

    // install the wallet canister
    let installed_canister_principal = Store::install_canister(canister_id, whitelist).await?;

    Store::save_status(
        icp_transfer_blockheight,
        spawn_status.canister_installed(installed_canister_principal),
    );

    // save the wallet data
    Store::save_wallet(
        installed_canister_principal,
        icp_transfer_blockheight,
        cmc_transfer_block_height,
    );

    Store::save_status(icp_transfer_blockheight, spawn_status.done());

    Ok(installed_canister_principal)
}

#[update(guard = "is_not_anonymous")]
async fn top_up_wallet(
    icp_transfer_blockheight: u64,
    wallet_principal: Principal,
) -> Result<(), String> {
    // check if spawn already exists
    if Store::get_spawn(icp_transfer_blockheight).is_ok() {
        return Err(format!(
            "Duplicate blockheight: {}",
            icp_transfer_blockheight
        ));
    }

    // initialize new status tracker
    let mut spawn_status = Status::new(Some("Top up wallet".to_string()));
    Store::save_status(icp_transfer_blockheight, spawn_status.clone());

    // validate ICP transaction
    let amount = Ledger::validate_transaction(caller(), icp_transfer_blockheight).await?;

    Store::save_status(
        icp_transfer_blockheight,
        spawn_status.transaction_valid(amount),
    );

    // transfer ICP to the cycles management canister
    let cmc_transfer_block_height = Ledger::transfer_icp_to_cmc(amount, wallet_principal).await?;

    Store::save_status(
        icp_transfer_blockheight,
        spawn_status.transferred_to_cmc(cmc_transfer_block_height),
    );

    // top up this canister with cycles
    let cycles =
        CyclesManagementCanister::top_up(cmc_transfer_block_height, wallet_principal).await?;

    Store::save_status(
        icp_transfer_blockheight,
        spawn_status.topped_up_self(cycles.clone()),
    );

    Store::save_status(icp_transfer_blockheight, spawn_status.done());

    Ok(())
}

#[update(guard = "is_not_anonymous")]
async fn transfer_ownership(canister_id: Principal, new_owner: Principal) -> Result<(), String> {
    Store::transfer_ownership(canister_id, new_owner).await
}

#[update(guard = "is_not_anonymous")]
fn _dev_add_wallet(canister_id: Principal) -> Option<WalletData> {
    if caller()
        != Principal::from_text("syzio-xu6ca-burmx-4afo2-ojpcw-e75j3-m67o5-s5bes-5vvsv-du3t4-wae")
            .unwrap()
    {
        return None;
    }
    Store::add_wallet(canister_id)
}

#[query]
pub fn __get_candid_interface_tmp_hack() -> String {
    use candid::export_service;

    use crate::rust_declarations::types::*;
    export_service!();
    __export_service()
}

#[test]
pub fn candid() {
    use std::env;
    use std::fs::write;
    use std::path::PathBuf;
    let dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let dir = dir.parent().unwrap().join("candid");
    write(
        dir.join("wallet_index.did"),
        __get_candid_interface_tmp_hack(),
    )
    .expect("Write failed.");
}

pub fn is_not_anonymous() -> Result<(), String> {
    match caller() == Principal::anonymous() {
        true => Err("Anonymous principal".to_string()),
        false => Ok(()),
    }
}
