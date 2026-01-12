use axum::{
    extract::{Path, State},
    Json,
    http::HeaderMap,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::AppState;
use crate::error::{Result, VaultError};

#[derive(Debug, Deserialize)]
pub struct EnableMfaRequest {
    pub vault_address: String,
}

#[derive(Debug, Serialize)]
pub struct EnableMfaResponse {
    pub qr_code_svg: String,
    pub secret: String,
    pub backup_codes: Vec<String>,
}

// two-step process: generate QR first, then verify code before enabling
// this way users can scan and test before we commit to the DB
pub async fn setup_mfa(
    State(state): State<Arc<AppState>>,
    Json(req): Json<EnableMfaRequest>,
) -> Result<Json<EnableMfaResponse>> {
    let mfa_service = &state.mfa_service;
    
    let secret = mfa_service.generate_secret();
    let qr_code_svg = mfa_service.generate_qr_code(&req.vault_address, &secret)?;
    
    // generate codes now so frontend can show them, but don't save to DB yet
    let backup_codes = mfa_service.generate_backup_codes();
    
    Ok(Json(EnableMfaResponse {
        qr_code_svg,
        secret,
        backup_codes,
    }))
}

#[derive(Debug, Deserialize)]
pub struct VerifyMfaSetupRequest {
    pub vault_address: String,
    pub secret: String,
    pub code: String,
}

#[derive(Debug, Serialize)]
pub struct VerifyMfaSetupResponse {
    pub success: bool,
    pub backup_codes: Option<Vec<String>>,
}

pub async fn verify_and_enable_mfa(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<VerifyMfaSetupRequest>,
) -> Result<Json<VerifyMfaSetupResponse>> {
    let mfa_service = &state.mfa_service;
    
    // extract real IP for audit trail - handles proxies and load balancers
    let ip = headers
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.split(',').next())
        .and_then(|s| s.trim().parse::<std::net::IpAddr>().ok())
        .or_else(|| {
            headers
                .get("x-real-ip")
                .and_then(|h| h.to_str().ok())
                .and_then(|s| s.parse::<std::net::IpAddr>().ok())
        })
        .unwrap_or_else(|| "127.0.0.1".parse().unwrap())
        .to_string();
    
    let user_agent = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    
    tracing::info!("MFA enable attempt - IP: {}, User-Agent: {:?}", ip, user_agent);
    
    let valid = mfa_service.verify_totp(&req.secret, &req.code)?;
    
    if !valid {
        return Ok(Json(VerifyMfaSetupResponse {
            success: false,
            backup_codes: None,
        }));
    }
    
    let backup_codes = mfa_service.enable_mfa(&req.vault_address, &req.secret, Some(ip), user_agent).await?;
    
    Ok(Json(VerifyMfaSetupResponse {
        success: true,
        backup_codes: Some(backup_codes),
    }))
}

#[derive(Debug, Deserialize)]
pub struct DisableMfaRequest {
    pub vault_address: String,
    pub code: String,
}

pub async fn disable_mfa(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<DisableMfaRequest>,
) -> Result<Json<serde_json::Value>> {
    let mfa_service = &state.mfa_service;
    
    let ip = headers
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.split(',').next())
        .and_then(|s| s.trim().parse::<std::net::IpAddr>().ok())
        .or_else(|| {
            headers
                .get("x-real-ip")
                .and_then(|h| h.to_str().ok())
                .and_then(|s| s.parse::<std::net::IpAddr>().ok())
        })
        .unwrap_or_else(|| "127.0.0.1".parse().unwrap())
        .to_string();
    
    let user_agent = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    
    // require valid MFA code to disable - prevents attacker from turning off MFA
    let verified = mfa_service.verify_mfa(&req.vault_address, &req.code, Some(ip.clone()), user_agent.clone()).await?;
    
    if !verified {
        return Err(VaultError::InvalidAmount("Invalid MFA code".to_string()));
    }
    
    mfa_service.disable_mfa(&req.vault_address, Some(ip), user_agent).await?;
    
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "MFA disabled successfully"
    })))
}

#[derive(Debug, Deserialize)]
pub struct CheckMfaRequest {
    pub vault_address: String,
    pub code: String,
}

pub async fn check_mfa(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<CheckMfaRequest>,
) -> Result<Json<serde_json::Value>> {
    let mfa_service = &state.mfa_service;
    
    let ip = headers
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.split(',').next())
        .and_then(|s| s.trim().parse::<std::net::IpAddr>().ok())
        .or_else(|| {
            headers
                .get("x-real-ip")
                .and_then(|h| h.to_str().ok())
                .and_then(|s| s.parse::<std::net::IpAddr>().ok())
        })
        .unwrap_or_else(|| "127.0.0.1".parse().unwrap())
        .to_string();
    
    let user_agent = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    
    let verified = mfa_service.verify_mfa(&req.vault_address, &req.code, Some(ip), user_agent).await?;
    
    Ok(Json(serde_json::json!({
        "valid": verified
    })))
}

pub async fn get_mfa_status(
    State(state): State<Arc<AppState>>,
    Path(vault_address): Path<String>,
) -> Result<Json<serde_json::Value>> {
    let enabled = state.mfa_service.is_mfa_enabled(&vault_address).await?;
    
    Ok(Json(serde_json::json!({
        "mfa_enabled": enabled,
        "vault_address": vault_address
    })))
}
