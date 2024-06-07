use candid::Principal;
use ic_cdk::{caller, id};
use ic_ledger_types::{
    query_archived_blocks, query_blocks, transfer, AccountIdentifier, Block, BlockIndex,
    GetBlocksArgs, Memo, Subaccount, Tokens, TransferArgs, DEFAULT_SUBACCOUNT,
    MAINNET_CYCLES_MINTING_CANISTER_ID, MAINNET_LEDGER_CANISTER_ID,
};

use crate::{
    storage::state::{CATALYZE_E8S_FEE, ICP_TRANSACTION_FEE, MEMO_TOP_UP_CANISTER},
    types::{error::Error, result::CanisterResult},
};

pub struct Ledger;

impl Ledger {
    pub async fn transfer_icp_back_to_caller(amount: Tokens) -> CanisterResult<u64> {
        let send_back_amount = amount - ICP_TRANSACTION_FEE;

        let transfer_back_args = TransferArgs {
            memo: Memo(0),
            amount: send_back_amount,
            fee: ICP_TRANSACTION_FEE,
            from_subaccount: None,
            to: AccountIdentifier::new(&caller(), &DEFAULT_SUBACCOUNT),
            created_at_time: None,
        };

        let blockheight = transfer(MAINNET_LEDGER_CANISTER_ID, transfer_back_args)
            .await
            .map_err(|e| Error::internal().add_message(e.1.as_str()))?
            .map_err(|e| Error::bad_request().add_message(e.to_string().as_str()))?;

        Ok(blockheight)
    }

    pub async fn transfer_icp_to_cmc(
        amount: Tokens,
        canister_id: Principal,
    ) -> CanisterResult<u64> {
        let catalyze_amount = CATALYZE_E8S_FEE - ICP_TRANSACTION_FEE;
        let wallet_amount = amount - ICP_TRANSACTION_FEE - catalyze_amount;

        let multig_spinup_ledger_args = TransferArgs {
            memo: MEMO_TOP_UP_CANISTER,
            amount: wallet_amount,
            fee: ICP_TRANSACTION_FEE,
            from_subaccount: None,
            to: AccountIdentifier::new(
                &MAINNET_CYCLES_MINTING_CANISTER_ID,
                &Subaccount::from(canister_id),
            ),
            created_at_time: None,
        };

        let blockheight = transfer(MAINNET_LEDGER_CANISTER_ID, multig_spinup_ledger_args)
            .await
            .map_err(|e| Error::internal().add_message(e.1.as_str()))?
            .map_err(|e| Error::bad_request().add_message(e.to_string().as_str()))?;

        Ok(blockheight)
    }

    // This method checks if the transaction is send and received from the given principal
    pub async fn validate_transaction(
        principal: Principal,
        block_index: BlockIndex,
    ) -> CanisterResult<Tokens> {
        // Get the block
        let block = Self::get_block(block_index)
            .await
            .ok_or(Error::not_found().add_message("Block not found"))?;

        // Check if the block has a transaction
        match block.transaction.operation {
            Some(op) => match op {
                ic_ledger_types::Operation::Transfer {
                    from,
                    to,
                    amount,
                    fee: _,
                } => {
                    if from != Self::principal_to_account_identifier(principal) {
                        return Err(Error::bad_request()
                            .add_message("Transaction not from the given principal"));
                    }
                    if to != Self::principal_to_account_identifier(id()) {
                        return Err(Error::bad_request()
                            .add_message("Transaction not to the given principal"));
                    }
                    Ok(amount)
                }
                _ => Err(Error::unsupported().add_message("Not a transfer")),
            },
            None => Err(Error::not_found().add_message("Transaction not found")),
        }
    }

    async fn get_block(block_index: BlockIndex) -> Option<Block> {
        let args = GetBlocksArgs {
            start: block_index,
            length: 1,
        };

        let blocks_result = query_blocks(MAINNET_LEDGER_CANISTER_ID, args.clone())
            .await
            .ok()?;

        if !blocks_result.blocks.is_empty() {
            debug_assert_eq!(blocks_result.first_block_index, block_index);
            return blocks_result.blocks.into_iter().next();
        }

        if let Some(func) = blocks_result.archived_blocks.into_iter().find_map(|b| {
            (b.start <= block_index && (block_index - b.start) < b.length).then_some(b.callback)
        }) {
            query_archived_blocks(&func, args)
                .await
                .ok()?
                .map_err(|e| e.to_string())
                .ok();
        }

        None
    }

    fn principal_to_account_identifier(principal: Principal) -> AccountIdentifier {
        AccountIdentifier::new(&principal, &DEFAULT_SUBACCOUNT)
    }
}
