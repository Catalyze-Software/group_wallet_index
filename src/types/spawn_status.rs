use candid::{CandidType, Nat, Principal};
use ic_ledger_types::Tokens;
use serde::{Deserialize, Serialize};

use crate::impl_storable_for;

impl_storable_for!(SpawnStatus);

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct SpawnStatus {
    status_type: Option<String>,
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
        Self::new(None)
    }
}

impl SpawnStatus {
    pub fn new(status_type: Option<String>) -> Self {
        Self {
            status_type,
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
