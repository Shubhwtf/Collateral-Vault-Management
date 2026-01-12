use anchor_client::solana_sdk::pubkey::Pubkey;
use sqlx::PgPool;
use std::str::FromStr;
use sqlx::Row;

use crate::db::models::{VaultRecord, TransactionRecord};
use crate::error::{Result, VaultError};
use crate::solana::SolanaClient;

pub struct VaultManager {
    solana_client: SolanaClient,
    db_pool: PgPool,
}

impl VaultManager {
    pub fn new(solana_client: SolanaClient, db_pool: PgPool) -> Self {
        Self {
            solana_client,
            db_pool,
        }
    }

    // these methods all return UserSignatureRequired because the on-chain program requires
    // the user to be a signer - we can't submit these transactions on their behalf
    pub async fn initialize_vault(&self, user_pubkey: &str) -> Result<VaultRecord> {
        let _ = user_pubkey;
        Err(VaultError::UserSignatureRequired(
            "initialize_vault must be signed by the user wallet (owner is Signer and payer).".to_string(),
        ))
    }

    pub async fn deposit(&self, user_pubkey: &str, amount: u64) -> Result<()> {
        let _ = (user_pubkey, amount);
        Err(VaultError::UserSignatureRequired(
            "deposit must be signed by the user wallet (user is SPL token transfer authority).".to_string(),
        ))
    }

    pub async fn withdraw(&self, user_pubkey: &str, amount: u64) -> Result<()> {
        let _ = (user_pubkey, amount);
        Err(VaultError::UserSignatureRequired(
            "withdraw must be signed by the user wallet (user is Signer in the instruction).".to_string(),
        ))
    }

    pub async fn get_balance(&self, user_pubkey: &str) -> Result<VaultRecord> {
        let user = Pubkey::from_str(user_pubkey)
            .map_err(|e| VaultError::InvalidAmount(format!("Invalid pubkey: {}", e)))?;

        let (vault_pda, _) = self.solana_client.derive_vault_pda(&user);
        
        let vault = sqlx::query_as::<_, VaultRecord>(
            r#"SELECT * FROM public.vaults WHERE vault_address = $1"#,
        )
        .bind(vault_pda.to_string())
        .fetch_optional(&self.db_pool)
        .await
        .map_err(|e| VaultError::Database(e.to_string()))?
        .ok_or_else(|| VaultError::VaultNotFound(user_pubkey.to_string()))?;
        
        Ok(vault)
    }

    pub async fn get_transactions(&self, user_pubkey: &str) -> Result<Vec<TransactionRecord>> {
        let user = Pubkey::from_str(user_pubkey)
            .map_err(|e| VaultError::InvalidAmount(format!("Invalid pubkey: {}", e)))?;

        let (vault_pda, _) = self.solana_client.derive_vault_pda(&user);
        
        let transactions = sqlx::query_as::<_, TransactionRecord>(
            r#"
            SELECT * FROM public.transactions
            WHERE vault_address = $1
            ORDER BY created_at DESC
            LIMIT 100
            "#,
        )
        .bind(vault_pda.to_string())
        .fetch_all(&self.db_pool)
        .await
        .map_err(|e| VaultError::Database(e.to_string()))?;
        
        Ok(transactions)
    }

    pub async fn get_tvl(&self) -> Result<i64> {
        let row = sqlx::query(
            r#"SELECT COALESCE(SUM(total_balance), 0)::BIGINT AS tvl FROM public.vaults"#,
        )
        .fetch_one(&self.db_pool)
        .await
        .map_err(|e| VaultError::Database(e.to_string()))?;
        
        let tvl = row
        .try_get::<i64, _>("tvl")
        .map_err(|e| VaultError::Database(e.to_string()))?;
    
        Ok(tvl)
    }

    pub fn solana_client(&self) -> &SolanaClient {
        &self.solana_client
    }

