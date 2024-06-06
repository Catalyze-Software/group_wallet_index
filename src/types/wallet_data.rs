use candid::{CandidType, Principal};
use ic_cdk::{api::time, caller};
use serde::{Deserialize, Serialize};

use crate::impl_storable_for;

impl_storable_for!(WalletData);

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct WalletData {
    created_by: Principal,
    owner: Principal,
    created_at: u64,
    updated_at: u64,
    icp_blockheight: u64,
    cmc_blockheight: u64,
    group_id: u64,
}

impl WalletData {
    pub fn new(icp_blockheight: u64, cmc_blockheight: u64, group_id: u64) -> Self {
        Self {
            created_by: caller(),
            owner: caller(),
            created_at: time(),
            updated_at: time(),
            icp_blockheight,
            cmc_blockheight,
            group_id,
        }
    }

    pub fn is_owner(&self, principal: Principal) -> bool {
        self.owner == principal
    }

    pub fn set_owner(&mut self, owner: Principal) -> Self {
        self.owner = owner;
        self.clone()
    }
}
