use candid::Principal;
use ic_cdk::{caller, id, query, update};
use ic_ledger_types::Tokens;

use crate::{
    logic::{
        cmc::CyclesManagement,
        guards::{is_not_anonymous, is_prod_developer},
        ledger::Ledger,
        store::Store,
    },
    storage::{
        cell_api::CellStorage, multisig_wasm_storage::MultisigWasmStorage,
        proxy_storage::ProxyCanisterStorage,
    },
    types::{
        error::Error, result::CanisterResult, spawn_status::SpawnStatus, wallet_data::WalletData,
    },
};

#[query]
fn get_cycles() -> u64 {
    Store::get_cycles()
}

#[query]
fn get_spawns() -> Vec<(u64, SpawnStatus)> {
    Store::get_spawns()
}

#[query]
fn get_spawn(blockheight: u64) -> CanisterResult<(u64, SpawnStatus)> {
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
    group_id: u64,
) -> CanisterResult<Principal> {
    Store::validate_whitelist(&whitelist)?;

    // check if spawn already exists
    if Store::get_spawn(icp_transfer_blockheight).is_ok() {
        return Err(Error::bad_request()
            .add_message(format!("Duplicate blockheight: {}", icp_transfer_blockheight).as_str()));
    }

    // initialize new spawn status tracker
    let mut spawn_status = SpawnStatus::new(Some("Wallet spawn".to_string()));
    Store::save_status(icp_transfer_blockheight, spawn_status.clone())?;

    // validate ICP transaction
    let amount = Ledger::validate_transaction(caller(), icp_transfer_blockheight).await?;

    Store::update_status(
        icp_transfer_blockheight,
        spawn_status.transaction_valid(amount),
    )?;

    let minimum_spawn_icp_amount = CyclesManagement::get_minimum_spawn_icp_amount().await?;

    // if amount is less than minimum required, transfer ICP back to caller
    if amount < minimum_spawn_icp_amount {
        let transfer_back_blockheight = Ledger::transfer_icp_back_to_caller(amount).await?;

        Store::update_status(
            icp_transfer_blockheight,
            spawn_status.min_amount_error(transfer_back_blockheight),
        )?;

        return Err(Error::insufficient_balance().add_message(
            format!(
                "Amount ({}) is less than {}, ICP transferred back: blockheight: {}",
                amount,
                minimum_spawn_icp_amount.e8s(),
                transfer_back_blockheight
            )
            .as_str(),
        ));
    }

    // transfer ICP to the cycles management canister
    let cmc_transfer_block_height = Ledger::transfer_icp_to_cmc(amount, id()).await?;

    Store::update_status(
        icp_transfer_blockheight,
        spawn_status.transferred_to_cmc(cmc_transfer_block_height),
    )?;

    // top up this canister with cycles
    let cycles = CyclesManagement::top_up(cmc_transfer_block_height, id()).await?;

    Store::update_status(
        icp_transfer_blockheight,
        spawn_status.topped_up_self(cycles.clone()),
    )?;

    // spawn a new canister
    let canister_id = Store::spawn_canister(cycles).await?;

    Store::update_status(
        icp_transfer_blockheight,
        spawn_status.canister_spawned(canister_id),
    )?;

    // install the wallet canister
    let installed_canister_principal =
        Store::install_canister(canister_id, whitelist, group_id).await?;

    Store::update_status(
        icp_transfer_blockheight,
        spawn_status.canister_installed(installed_canister_principal),
    )?;

    // save the wallet data
    Store::save_wallet(
        installed_canister_principal,
        icp_transfer_blockheight,
        cmc_transfer_block_height,
        group_id,
    )?;

    Store::update_status(icp_transfer_blockheight, spawn_status.done())?;

    Ok(installed_canister_principal)
}

#[update(guard = "is_not_anonymous")]
async fn top_up_wallet(
    icp_transfer_blockheight: u64,
    wallet_principal: Principal,
) -> CanisterResult<()> {
    // check if spawn already exists
    if Store::get_spawn(icp_transfer_blockheight).is_ok() {
        return Err(Error::bad_request()
            .add_message(format!("Duplicate blockheight: {}", icp_transfer_blockheight).as_str()));
    }

    // initialize new status tracker
    let mut spawn_status = SpawnStatus::new(Some("Top up wallet".to_string()));
    Store::save_status(icp_transfer_blockheight, spawn_status.clone())?;

    // validate ICP transaction
    let amount = Ledger::validate_transaction(caller(), icp_transfer_blockheight).await?;

    Store::update_status(
        icp_transfer_blockheight,
        spawn_status.transaction_valid(amount),
    )?;

    // transfer ICP to the cycles management canister
    let cmc_transfer_block_height = Ledger::transfer_icp_to_cmc(amount, wallet_principal).await?;

    Store::update_status(
        icp_transfer_blockheight,
        spawn_status.transferred_to_cmc(cmc_transfer_block_height),
    )?;

    // top up this canister with cycles
    let cycles = CyclesManagement::top_up(cmc_transfer_block_height, wallet_principal).await?;

    Store::update_status(
        icp_transfer_blockheight,
        spawn_status.topped_up_self(cycles.clone()),
    )?;

    Store::update_status(icp_transfer_blockheight, spawn_status.done())?;

    Ok(())
}

#[update(guard = "is_not_anonymous")]
async fn transfer_ownership(
    canister_id: Principal,
    new_owner: Principal,
) -> CanisterResult<(Principal, WalletData)> {
    Store::transfer_ownership(canister_id, new_owner).await
}

#[update]
async fn get_minimum_spawn_icp_amount() -> CanisterResult<Tokens> {
    CyclesManagement::get_minimum_spawn_icp_amount().await
}

#[query]
pub fn icts_name() -> String {
    env!("CARGO_PKG_NAME").to_string()
}

#[query]
pub fn icts_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[update(guard = "is_prod_developer")]
fn _dev_add_wallet(canister_id: Principal) -> bool {
    Store::_test_add_wallet(canister_id).is_ok()
}

#[update(guard = "is_prod_developer")]
fn _dev_set_proxy(canister_id: Principal) -> bool {
    ProxyCanisterStorage::set(canister_id).is_ok()
}

#[update(guard = "is_prod_developer")]
fn _dev_upload_multisig_wasm(wasm: Vec<u8>) -> bool {
    MultisigWasmStorage::set(wasm).is_ok()
}

#[update(guard = "is_prod_developer")]
fn _dev_prod_init() -> CanisterResult<()> {
    if id().to_string() != "o7ouu-niaaa-aaaap-ahhdq-cai" {
        return Err(
            Error::unsupported().add_message("This canister is not the production canister")
        );
    }

    let _ = ProxyCanisterStorage::set(Principal::from_text("2jvhk-5aaaa-aaaap-ahewa-cai").unwrap());
    Ok(())
}

#[query]
pub fn __get_candid_interface_tmp_hack() -> String {
    use candid::export_service;

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
