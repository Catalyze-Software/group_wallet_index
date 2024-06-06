use candid::{Nat, Principal};
use ic_ledger_types::MAINNET_CYCLES_MINTING_CANISTER_ID;

use crate::{
    services::cmc_service::{CmcService, NotifyTopUpArg, NotifyTopUpResult},
    types::{error::Error, result::CanisterResult},
};

pub struct CyclesManagementCanister;

impl CyclesManagementCanister {
    pub async fn top_up(block_index: u64, canister_id: Principal) -> CanisterResult<Nat> {
        let call = CmcService(MAINNET_CYCLES_MINTING_CANISTER_ID)
            .notify_top_up(NotifyTopUpArg {
                block_index,
                canister_id,
            })
            .await
            .map_err(|e| Error::internal().add_message(e.1.as_str()))?
            .0;

        match call {
            NotifyTopUpResult::Ok(result) => Ok(result),
            NotifyTopUpResult::Err(err) => {
                Err(Error::bad_request().add_message(format!("Error: {:?}", err).as_str()))
            }
        }
    }
}
