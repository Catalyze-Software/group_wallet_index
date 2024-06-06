use std::{cell::RefCell, thread::LocalKey};

use ic_stable_structures::{memory_manager::MemoryId, Cell, Storable};

use crate::types::{error::Error, result::CanisterResult};

use super::state::Memory;

pub type CellStorageRef<V> = &'static LocalKey<RefCell<Cell<Option<V>, Memory>>>;

pub trait CellStorage<V: Storable + Clone + 'static> {
    const NAME: &'static str;

    fn memory_id() -> MemoryId;
    fn storage() -> CellStorageRef<V>;

    fn get() -> CanisterResult<V> {
        Self::storage()
            .with(|data| data.borrow().get().clone())
            .ok_or_else(|| {
                Error::internal()
                    .add_message(&format!("Failed to get {}, not initialized", Self::NAME))
            })
    }

    fn set(value: V) -> Result<V, Error> {
        Self::storage()
            .with(|data| data.borrow_mut().set(Some(value.clone())))
            .map_err(|_| Error::internal().add_message(&format!("Failed to set {}", Self::NAME)))?;
        Ok(value)
    }
}
