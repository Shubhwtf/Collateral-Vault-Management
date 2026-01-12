use totp_rs::{Algorithm, TOTP};
use qrcode::{QrCode, render::svg};
use rand::Rng;
use sqlx::PgPool;
use base32::Alphabet;

use crate::error::{Result, VaultError};

pub struct MfaService {
    db_pool: PgPool,
    issuer: String,
}

impl MfaService {
    pub fn new(db_pool: PgPool, issuer: String) -> Self {
        Self { db_pool, issuer }
    }

    pub fn generate_secret(&self) -> String {
        // using Base32 charset because TOTP apps expect this format
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
        let mut rng = rand::rng();
        
        let secret: String = (0..32)
            .map(|_| {
                let idx = rng.random_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect();
        
        secret
    }

    pub fn generate_backup_codes(&self) -> Vec<String> {
        let mut rng = rand::rng();
        // 10 codes in XXXX-XXXX format - easy to read and type
        (0..10)
            .map(|_| {
                format!(
                    "{:04X}-{:04X}",
                    rng.random_range(0..0x10000),
                    rng.random_range(0..0x10000)
                )
            })
            .collect()
    }

    pub fn generate_qr_code(&self, user_pubkey: &str, secret: &str) -> Result<String> {
        let secret_bytes = base32::decode(Alphabet::RFC4648 { padding: false }, secret)
            .ok_or_else(|| VaultError::Internal("Failed to decode Base32 secret".to_string()))?;

        let _totp = TOTP::new(
            Algorithm::SHA1,
            6,
            1,
            30,
            secret_bytes,
        )
        .map_err(|e| VaultError::Internal(format!("Failed to create TOTP: {}", e)))?;

        // building otpauth URL manually for more control over parameters
        let uri = format!(
            "otpauth://totp/{}:{}?secret={}&issuer={}&algorithm=SHA1&digits=6&period=30",
            urlencoding::encode(&self.issuer),
            urlencoding::encode(user_pubkey),
            secret,
            urlencoding::encode(&self.issuer)
        );
        
        let qr = QrCode::new(uri.as_bytes())
            .map_err(|e| VaultError::Internal(format!("Failed to create QR code: {}", e)))?;
        
        let svg = qr.render::<svg::Color>()
            .min_dimensions(200, 200)
            .build();
        
        Ok(svg)
    }

    pub fn verify_totp(&self, secret: &str, code: &str) -> Result<bool> {
        let secret_bytes = base32::decode(Alphabet::RFC4648 { padding: false }, secret)
            .ok_or_else(|| VaultError::Internal("Failed to decode Base32 secret".to_string()))?;

        let totp = TOTP::new(
            Algorithm::SHA1,
            6,
            1,
            30,
            secret_bytes,
        )
        .map_err(|e| VaultError::Internal(format!("Failed to create TOTP: {}", e)))?;

        Ok(totp.check_current(code).unwrap_or(false))
    }

    pub async fn enable_mfa(&self, vault_address: &str, secret: &str, ip: Option<String>, user_agent: Option<String>) -> Result<Vec<String>> {
        let backup_codes = self.generate_backup_codes();
        
        sqlx::query(
            r#"
            UPDATE public.vaults 
            SET mfa_enabled = TRUE,
                mfa_secret = $1,
                mfa_backup_codes = $2,
                updated_at = NOW()
            WHERE vault_address = $3
            "#
        )
        .bind(secret)
        .bind(&backup_codes)
        .bind(vault_address)
        .execute(&self.db_pool)
        .await
        .map_err(|e| VaultError::Database(e.to_string()))?;

        self.log_mfa_action(vault_address, "enable", ip, user_agent, true).await?;

        Ok(backup_codes)
    }

    pub async fn disable_mfa(&self, vault_address: &str, ip: Option<String>, user_agent: Option<String>) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE public.vaults 
            SET mfa_enabled = FALSE,
                mfa_secret = NULL,
                mfa_backup_codes = NULL,
                updated_at = NOW()
            WHERE vault_address = $1
            "#
        )
        .bind(vault_address)
        .execute(&self.db_pool)
        .await
        .map_err(|e| VaultError::Database(e.to_string()))?;

        self.log_mfa_action(vault_address, "disable", ip, user_agent, true).await?;

        Ok(())
    }

