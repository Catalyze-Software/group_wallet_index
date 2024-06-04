# Build script wallet_index canister

# Generate candid
cargo test candid -p wallet_index

# Build wasm
cargo build -p wallet_index --release --target wasm32-unknown-unknown

# Gzip wasm
gzip -c target/wasm32-unknown-unknown/release/wallet_index.wasm > target/wasm32-unknown-unknown/release/wallet_index.wasm.gz

# Copy wasm
cp target/wasm32-unknown-unknown/release/wallet_index.wasm.gz wasm/wallet_index.wasm.gz
