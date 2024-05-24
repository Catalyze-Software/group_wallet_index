# My Rust Project

This project is a Rust application that provides a set of query and update functions for managing spawns and wallets.

## Features

- Query the number of cycles
- Query all spawns
- Query a specific spawn based on blockheight
- Query all wallets
- Spawn a wallet with a whitelist of principals

## Methods

### `get_cycles() -> u64`

This method returns the number of cycles.

### `get_spawns() -> Vec<(u64, SpawnStatus)>`

This method returns a vector of tuples containing a `u64` and `SpawnStatus`.

### `get_spawn(blockheight: u64) -> Result<SpawnStatus, String>`

This method returns a `SpawnStatus` or a `String` error. It retrieves a specific spawn based on the blockheight.

### `get_wallets() -> Vec<(Principal, MultisigData)>`

This method returns a vector of tuples containing a `Principal` and `MultisigData`.

### `spawn_wallet(icp_transfer_blockheight: u64, whitelist: Vec<Principal>) -> Result<Principal, String>`

This method spawns a multisig with a whitelist of principals. It includes:

- Validation of the whitelist.
- Check if a spawn already exists for the given blockheight.
- Initialization of a new spawn status tracker.
- Saving of the spawn status after initialization.
