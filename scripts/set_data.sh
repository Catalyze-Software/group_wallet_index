# catalyze_production

# upload scalable wasm
FILE=scripts/multisig.wasm.gz
CANISTER=o7ouu-niaaa-aaaap-ahhdq-cai
IDENTITY=$(dfx identity whoami)

ic-repl -r ic << END
identity ${IDENTITY} "~/.config/dfx/identity/${IDENTITY}/identity.pem"
import controller = "${CANISTER}" as "candid/wallet_index.did"
call controller._dev_upload_multisig_wasm(file("${FILE}"))
call controller._dev_set_proxy(principal "onidn-byaaa-aaaap-ahhaq-cai")
END