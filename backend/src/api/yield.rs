use axum::{
    extract::{Path, State},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::AppState;
use crate::db::models::VaultRecord;
use crate::error::{Result, VaultError};

use anchor_client::solana_sdk::{
    hash::Hash,
    instruction::Instruction,
    pubkey::Pubkey,
    transaction::Transaction,
};
use anchor_lang::InstructionData;
use anchor_lang::ToAccountMetas;

use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine;

use std::str::FromStr;

#[derive(Debug, Serialize)]
pub struct BuildTxResponse {
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

#[derive(Debug, Deserialize)]
pub struct CompoundYieldRequest {
    pub user_pubkey: String,
}

pub async fn build_compound_yield_tx(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CompoundYieldRequest>,
) -> Result<Json<BuildTxResponse>> {
    let user = Pubkey::from_str(&req.user_pubkey)
        .map_err(|e| VaultError::InvalidAmount(format!("Invalid pubkey: {e}")))?;

    let solana_client = state.vault_manager.solana_client();
    let (vault_pda, _bump) = solana_client.derive_vault_pda(&user);

    let ix: Instruction = Instruction {
        program_id: solana_client.program_id,
        accounts: collateral_vault::accounts::CompoundYield {
            user,
            vault: vault_pda,
            owner: user,
        }
        .to_account_metas(None),
        data: collateral_vault::instruction::CompoundYield {}.data(),
    };

    let recent_blockhash = solana_client
        .rpc
        .get_latest_blockhash()
        .map_err(|e| VaultError::SolanaClient(e.to_string()))?;

    Ok(Json(make_unsigned_tx(vec![ix], user, recent_blockhash)?))
}

#[derive(Debug, Deserialize)]
pub struct AutoCompoundRequest {
    pub vault_owner_pubkey: String,
    pub caller_pubkey: String,
}

pub async fn build_auto_compound_tx(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AutoCompoundRequest>,
) -> Result<Json<BuildTxResponse>> {
    let vault_owner = Pubkey::from_str(&req.vault_owner_pubkey)
        .map_err(|e| VaultError::InvalidAmount(format!("Invalid vault owner pubkey: {e}")))?;
    
    let caller = Pubkey::from_str(&req.caller_pubkey)
        .map_err(|e| VaultError::InvalidAmount(format!("Invalid caller pubkey: {e}")))?;

    let solana_client = state.vault_manager.solana_client();
    let (vault_pda, _bump) = solana_client.derive_vault_pda(&vault_owner);

    let ix: Instruction = Instruction {
        program_id: solana_client.program_id,
        accounts: collateral_vault::accounts::AutoCompound {
            caller,
            vault: vault_pda,
        }
        .to_account_metas(None),
        data: collateral_vault::instruction::AutoCompound {}.data(),
    };

    let recent_blockhash = solana_client
        .rpc
        .get_latest_blockhash()
        .map_err(|e| VaultError::SolanaClient(e.to_string()))?;

    Ok(Json(make_unsigned_tx(vec![ix], caller, recent_blockhash)?))
}

#[derive(Debug, Deserialize)]
pub struct ConfigureYieldRequest {
    pub user_pubkey: String,
    pub enabled: bool,
}

pub async fn build_configure_yield_tx(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ConfigureYieldRequest>,
) -> Result<Json<BuildTxResponse>> {
    let user = Pubkey::from_str(&req.user_pubkey)
        .map_err(|e| VaultError::InvalidAmount(format!("Invalid pubkey: {e}")))?;

    let solana_client = state.vault_manager.solana_client();
    let (vault_pda, _bump) = solana_client.derive_vault_pda(&user);

    let ix: Instruction = Instruction {
        program_id: solana_client.program_id,
        accounts: collateral_vault::accounts::ConfigureYield {
            user,
            vault: vault_pda,
            owner: user,
        }
        .to_account_metas(None),
        data: collateral_vault::instruction::ConfigureYield {
            enabled: req.enabled,
        }
        .data(),
    };

    let recent_blockhash = solana_client
        .rpc
        .get_latest_blockhash()
        .map_err(|e| VaultError::SolanaClient(e.to_string()))?;

    Ok(Json(make_unsigned_tx(vec![ix], user, recent_blockhash)?))
}

#[derive(Debug, Serialize)]
pub struct YieldInfoResponse {
    pub vault_address: String,
    pub yield_enabled: bool,
    pub total_yield_earned: u64,
    pub last_yield_compound: i64,
    pub estimated_next_yield: u64,
    pub time_until_next_compound: i64,
}

pub async fn get_yield_info(
    State(state): State<Arc<AppState>>,
    Path(user_pubkey): Path<String>,
) -> Result<Json<YieldInfoResponse>> {
    let user = Pubkey::from_str(&user_pubkey)
        .map_err(|e| VaultError::InvalidAmount(format!("Invalid pubkey: {e}")))?;

    let solana_client = state.vault_manager.solana_client();
    let (vault_pda, _bump) = solana_client.derive_vault_pda(&user);

    let account_data = solana_client
        .rpc
        .get_account_data(&vault_pda)
        .map_err(|e| {
            VaultError::SolanaClient(format!(
                "Failed to fetch vault account {}: {}. Vault may not exist.",
                vault_pda, e
            ))
        })?;

    if account_data.len() < 8 {
        return Err(VaultError::SolanaClient(
            "Invalid account data: too short".to_string(),
        ));
    }

    let data = &account_data[8..];
    use std::convert::TryInto;
    
    let total_balance = u64::from_le_bytes(data[64..72].try_into().unwrap_or([0u8; 8]));
    
    // Navigate through dynamic Vec fields to reach yield data
    let signers_len_offset = 114;
    let signers_len = u32::from_le_bytes(data[signers_len_offset..signers_len_offset+4].try_into().unwrap_or([0u8; 4])) as usize;
    let mut offset = signers_len_offset + 4 + (signers_len * 32);
    
    let delegates_len = u32::from_le_bytes(data[offset..offset+4].try_into().unwrap_or([0u8; 4])) as usize;
    offset += 4 + (delegates_len * 32);
    offset += 8;
    
    let has_pending = data[offset] != 0;
    offset += 1;
    if has_pending {
        offset += 56;
    }
    
    offset += 1;
    let yield_enabled = data[offset] != 0;
    offset += 1;
    
    let total_yield_earned = u64::from_le_bytes(data[offset..offset+8].try_into().unwrap_or([0u8; 8]));
    offset += 8;
    let last_yield_compound = i64::from_le_bytes(data[offset..offset+8].try_into().unwrap_or([0u8; 8]));

    let current_time = chrono::Utc::now().timestamp();
    let time_since_last = current_time - last_yield_compound;
    
    let annual_rate = 10000000u128;
    let seconds_per_year = 31_536_000u128;
    
    let estimated_next_yield = if total_balance > 0 && time_since_last > 0 {
        ((total_balance as u128 * annual_rate * time_since_last as u128) 
            / 10000 / seconds_per_year) as u64
    } else {
        0
    };

    let min_compound_interval = 10i64;
    let time_until_next = (last_yield_compound + min_compound_interval - current_time).max(0);

    Ok(Json(YieldInfoResponse {
        vault_address: vault_pda.to_string(),
        yield_enabled,
        total_yield_earned,
        last_yield_compound,
        estimated_next_yield,
        time_until_next_compound: time_until_next,
    }))
}

#[derive(Debug, Deserialize)]
pub struct SyncYieldRequest {
    pub user_pubkey: String,
    pub signature: String,
}

#[derive(Debug, Serialize)]
pub struct SyncYieldResponse {
    pub vault: VaultRecord,
    pub success: bool,
}

pub async fn sync_yield_tx(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SyncYieldRequest>,
) -> Result<Json<SyncYieldResponse>> {
    let user = Pubkey::from_str(&req.user_pubkey)
        .map_err(|e| VaultError::InvalidAmount(format!("Invalid pubkey: {e}")))?;

    let solana_client = state.vault_manager.solana_client();
    let (vault_pda, _bump) = solana_client.derive_vault_pda(&user);

    // Get the old vault state from database to calculate yield delta
    let old_vault = sqlx::query_as::<_, VaultRecord>(
        r#"SELECT * FROM public.vaults WHERE vault_address = $1"#,
    )
    .bind(vault_pda.to_string())
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| VaultError::Database(e.to_string()))?;

    // Fetch updated vault data from blockchain
    let account_data = solana_client
        .rpc
        .get_account_data(&vault_pda)
        .map_err(|e| VaultError::SolanaClient(format!("Failed to fetch vault: {e}")))?;

    let yield_amount = if account_data.len() >= 80 {
        let data = &account_data[8..];
        use std::convert::TryInto;
        
        let new_total_balance = u64::from_le_bytes(data[64..72].try_into().unwrap_or([0u8; 8])) as i64;
        
        // Calculate the yield amount as the difference in total_balance
        if let Some(old) = old_vault {
            Some(new_total_balance.saturating_sub(old.total_balance))
        } else {
            None
        }
    } else {
        None
    };

    let vault = state
        .vault_manager
        .sync_confirmed_tx(
            &req.user_pubkey,
            &req.signature,
            crate::db::models::TransactionType::Deposit,
            yield_amount,
        )
        .await?;

    Ok(Json(SyncYieldResponse { 
        vault, 
        success: true 
    }))
}
