use ic_stable_structures::memory_manager::MemoryId;

use super::{
    cell_api::{CellStorage, CellStorageRef},
    state::{MULTISIG_WASM, MULTISIG_WASM_MEMORY_ID},
};

pub struct MultisigWasmStorage;

impl CellStorage<Vec<u8>> for MultisigWasmStorage {
    const NAME: &'static str = "multisig_wasm";

    fn storage() -> CellStorageRef<Vec<u8>> {
        &MULTISIG_WASM
    }

    fn memory_id() -> MemoryId {
        MULTISIG_WASM_MEMORY_ID
    }
}
