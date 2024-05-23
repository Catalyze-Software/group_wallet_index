use candid::Principal;
use ic_cdk::{caller, id, query, update};

use crate::{
    logic::{
        cmc::CyclesManagementCanister,
        ledger::Ledger,
        store::{Store, MIN_E8S_FOR_SPINUP},
    },
    rust_declarations::types::{MultisigData, SpawnStatus},
};

#[query]
fn get_cycles() -> u64 {
    Store::get_cycles()
}

#[update]
async fn get_cmc_icp_balance() -> Result<u64, String> {
    Store::get_icp_balance(id()).await
}

#[query]
fn get_multisigs() -> Vec<MultisigData> {
    Store::get_multisigs()
}

#[update]
async fn spawn_multisig(
    icp_transfer_blockheight: u64,
    whitelist: Vec<Principal>,
) -> Result<Principal, String> {
    // check if spawn already exists
    Store::check_spawn_exists(icp_transfer_blockheight)?;

    // initialize new spawn status tracker
    let mut spawn_status = SpawnStatus::new();

    // validate ICP transaction
    let amount = Ledger::validate_transaction(caller(), icp_transfer_blockheight).await?;

    Store::save_spawn_status(
        icp_transfer_blockheight,
        spawn_status.transaction_valid(amount),
    );

    // if amount is less than minimum required, transfer ICP back to caller
    if amount < MIN_E8S_FOR_SPINUP {
        let transfer_back_blockheight = Ledger::transfer_icp_back_to_caller(amount).await?;

        Store::save_spawn_status(
            icp_transfer_blockheight,
            spawn_status.min_amount_error(transfer_back_blockheight),
        );

        return Err(format!(
            "Amount ({}) is less than {}, ICP transferred back: blockheight: {}",
            amount, MIN_E8S_FOR_SPINUP, transfer_back_blockheight
        ));
    }

    // transfer ICP to the cycles management canister
    let cmc_transfer_block_height = Ledger::transfer_icp_to_cmc(amount).await?;

    Store::save_spawn_status(
        icp_transfer_blockheight,
        spawn_status.transferred_to_cmc(cmc_transfer_block_height),
    );

    // top up this canister with cycles
    let cycles = CyclesManagementCanister::top_up_self(cmc_transfer_block_height).await?;

    Store::save_spawn_status(
        icp_transfer_blockheight,
        spawn_status.topped_up_self(cycles.clone()),
    );

    // spawn a new canister
    let canister_id = Store::spawn_canister(cycles).await?;

    Store::save_spawn_status(
        icp_transfer_blockheight,
        spawn_status.canister_spawned(canister_id),
    );

    // install the multisig canister
    let installed_canister_principal = Store::install_canister(canister_id, whitelist).await?;

    Store::save_spawn_status(
        icp_transfer_blockheight,
        spawn_status.canister_installed(installed_canister_principal),
    );

    // save the multisig data
    Store::save_multisig(
        installed_canister_principal,
        icp_transfer_blockheight,
        cmc_transfer_block_height,
    );

    Store::save_spawn_status(icp_transfer_blockheight, spawn_status.done());

    Ok(installed_canister_principal)
}

#[test]
pub fn candid() {
    use candid::export_service;
    use std::env;
    use std::fs::write;
    use std::path::PathBuf;
    export_service!();
    let dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let dir = dir.parent().unwrap().join("candid");
    write(dir.join("multisig_index.did"), __export_service()).expect("Write failed.");
}