    // called after client submits a tx to sync DB with on-chain state
    // polls for confirmation since client might call this immediately after submit
    pub async fn sync_confirmed_tx(
        &self,
        user_pubkey: &str,
        signature: &str,
        expected_type: crate::db::models::TransactionType,
        expected_amount: Option<i64>,
    ) -> Result<VaultRecord> {
        use anchor_client::solana_sdk::signature::Signature;

        let user = Pubkey::from_str(user_pubkey)
            .map_err(|e| VaultError::InvalidAmount(format!("Invalid pubkey: {e}")))?;
        let sig = Signature::from_str(signature)
            .map_err(|e| VaultError::InvalidAmount(format!("Invalid signature: {e}")))?;

        // polling up to 15 seconds for confirmation
        let mut last_not_found = true;
        for _ in 0..30u32 {
            let statuses = self
                .solana_client
                .rpc
                .get_signature_statuses(&[sig])
                .map_err(|e| VaultError::SolanaClient(e.to_string()))?;

            match statuses.value.into_iter().next().flatten() {
                None => {
                    last_not_found = true;
                }
                Some(st) => {
                    last_not_found = false;

                    if let Some(err) = st.err {
                        return Err(VaultError::TransactionFailed(format!(
                            "On-chain transaction failed: {err:?}"
                        )));
                    }

                    if st.confirmation_status.is_some() {
                        break;
                    }
                }
            }

            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }

        if last_not_found {
            return Err(VaultError::TransactionFailed(
                "Signature not found yet (tx not confirmed).".to_string(),
            ));
        }

        let (vault_pda, _bump) = self.solana_client.derive_vault_pda(&user);

        let account_data = self
            .solana_client
            .rpc
            .get_account_data(&vault_pda)
            .map_err(|e| {
                VaultError::SolanaClient(format!(
                    "Failed to fetch vault account {}: {}",
                    vault_pda, e
                ))
            })?;

        // manually parsing account data since we're not using Anchor's Account wrapper
        if account_data.len() < 8 + 32 + 32 + 8 + 8 + 8 + 8 + 8 {
            return Err(VaultError::SolanaClient(
                "Invalid vault account data".to_string(),
            ));
        }

        let data = &account_data[8..];
        let offset = 32 + 32;
        
        use std::convert::TryInto;
        let total_balance = u64::from_le_bytes(data[offset..offset+8].try_into().unwrap()) as i64;
        let locked_balance = u64::from_le_bytes(data[offset+8..offset+16].try_into().unwrap()) as i64;
        let available_balance = u64::from_le_bytes(data[offset+16..offset+24].try_into().unwrap()) as i64;
        let total_deposited = u64::from_le_bytes(data[offset+24..offset+32].try_into().unwrap()) as i64;
        let total_withdrawn = u64::from_le_bytes(data[offset+32..offset+40].try_into().unwrap()) as i64;

        if let Some(expected_amount) = expected_amount {
            if expected_amount <= 0 {
                return Err(VaultError::InvalidAmount(
                    "expected_amount must be > 0".to_string(),
                ));
            }
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
        .fetch_one(&self.db_pool)
        .await
        .map_err(|e| VaultError::Database(e.to_string()))?;

        // idempotent insert - avoid duplicate transaction records if client retries
        let exists = sqlx::query(r#"SELECT 1 FROM public.transactions WHERE signature = $1 LIMIT 1"#)
            .bind(signature)
            .fetch_optional(&self.db_pool)
            .await
            .map_err(|e| VaultError::Database(e.to_string()))?
            .is_some();

        if !exists {
            let amount_to_store = expected_amount.unwrap_or(1);

            sqlx::query(
                r#"
                INSERT INTO public.transactions (
                    vault_address,
                    transaction_type,
                    amount,
                    signature
                )
                VALUES ($1,$2,$3,$4)
                "#,
            )
            .bind(vault_pda.to_string())
            .bind(expected_type)
            .bind(amount_to_store)
            .bind(signature)
            .execute(&self.db_pool)
            .await
            .map_err(|e| VaultError::Database(e.to_string()))?;
        }

        Ok(vault_row)
    }
}