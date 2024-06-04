use candid::{Nat, Principal};
use ic_ledger_types::MAINNET_CYCLES_MINTING_CANISTER_ID;

use crate::rust_declarations::cmc_service::{CmcService, NotifyTopUpArg, NotifyTopUpResult};

pub struct CyclesManagementCanister;

impl CyclesManagementCanister {
    pub async fn top_up(block_index: u64, canister_id: Principal) -> Result<Nat, String> {
        match CmcService(MAINNET_CYCLES_MINTING_CANISTER_ID)
            .notify_top_up(NotifyTopUpArg {
                block_index,
                canister_id,
            })
            .await
        {
            Ok((result,)) => match result {
                NotifyTopUpResult::Ok(cycles) => Ok(cycles),
                NotifyTopUpResult::Err(err) => Err(format!("{:?}", err)),
            },
            Err((_, err)) => Err(err),
        }
    }
}
