use anchor_client::{Client, Cluster, Program};
use anchor_client::solana_sdk::{
    pubkey::Pubkey,
    signature::{read_keypair_file, Keypair, Signer},
    commitment_config::CommitmentConfig,
};
use anchor_client::ClientError;
use solana_client::rpc_client::RpcClient;
use anyhow::{anyhow, Result};
use std::sync::Arc;
use std::str::FromStr;

use crate::config::Config;

pub struct SolanaClient {
    pub client: Client<Arc<Keypair>>,
    pub program_id: Pubkey,
    pub payer: Arc<Keypair>,
    pub usdt_mint: Pubkey,
    // keeping a separate RpcClient because anchor_client doesn't expose all the methods we need
    pub rpc: RpcClient,
}

impl SolanaClient {
    pub fn new(config: &Config) -> Result<Self> {
        let payer = read_keypair_file(&config.payer_keypair_path)
            .map_err(|e| anyhow!("Failed to read keypair file: {}", e))?;
        let payer = Arc::new(payer);

        tracing::info!("Solana RPC URL: {}", config.solana_rpc_url);
        tracing::info!("Solana WS URL: {}", config.solana_ws_url);

        let cluster = Cluster::Custom(
            config.solana_rpc_url.clone(),
            config.solana_ws_url.clone(),
        );

        let program_id = Pubkey::from_str(&config.program_id)
            .map_err(|e| anyhow!("Invalid program ID: {}", e))?;
        let usdt_mint = Pubkey::from_str(&config.usdt_mint)
            .map_err(|e| anyhow!("Invalid USDT mint: {}", e))?;

        let client = Client::new_with_options(
            cluster,
            payer.clone(),
            CommitmentConfig::confirmed(),
        );

        let rpc = RpcClient::new_with_commitment(
            config.solana_rpc_url.clone(),
            CommitmentConfig::confirmed(),
        );

        Ok(Self {
            client,
            program_id,
            payer,
            usdt_mint,
            rpc,
        })
    }

    pub fn program(&self) -> Result<Program<Arc<Keypair>>, ClientError> {
        self.client.program(self.program_id)
    }

    pub fn payer_pubkey(&self) -> Pubkey {
        self.payer.pubkey()
    }

    pub fn derive_vault_pda(&self, user: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[b"vault", user.as_ref()],
            &self.program_id,
        )
    }

    pub fn derive_authority_pda(&self) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[b"vault_authority"],
            &self.program_id,
        )
    }
}