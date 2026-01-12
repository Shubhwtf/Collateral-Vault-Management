use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum VaultError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Solana client error: {0}")]
    SolanaClient(String),

    #[error("Vault not found for user: {0}")]
    VaultNotFound(String),

    #[error("Insufficient balance: available={0}, requested={1}")]
    InsufficientBalance(u64, u64),

    #[error("Invalid amount: {0}")]
    InvalidAmount(String),

    #[error("Transaction failed: {0}")]
    TransactionFailed(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Internal server error: {0}")]
    Internal(String),

    #[error("User signature required: {0}")]
    UserSignatureRequired(String),
}

impl IntoResponse for VaultError {
    fn into_response(self) -> Response {
        let (status, error_message, details) = match &self {
            VaultError::Database(e) => {
                tracing::error!("Database error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Database error occurred", self.to_string())
            }
            VaultError::SolanaClient(e) => {
                tracing::error!("Solana client error: {}", e);
                (StatusCode::BAD_GATEWAY, "Blockchain communication error", self.to_string())
            }
            VaultError::VaultNotFound(_) => {
                tracing::warn!("Vault not found: {}", self);
                (StatusCode::NOT_FOUND, "Vault not found", self.to_string())
            }
            VaultError::InsufficientBalance(_, _) => {
                tracing::warn!("Insufficient balance: {}", self);
                (StatusCode::BAD_REQUEST, "Insufficient balance", self.to_string())
            }
            VaultError::InvalidAmount(_) => {
                tracing::warn!("Invalid amount: {}", self);
                (StatusCode::BAD_REQUEST, "Invalid amount", self.to_string())
            }
            VaultError::TransactionFailed(_) => {
                tracing::error!("Transaction failed: {}", self);
                (StatusCode::BAD_REQUEST, "Transaction failed", self.to_string())
            }
            VaultError::Config(_) => {
                tracing::error!("Configuration error: {}", self);
                (StatusCode::INTERNAL_SERVER_ERROR, "Configuration error", self.to_string())
            }
            VaultError::Internal(_) => {
                tracing::error!("Internal error: {}", self);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error", self.to_string())
            }
            VaultError::UserSignatureRequired(_) => {
                tracing::warn!("User signature required: {}", self);
                (StatusCode::BAD_REQUEST, "User wallet signature required", self.to_string())
            }
        };

        let body = Json(json!({
            "error": error_message,
            "details": details,
        }));

        (status, body).into_response()
    }
}

impl From<anchor_client::ClientError> for VaultError {
    fn from(error: anchor_client::ClientError) -> Self {
        VaultError::SolanaClient(error.to_string())
    }
}

impl From<sqlx::Error> for VaultError {
    fn from(error: sqlx::Error) -> Self {
        VaultError::Database(error.to_string())
    }
}

pub type Result<T> = std::result::Result<T, VaultError>;

