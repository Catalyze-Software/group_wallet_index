use candid::{CandidType, Nat, Principal};
use ic_cdk::{api::time, caller};
use ic_ledger_types::Tokens;
use serde::Deserialize;
use std::borrow::Cow;

use candid::{Decode, Encode};
use ic_stable_structures::{storable::Bound, Storable};

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct MultisigData {
    created_by: Principal,
    created_at: u64,
    icp_blockheight: u64,
    cmc_blockheight: u64,
}

impl MultisigData {
    pub fn new(icp_blockheight: u64, cmc_blockheight: u64) -> Self {
        Self {
            created_by: caller(),
            created_at: time(),
            icp_blockheight,
            cmc_blockheight,
        }
    }
}

impl Storable for MultisigData {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Unbounded;
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct SpawnStatus {
    transaction_valid: Option<Tokens>,
    min_amount_error: Option<u64>,
    transferred_to_cmc: Option<u64>,
    topped_up_self: Option<Nat>,
    canister_spawned: Option<Principal>,
    canister_installed: Option<Principal>,
    done: Option<()>,
}

impl Default for SpawnStatus {
    fn default() -> Self {
        Self::new()
    }
}

impl SpawnStatus {
    pub fn new() -> Self {
        Self {
            transaction_valid: None,
            min_amount_error: None,
            transferred_to_cmc: None,
            topped_up_self: None,
            canister_spawned: None,
            canister_installed: None,
            done: None,
        }
    }

    pub fn transaction_valid(&mut self, amount: Tokens) -> Self {
        self.transaction_valid = Some(amount);
        self.clone()
    }

    pub fn min_amount_error(&mut self, block_index: u64) -> Self {
        self.min_amount_error = Some(block_index);
        self.clone()
    }

    pub fn transferred_to_cmc(&mut self, block_index: u64) -> Self {
        self.transferred_to_cmc = Some(block_index);
        self.clone()
    }

    pub fn topped_up_self(&mut self, cycles: Nat) -> Self {
        self.topped_up_self = Some(cycles);
        self.clone()
    }

    pub fn canister_spawned(&mut self, canister_id: Principal) -> Self {
        self.canister_spawned = Some(canister_id);
        self.clone()
    }

    pub fn canister_installed(&mut self, canister_id: Principal) -> Self {
        self.canister_installed = Some(canister_id);
        self.clone()
    }

    pub fn done(&mut self) -> Self {
        self.done = Some(());
        self.clone()
    }
}

impl Storable for SpawnStatus {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Unbounded;
}
