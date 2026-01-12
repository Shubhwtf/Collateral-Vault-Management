use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub solana_rpc_url: String,
    pub solana_ws_url: String,
    pub program_id: String,
    pub payer_keypair_path: String,
    pub usdt_mint: String,
    pub database_url: String,
    pub transaction_timeout_seconds: u64,
    pub max_retry_attempts: u32,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        // treating empty DATABASE_URL as unset because docker-compose was setting it to ""
        let mut database_url = env::var("DATABASE_URL").ok().filter(|v| !v.trim().is_empty());

        // fallback to loading backend/.env explicitly in case working directory isn't set correctly
        if database_url.is_none() {
            let env_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(".env");
            let _ = dotenvy::from_path_override(&env_path);
            database_url = env::var("DATABASE_URL").ok().filter(|v| !v.trim().is_empty());
        }

        Ok(Config {
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()?,
            solana_rpc_url: env::var("SOLANA_RPC_URL")?,
            solana_ws_url: env::var("SOLANA_WS_URL")?,
            program_id: env::var("PROGRAM_ID")?,
            payer_keypair_path: env::var("PAYER_KEYPAIR_PATH")?,
            usdt_mint: env::var("USDT_MINT")?,
            database_url: database_url.ok_or_else(|| anyhow::anyhow!("DATABASE_URL is not set"))?,
            transaction_timeout_seconds: env::var("TRANSACTION_TIMEOUT_SECONDS")
                .unwrap_or_else(|_| "30".to_string())
                .parse()?,
            max_retry_attempts: env::var("MAX_RETRY_ATTEMPTS")
                .unwrap_or_else(|_| "3".to_string())
                .parse()?,
        })
    }
}

