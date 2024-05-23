# Build script multisig_index canister

# Generate candid
cargo test candid -p multisig_index

# Build wasm
cargo build -p multisig_index --release --target wasm32-unknown-unknown

# Gzip wasm
gzip -c target/wasm32-unknown-unknown/release/multisig_index.wasm > target/wasm32-unknown-unknown/release/multisig_index.wasm.gz

# Copy wasm
cp target/wasm32-unknown-unknown/release/multisig_index.wasm.gz wasm/multisig_index.wasm.gz
