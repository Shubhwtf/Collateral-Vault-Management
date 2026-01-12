use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Type;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct VaultRecord {
    pub id: i32,
    pub owner: String,
    pub vault_address: String,
    // all balances stored as i64 to match Postgres BIGINT
    // this avoids floating point issues with large lamport amounts
    pub total_balance: i64,
    pub locked_balance: i64,
    pub available_balance: i64,
    pub total_deposited: i64,
    pub total_withdrawn: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
#[sqlx(type_name = "transaction_type", rename_all = "lowercase")]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Lock,
    Unlock,
    Transfer,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TransactionRecord {
    pub id: i32,
    pub vault_address: String,
    pub transaction_type: TransactionType,
    pub amount: i64,
    pub signature: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TvlSnapshot {
    pub id: i32,
    pub snapshot_time: DateTime<Utc>,
    pub total_value_locked: i64,
    pub total_users: i32,
    pub active_vaults: i32,
    pub total_deposited: i64,
    pub total_withdrawn: i64,
    pub average_balance: i64,
    pub created_at: DateTime<Utc>,
}

