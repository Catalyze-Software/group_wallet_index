use super::store::Store;
use candid::Principal;
use ic_cdk::{
    api::call::{call, CallResult},
    caller, update,
};

#[update(guard = "is_known_wallet")]
pub async fn multisig_whitelist_notice_notification(
    receivers: Vec<Principal>,
    proxy_canister_id: Principal,
) {
    let _: CallResult<(bool,)> = call(
        proxy_canister_id,
        "multisig_whitelist_notice_notification",
        (receivers, caller()),
    )
    .await;
}

#[update(guard = "is_known_wallet")]
pub async fn multisig_proposal_accept_notification(
    receivers: Vec<Principal>,
    proposal_id: u64,
    proxy_canister_id: Principal,
) {
    let _: CallResult<(bool,)> = call(
        proxy_canister_id,
        "multisig_proposal_accept_notification",
        (receivers, caller(), proposal_id),
    )
    .await;
}

#[update(guard = "is_known_wallet")]
pub async fn multisig_proposal_decline_notification(
    receivers: Vec<Principal>,
    proposal_id: u64,
    proxy_canister_id: Principal,
) {
    let _: CallResult<(bool,)> = call(
        proxy_canister_id,
        "multisig_proposal_decline_notification",
        (receivers, caller(), proposal_id),
    )
    .await;
}
#[update(guard = "is_known_wallet")]
pub async fn multisig_proposal_status_update_notification(
    receivers: Vec<Principal>,
    proposal_id: u64,
    proxy_canister_id: Principal,
) {
    let _: CallResult<(bool,)> = call(
        proxy_canister_id,
        "multisig_proposal_status_update_notification",
        (receivers, caller(), proposal_id),
    )
    .await;
}

#[update(guard = "is_known_wallet")]
pub async fn multisig_new_proposal_notification(
    receivers: Vec<Principal>,
    proposal_id: u64,
    proxy_canister_id: Principal,
) {
    let _: CallResult<(bool,)> = call(
        proxy_canister_id,
        "multisig_new_proposal_notification",
        (receivers, caller(), proposal_id),
    )
    .await;
}

fn is_known_wallet() -> Result<(), String> {
    match Store::get_wallet(&caller()).is_none() {
        true => Err("unauthorized".to_string()),
        false => Ok(()),
    }
}
