use candid::Principal;
use ic_cdk::{caller, id};
use ic_ledger_types::{
    query_archived_blocks, query_blocks, transfer, AccountIdentifier, Block, BlockIndex,
    GetBlocksArgs, Memo, Subaccount, Tokens, TransferArgs, DEFAULT_SUBACCOUNT,
    MAINNET_CYCLES_MINTING_CANISTER_ID, MAINNET_LEDGER_CANISTER_ID,
};

use super::store::{CATALYZE_E8S_FEE, ICP_TRANSACTION_FEE, MEMO_TOP_UP_CANISTER};

pub struct Ledger;

impl Ledger {
    pub async fn transfer_icp_back_to_caller(amount: Tokens) -> Result<u64, String> {
        let send_back_amount = ICP_TRANSACTION_FEE - amount;

        let transfer_back_args = TransferArgs {
            memo: Memo(0),
            amount: send_back_amount,
            fee: ICP_TRANSACTION_FEE,
            from_subaccount: None,
            to: AccountIdentifier::new(&caller(), &DEFAULT_SUBACCOUNT),
            created_at_time: None,
        };

        match transfer(MAINNET_LEDGER_CANISTER_ID, transfer_back_args).await {
            Ok(result) => match result {
                Ok(block_index) => Ok(block_index),
                Err(err) => Err(err.to_string()),
            },
            Err((_, err)) => Err(err),
        }
    }

    pub async fn transfer_icp_to_cmc(amount: Tokens) -> Result<u64, String> {
        let catalyze_amount = CATALYZE_E8S_FEE - ICP_TRANSACTION_FEE;
        let multisig_amount = amount - ICP_TRANSACTION_FEE - catalyze_amount;

        let multig_spinup_ledger_args = TransferArgs {
            memo: MEMO_TOP_UP_CANISTER,
            amount: multisig_amount,
            fee: ICP_TRANSACTION_FEE,
            from_subaccount: None,
            to: AccountIdentifier::new(
                &MAINNET_CYCLES_MINTING_CANISTER_ID,
                &Subaccount::from(id()),
            ),
            created_at_time: None,
        };

        match transfer(MAINNET_LEDGER_CANISTER_ID, multig_spinup_ledger_args).await {
            Ok(result) => match result {
                Ok(block_index) => Ok(block_index),
                Err(err) => Err(err.to_string()),
            },
            Err((_, err)) => Err(err),
        }
    }

    // This method checks if the transaction is send and received from the given principal
    pub async fn validate_transaction(
        principal: Principal,
        block_index: BlockIndex,
    ) -> Result<Tokens, String> {
        // Get the block
        let block = Self::get_block(block_index)
            .await
            .ok_or("Block not found".to_string())?;

        // Check if the block has a transaction
        if let Some(operation) = block.transaction.operation {
            if let ic_ledger_types::Operation::Transfer {
                from,
                to,
                amount,
                fee: _, // Ignore fee
            } = operation
            {
                if from != Self::principal_to_account_identifier(principal) {
                    return Err("Transaction not from the given principal".to_string());
                }
                if to != Self::principal_to_account_identifier(id()) {
                    return Err("Transaction not to the given principal".to_string());
                }
                Ok(amount)
            } else {
                // Not a transfer
                Err("Not a transfer".to_string())
            }
        } else {
            // No operation
            Err("No operation".to_string())
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
            match query_archived_blocks(&func, args).await {
                Ok(range) => match range {
                    Ok(_range) => return _range.blocks.into_iter().next(),
                    Err(_) => return None,
                },
                Err(_) => return None,
            }
        }

        None
    }

    fn principal_to_account_identifier(principal: Principal) -> AccountIdentifier {
        AccountIdentifier::new(&principal, &DEFAULT_SUBACCOUNT)
    }
}