    pub async fn verify_mfa(
        &self,
        vault_address: &str,
        code: &str,
        ip: Option<String>,
        user_agent: Option<String>,
    ) -> Result<bool> {
        let record: Option<(bool, Option<String>, Option<Vec<String>>)> = sqlx::query_as(
            "SELECT mfa_enabled, mfa_secret, mfa_backup_codes FROM public.vaults WHERE vault_address = $1"
        )
        .bind(vault_address)
        .fetch_optional(&self.db_pool)
        .await
        .map_err(|e| VaultError::Database(e.to_string()))?;

        let (mfa_enabled, secret_opt, backup_codes_opt) = record
            .ok_or_else(|| VaultError::VaultNotFound(vault_address.to_string()))?;

        if !mfa_enabled {
            return Ok(true);
        }

        let secret = secret_opt
            .ok_or_else(|| VaultError::Internal("MFA enabled but no secret".to_string()))?;

        // try TOTP first since it's the primary method
        let totp_valid = self.verify_totp(&secret, code)?;
        
        if totp_valid {
            self.log_mfa_action(vault_address, "verify_success", ip, user_agent, true).await?;
            return Ok(true);
        }

        // fallback to backup codes if TOTP fails
        if let Some(backup_codes) = backup_codes_opt {
            if backup_codes.contains(&code.to_string()) {
                // remove used backup code so it can't be reused
                let remaining: Vec<String> = backup_codes
                    .into_iter()
                    .filter(|c| c != code)
                    .collect();

                sqlx::query(
                    "UPDATE public.vaults SET mfa_backup_codes = $1 WHERE vault_address = $2"
                )
                .bind(&remaining)
                .bind(vault_address)
                .execute(&self.db_pool)
                .await
                .map_err(|e| VaultError::Database(e.to_string()))?;

                self.log_mfa_action(vault_address, "verify_success_backup", ip, user_agent, true).await?;
                return Ok(true);
            }
        }

        self.log_mfa_action(vault_address, "verify_failed", ip, user_agent, false).await?;
        Ok(false)
    }

    pub async fn is_mfa_enabled(&self, vault_address: &str) -> Result<bool> {
        let record: Option<(bool,)> = sqlx::query_as(
            "SELECT mfa_enabled FROM public.vaults WHERE vault_address = $1"
        )
        .bind(vault_address)
        .fetch_optional(&self.db_pool)
        .await
        .map_err(|e| VaultError::Database(e.to_string()))?;

        Ok(record.map(|(enabled,)| enabled).unwrap_or(false))
    }

    async fn log_mfa_action(
        &self,
        vault_address: &str,
        action: &str,
        ip: Option<String>,
        user_agent: Option<String>,
        success: bool,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO public.mfa_audit_log (vault_address, action, ip_address, user_agent, success)
            VALUES ($1, $2, $3, $4, $5)
            "#
        )
        .bind(vault_address)
        .bind(action)
        .bind(ip)
        .bind(user_agent)
        .bind(success)
        .execute(&self.db_pool)
        .await
        .map_err(|e| VaultError::Database(e.to_string()))?;

        Ok(())
    }

    pub async fn get_audit_logs(&self, vault_address: &str, limit: i64) -> Result<Vec<MfaAuditLog>> {
        let logs = sqlx::query_as::<_, MfaAuditLog>(
            r#"
            SELECT * FROM public.mfa_audit_log 
            WHERE vault_address = $1
            ORDER BY created_at DESC
            LIMIT $2
            "#
        )
        .bind(vault_address)
        .bind(limit)
        .fetch_all(&self.db_pool)
        .await
        .map_err(|e| VaultError::Database(e.to_string()))?;

        Ok(logs)
    }
}

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct MfaAuditLog {
    pub id: i32,
    pub vault_address: String,
    pub action: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub success: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
