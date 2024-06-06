# # upload scalable wasm
# FILE=files/tos.txt
# CANISTER=o7ouu-niaaa-aaaap-ahhdq-cai
# IDENTITY=$(dfx identity whoami)

# ic-repl -r ic << END
# identity ${IDENTITY} "~/.config/dfx/identity/${IDENTITY}/identity.pem"
# import controller = "${CANISTER}" as "candid/static_files.did"
# call controller.upload_document(variant{TermsOfService=file("${FILE}")})
# END