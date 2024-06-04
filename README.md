# Wallet Index Canister

This project is a Rust application that serves as an index canister for managing wallet canisters on the Internet Computer. It provides a set of functions for creating wallet canisters, topping up canisters with cycles by converting ICP (Internet Computer Protocol) tokens to cycles, and querying the status of these operations.

## Features

### Wallet Canister Management

The index canister can create new wallet canisters with the `spawn_wallet` function. This function takes a blockheight of the ICP transfer and a whitelist of principals as arguments. It validates the whitelist, checks if a spawn already exists for the given blockheight, initializes a new spawn status tracker, and saves the spawn status after initialization.

### ICP to Cycles Conversion

The index canister can top up canisters with cycles by converting ICP tokens to cycles with the `top_up_wallet` function. This function takes a blockheight of the ICP transfer and the principal of the wallet to be topped up as arguments. It checks if a spawn already exists for the given blockheight, initializes a new status tracker, validates the ICP transaction, updates the status tracker with the transaction amount, transfers the ICP to the cycles management canister, and updates the status tracker with the blockheight of the transfer.

### Query Functions

The index canister provides several query functions for retrieving the number of cycles, all spawns, a specific spawn based on blockheight, and all wallets.

## How to Run

To run this project, you need to have Rust installed on your machine. Once you have Rust installed, you can clone this repository and run the application with the following commands:

```bash
git clone <repository-url>
cd <repository-name>
cargo run
```
