# bin/bash

dfx identity use catalyze_production
OWNER_IDENTITY=$(dfx identity whoami)
PEM_FILE="$(readlink -f "$HOME/.config/dfx/identity/${OWNER_IDENTITY}/identity.pem")"
DEVELOPER_NEURON_ID="37173462e82235788f2592e076b31cf0e8601eff16b2a8687b564589d867de36"

VERSION=$(grep '^version = ' src/Cargo.toml | sed -E 's/version = "(.*)"/\1/')
CHANGELOG="Minor bug fixes and improvements."
# CHANGELOG=$(sed -n '/## \['"$VERSION"'\]/,/^## \[/p' CHANGELOG.md | sed '$d')
WASM_PATH="wasm/wallet_index.wasm.gz"

bash scripts/build.sh
UPGRADE_CANISTER_ID="o7ouu-niaaa-aaaap-ahhdq-cai"

TITLE="Upgrade multisig index canister to version $VERSION"
URL="https://catalyze.one"
SUMMARY=$CHANGELOG

quill sns make-upgrade-canister-proposal \
   --canister-ids-file scripts/sns_canister_ids.json  \
   --target-canister-id "${UPGRADE_CANISTER_ID}" \
   --wasm-path "${WASM_PATH}" \
   --summary "${SUMMARY}" \
   --pem-file "${PEM_FILE}" \
   --url "${URL}" \
   --title "${TITLE}" \
   "${DEVELOPER_NEURON_ID}" > msg.json
   
quill send --yes msg.json

rm msg.json