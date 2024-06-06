use candid::{Nat, Principal};
use ic_ledger_types::{Tokens, MAINNET_CYCLES_MINTING_CANISTER_ID};

use crate::{
    services::cmc_service::{CmcService, NotifyTopUpArg, NotifyTopUpResult},
    storage::state::{CATALYZE_E8S_FEE, MIN_CYCLES_FOR_SPINUP},
    types::{error::Error, result::CanisterResult},
};

pub struct CyclesManagement;

impl CyclesManagement {
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

    pub async fn get_cycles_per_icp() -> CanisterResult<u64> {
        let cmc = CmcService(MAINNET_CYCLES_MINTING_CANISTER_ID);
        let result = cmc
            .get_icp_xdr_conversion_rate()
            .await
            .map(|(rate,)| rate)
            .map_err(|_| Error::bad_request().add_message("Error getting XDR conversion rate"))?;

        Ok(result.data.xdr_permyriad_per_icp * 1_000_000_000_000 / 10_000)
    }

    pub async fn get_minimum_spawn_icp_amount() -> CanisterResult<Tokens> {
        let cycles_per_icp = CyclesManagement::get_cycles_per_icp().await?;
        let calc = MIN_CYCLES_FOR_SPINUP as f64 / cycles_per_icp as f64;
        Ok(Tokens::from_e8s((calc * 1e8) as u64) + CATALYZE_E8S_FEE)
    }
}
