use axum::{
    extract::{Path, State},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::AppState;
use crate::db::models::{TransactionRecord, VaultRecord};
use crate::error::{Result, VaultError};

use anchor_client::solana_sdk::{
    hash::Hash,
    instruction::Instruction,
    pubkey::Pubkey,
    system_program::ID as SYSTEM_PROGRAM_ID,
    sysvar,
    transaction::Transaction,
};
use anchor_lang::InstructionData;
use anchor_lang::ToAccountMetas;

use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine;

use anchor_spl::associated_token::get_associated_token_address;
use std::str::FromStr;

#[derive(Debug, Serialize)]
pub struct BalanceResponse {
    pub vault: VaultRecord,
}

pub async fn get_balance(
    State(state): State<Arc<AppState>>,
    Path(user): Path<String>,
) -> Result<Json<BalanceResponse>> {
    let vault = state.vault_manager.get_balance(&user).await?;

    Ok(Json(BalanceResponse { vault }))
}

#[derive(Debug, Serialize)]
pub struct TransactionsResponse {
    pub transactions: Vec<TransactionRecord>,
}

pub async fn get_transactions(
    State(state): State<Arc<AppState>>,
    Path(user): Path<String>,
) -> Result<Json<TransactionsResponse>> {
    let transactions = state.vault_manager.get_transactions(&user).await?;

    Ok(Json(TransactionsResponse { transactions }))
}

#[derive(Debug, Serialize)]
pub struct TvlResponse {
    pub total_value_locked: i64,
}

pub async fn get_tvl(
    State(state): State<Arc<AppState>>,
) -> Result<Json<TvlResponse>> {
    let tvl = state.vault_manager.get_tvl().await?;

    Ok(Json(TvlResponse {
        total_value_locked: tvl,
    }))
}

#[derive(Debug, Deserialize)]
pub struct BuildInitializeVaultTxRequest {
    pub user_pubkey: String,
}

#[derive(Debug, Deserialize)]
pub struct BuildDepositTxRequest {
    pub user_pubkey: String,
    pub amount: u64,
}

#[derive(Debug, Serialize)]
pub struct BuildTxResponse {
    // returning unsigned tx so client signs with their wallet
    // this way private keys never touch the backend
    pub transaction_base64: String,
    pub recent_blockhash: String,
    pub fee_payer: String,
}

fn make_unsigned_tx(ixs: Vec<Instruction>, fee_payer: Pubkey, recent_blockhash: Hash) -> Result<BuildTxResponse> {
    let mut tx = Transaction::new_with_payer(&ixs, Some(&fee_payer));
    tx.message.recent_blockhash = recent_blockhash;

    let tx_bytes = bincode::serialize(&tx)
        .map_err(|e| VaultError::Internal(format!("Failed to serialize tx: {e}")))?;

    Ok(BuildTxResponse {
        transaction_base64: BASE64_STANDARD.encode(tx_bytes),
        recent_blockhash: recent_blockhash.to_string(),
        fee_payer: fee_payer.to_string(),
    })
}

pub async fn build_initialize_vault_tx(
    State(state): State<Arc<AppState>>,
    Json(req): Json<BuildInitializeVaultTxRequest>,
) -> Result<Json<BuildTxResponse>> {
    let user = Pubkey::from_str(&req.user_pubkey)
        .map_err(|e| VaultError::InvalidAmount(format!("Invalid pubkey: {e}")))?;

    let solana_client = state.vault_manager.solana_client();

    // vault PDA ensures one vault per user
    let (vault_pda, _bump) = solana_client.derive_vault_pda(&user);
    let vault_token_account = get_associated_token_address(&vault_pda, &solana_client.usdt_mint);

    let ix: Instruction = Instruction {
        program_id: solana_client.program_id,
        accounts: collateral_vault::accounts::InitializeVault {
            owner: user,
            vault: vault_pda,
            vault_token_account,
            usdt_mint: solana_client.usdt_mint,
            token_program: anchor_spl::token::ID,
            associated_token_program: anchor_spl::associated_token::ID,
            system_program: SYSTEM_PROGRAM_ID,
            rent: sysvar::rent::ID,
        }
        .to_account_metas(None),
        data: collateral_vault::instruction::InitializeVault {}.data(),
    };

    let recent_blockhash = solana_client
        .rpc
        .get_latest_blockhash()
        .map_err(|e| VaultError::SolanaClient(e.to_string()))?;

    Ok(Json(make_unsigned_tx(vec![ix], user, recent_blockhash)?))
}

pub async fn build_deposit_tx(
    State(state): State<Arc<AppState>>,
    Json(req): Json<BuildDepositTxRequest>,
) -> Result<Json<BuildTxResponse>> {
    if req.amount == 0 {
        return Err(VaultError::InvalidAmount("Amount must be greater than zero".to_string()));
    }

    let user = Pubkey::from_str(&req.user_pubkey)
        .map_err(|e| VaultError::InvalidAmount(format!("Invalid pubkey: {e}")))?;

    let solana_client = state.vault_manager.solana_client();

    let (vault_pda, _bump) = solana_client.derive_vault_pda(&user);
    let vault_token_account = get_associated_token_address(&vault_pda, &solana_client.usdt_mint);
    let user_token_account = get_associated_token_address(&user, &solana_client.usdt_mint);

    let ix: Instruction = Instruction {
        program_id: solana_client.program_id,
        accounts: collateral_vault::accounts::Deposit {
            user,
            vault: vault_pda,
            user_token_account,
            vault_token_account,
            owner: user,
            token_program: anchor_spl::token::ID,
        }
        .to_account_metas(None),
        data: collateral_vault::instruction::Deposit { amount: req.amount }.data(),
    };

    let recent_blockhash = solana_client
        .rpc
        .get_latest_blockhash()
        .map_err(|e| VaultError::SolanaClient(e.to_string()))?;

    Ok(Json(make_unsigned_tx(vec![ix], user, recent_blockhash)?))
}

#[derive(Debug, Deserialize)]
pub struct BuildWithdrawTxRequest {
    pub user_pubkey: String,
    pub amount: u64,
}

// keeping these aliases for backward compat with older frontend versions
pub async fn build_initialize_unsigned(
    State(state): State<Arc<AppState>>,
    Json(req): Json<BuildInitializeVaultTxRequest>,
) -> Result<Json<BuildTxResponse>> {
    build_initialize_vault_tx(State(state), Json(req)).await
}

pub async fn build_deposit_unsigned(
    State(state): State<Arc<AppState>>,
    Json(req): Json<BuildDepositTxRequest>,
) -> Result<Json<BuildTxResponse>> {
    build_deposit_tx(State(state), Json(req)).await
}

pub async fn build_withdraw_unsigned(
    State(state): State<Arc<AppState>>,
    Json(req): Json<BuildWithdrawTxRequest>,
) -> Result<Json<BuildTxResponse>> {
    if req.amount == 0 {
        return Err(VaultError::InvalidAmount("Amount must be greater than zero".to_string()));
    }

    let user = Pubkey::from_str(&req.user_pubkey)
        .map_err(|e| VaultError::InvalidAmount(format!("Invalid pubkey: {e}")))?;

    let solana_client = state.vault_manager.solana_client();

    let (vault_pda, _bump) = solana_client.derive_vault_pda(&user);
    let vault_token_account = get_associated_token_address(&vault_pda, &solana_client.usdt_mint);
    let user_token_account = get_associated_token_address(&user, &solana_client.usdt_mint);

    let ix: Instruction = Instruction {
        program_id: solana_client.program_id,
        accounts: collateral_vault::accounts::Withdraw {
            user,
            vault: vault_pda,
            user_token_account,
            vault_token_account,
            owner: user,
            token_program: anchor_spl::token::ID,
        }
        .to_account_metas(None),
        data: collateral_vault::instruction::Withdraw { amount: req.amount }.data(),
    };

    let recent_blockhash = solana_client
        .rpc
        .get_latest_blockhash()
        .map_err(|e| VaultError::SolanaClient(e.to_string()))?;

    Ok(Json(make_unsigned_tx(vec![ix], user, recent_blockhash)?))
}

#[derive(Debug, Deserialize)]
pub struct SyncTxRequest {
    pub user_pubkey: String,
    pub signature: String,
    pub transaction_type: crate::db::models::TransactionType,
    // optional because older clients might not send it
    pub amount: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct SyncTxResponse {
    pub vault: VaultRecord,
    pub recorded: bool,
}

// called after client submits a tx to update our DB with on-chain state
pub async fn sync_tx(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SyncTxRequest>,
) -> Result<Json<SyncTxResponse>> {
    let vault = state
        .vault_manager
        .sync_confirmed_tx(
            &req.user_pubkey,
            &req.signature,
            req.transaction_type,
            req.amount,
        )
        .await?;

    Ok(Json(SyncTxResponse { vault, recorded: true }))
}

#[derive(Debug, Deserialize)]
pub struct ForceSyncRequest {
    pub user_pubkey: String,
}

// added this because during testing we had vaults on-chain that weren't in the DB
// lets us bootstrap existing vaults without needing a tx signature
pub async fn force_sync_vault(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ForceSyncRequest>,
) -> Result<Json<SyncTxResponse>> {
    use anchor_client::solana_sdk::pubkey::Pubkey;
    use std::str::FromStr;

    let user = Pubkey::from_str(&req.user_pubkey)
        .map_err(|e| VaultError::InvalidAmount(format!("Invalid pubkey: {e}")))?;

    let solana_client = state.vault_manager.solana_client();
    let (vault_pda, _bump) = solana_client.derive_vault_pda(&user);

    let account_data = solana_client
        .rpc
        .get_account_data(&vault_pda)
        .map_err(|e| {
            VaultError::SolanaClient(format!(
                "Failed to fetch vault account {}: {}. Vault may not exist on-chain yet.",
                vault_pda, e
            ))
        })?;

    // manually deserialize because we can't use Anchor's Account wrapper without the program
    // Anchor format: 8 byte discriminator + account data
    if account_data.len() < 8 {
        return Err(VaultError::SolanaClient(
            "Invalid account data: too short".to_string(),
        ));
    }

    // CollateralVault layout from program:
    // owner: Pubkey (32), token_account: Pubkey (32),
    // total_balance: u64 (8), locked_balance: u64 (8), available_balance: u64 (8),
    // total_deposited: u64 (8), total_withdrawn: u64 (8)
    let data = &account_data[8..];
    
    if data.len() < 32 + 32 + 8 + 8 + 8 + 8 + 8 {
        return Err(VaultError::SolanaClient(
            "Invalid account data: insufficient length".to_string(),
        ));
    }

    use std::convert::TryInto;
    
    let offset = 32 + 32;
    let total_balance = u64::from_le_bytes(data[offset..offset+8].try_into().unwrap()) as i64;
    let locked_balance = u64::from_le_bytes(data[offset+8..offset+16].try_into().unwrap()) as i64;
    let available_balance = u64::from_le_bytes(data[offset+16..offset+24].try_into().unwrap()) as i64;
    let total_deposited = u64::from_le_bytes(data[offset+24..offset+32].try_into().unwrap()) as i64;
    let total_withdrawn = u64::from_le_bytes(data[offset+32..offset+40].try_into().unwrap()) as i64;

    // sanity check - this invariant is enforced by the program
    if total_balance != locked_balance + available_balance {
        tracing::error!(
            "Balance consistency check failed: total_balance ({}) != locked_balance ({}) + available_balance ({})",
            total_balance, locked_balance, available_balance
        );
        return Err(VaultError::InvalidAmount(format!(
            "On-chain vault data inconsistent: total_balance ({}) != locked_balance ({}) + available_balance ({}). This may indicate the vault is corrupted or in an invalid state.",
            total_balance, locked_balance, available_balance
        )));
    }

    let vault_row = sqlx::query_as::<_, VaultRecord>(
        r#"
        INSERT INTO public.vaults (
            owner,
            vault_address,
            total_balance,
            locked_balance,
            available_balance,
            total_deposited,
            total_withdrawn
        )
        VALUES ($1,$2,$3,$4,$5,$6,$7)
        ON CONFLICT (vault_address)
        DO UPDATE SET
            owner = EXCLUDED.owner,
            total_balance = EXCLUDED.total_balance,
            locked_balance = EXCLUDED.locked_balance,
            available_balance = EXCLUDED.available_balance,
            total_deposited = EXCLUDED.total_deposited,
            total_withdrawn = EXCLUDED.total_withdrawn,
            updated_at = NOW()
        RETURNING *
        "#,
    )
    .bind(user.to_string())
    .bind(vault_pda.to_string())
    .bind(total_balance)
    .bind(locked_balance)
    .bind(available_balance)
    .bind(total_deposited)
    .bind(total_withdrawn)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| VaultError::Database(e.to_string()))?;

    Ok(Json(SyncTxResponse { vault: vault_row, recorded: true }))
}

