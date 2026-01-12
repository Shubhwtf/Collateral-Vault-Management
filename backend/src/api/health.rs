use axum::{
    Json,
    response::IntoResponse,
    extract::State,
};
use serde::Serialize;
use serde_json::json;
use std::sync::Arc;

use crate::AppState;
use crate::error::Result;

pub async fn health_check() -> impl IntoResponse {
    Json(json!({
        "status": "healthy",
        "service": "Collateral Vault Management System",
        "version": "0.1.0"
    }))
}

#[derive(Debug, Serialize)]
pub struct PublicConfigResponse {
    pub program_id: String,
    pub usdt_mint: String,
    pub solana_rpc_url: String,
}

// exposing these publicly so the frontend doesn't need separate env vars
// program_id and mint are public anyway, RPC endpoint is fine to share
pub async fn public_config(State(state): State<Arc<AppState>>) -> Result<Json<PublicConfigResponse>> {
    let solana = state.vault_manager.solana_client();

    Ok(Json(PublicConfigResponse {
        program_id: solana.program_id.to_string(),
        usdt_mint: solana.usdt_mint.to_string(),
        solana_rpc_url: solana.rpc.url().to_string(),
    }))
}

