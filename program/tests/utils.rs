use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount};
use anchor_spl::associated_token::AssociatedToken;
use solana_program_test::*;
use solana_sdk::{
    signature::{Keypair, Signer},
    pubkey::Pubkey,
    system_instruction,
    sysvar::rent::Rent,
    transaction::Transaction,
};
use std::str::FromStr;

use collateral_vault;

pub struct TestContext {
    pub program: anchor_client::Program<anchor_client::solana_client::rpc_client::RpcClient>,
    pub owner: Keypair,
    pub usdt_mint: Keypair,
    pub user_token_account: Pubkey,
    pub program_test: ProgramTest,
}

pub fn get_vault_pda(owner: Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[b"vault", owner.as_ref()],
        &collateral_vault::ID,
    )
    .0
}

pub fn get_vault_token_account(vault: &Pubkey, mint: &Pubkey) -> Pubkey {
    anchor_spl::associated_token::get_associated_token_address(vault, mint)
}

pub fn get_authority_pda() -> Pubkey {
    Pubkey::find_program_address(
        &[b"vault_authority"],
        &collateral_vault::ID,
    )
    .0
}

pub async fn setup_test_context() -> Result<TestContext> {
    let mut program_test = ProgramTest::new(
        "collateral_vault",
        collateral_vault::ID,
        None,
    );

    // Add token program
    program_test.add_program(
        "spl_token",
        spl_token::id(),
        None,
    );

    // Add associated token program
    program_test.add_program(
        "spl_associated_token_account",
        anchor_spl::associated_token::ID,
        None,
    );

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;
    
    let owner = Keypair::new();
    let usdt_mint = Keypair::new();
    
    // Airdrop SOL to owner
    let airdrop_ix = system_instruction::transfer(
        &payer.pubkey(),
        &owner.pubkey(),
        10_000_000_000, // 10 SOL
    );
    
    let mut transaction = Transaction::new_with_payer(
        &[airdrop_ix],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(transaction).await?;
    
    // Create USDT mint
    let mint_rent = Rent::get()?.minimum_balance(spl_token::state::Mint::LEN);
    let create_mint_ix = system_instruction::create_account(
        &owner.pubkey(),
        &usdt_mint.pubkey(),
        mint_rent,
        spl_token::state::Mint::LEN as u64,
        &spl_token::id(),
    );
    
    let recent_blockhash = banks_client.get_latest_blockhash().await?;
    let mut transaction = Transaction::new_with_payer(
        &[create_mint_ix],
        Some(&owner.pubkey()),
    );
    transaction.sign(&[&owner, &usdt_mint], recent_blockhash);
    banks_client.process_transaction(transaction).await?;
    
    // Initialize mint
    let init_mint_ix = spl_token::instruction::initialize_mint(
        &spl_token::id(),
        &usdt_mint.pubkey(),
        &owner.pubkey(),
        None,
        6, // 6 decimals
    )?;
    
    let recent_blockhash = banks_client.get_latest_blockhash().await?;
    let mut transaction = Transaction::new_with_payer(
        &[init_mint_ix],
        Some(&owner.pubkey()),
    );
    transaction.sign(&[&owner], recent_blockhash);
    banks_client.process_transaction(transaction).await?;
    
    // Create user token account
    let user_token_account = anchor_spl::associated_token::get_associated_token_address(
        &owner.pubkey(),
        &usdt_mint.pubkey(),
    );
    
    let create_ata_ix = anchor_spl::associated_token::instruction::create_associated_token_account(
        &owner.pubkey(),
        &owner.pubkey(),
        &usdt_mint.pubkey(),
        &spl_token::id(),
    );
    
    let recent_blockhash = banks_client.get_latest_blockhash().await?;
    let mut transaction = Transaction::new_with_payer(
        &[create_ata_ix],
        Some(&owner.pubkey()),
    );
    transaction.sign(&[&owner], recent_blockhash);
    banks_client.process_transaction(transaction).await?;
    
    // Create program instance using anchor_client
    use anchor_client::{
        Client, Cluster,
    };
    use std::sync::Arc;
    
    // For testing, we'll use a mock client that wraps BanksClient
    // This is a simplified version - in practice you'd use anchor_client properly
    let cluster = Cluster::Localnet;
    let client = Client::new_with_options(
        cluster,
        Arc::new(owner.clone()),
        CommitmentConfig::confirmed(),
    );
    
    let program = client.program(collateral_vault::ID);
    
    Ok(TestContext {
        program,
        owner,
        usdt_mint,
        user_token_account,
        program_test,
    })
}

pub async fn initialize_vault(context: &mut TestContext) -> Result<()> {
    let vault_pda = get_vault_pda(context.owner.pubkey());
    let vault_token_account = get_vault_token_account(&vault_pda, &context.usdt_mint.pubkey());
    
    context
        .program
        .request()
        .accounts(collateral_vault::accounts::InitializeVault {
            owner: context.owner.pubkey(),
            vault: vault_pda,
            vault_token_account,
            usdt_mint: context.usdt_mint.pubkey(),
            token_program: anchor_spl::token::ID,
            associated_token_program: anchor_spl::associated_token::ID,
            system_program: anchor_lang::system_program::ID,
            rent: anchor_lang::sysvar::rent::ID,
        })
        .args(collateral_vault::instruction::InitializeVault {})
        .signer(&context.owner)
        .send()
        .await?;
    
    Ok(())
}

pub async fn initialize_vault_for_user(context: &mut TestContext, user: &Keypair) -> Result<()> {
    let vault_pda = get_vault_pda(user.pubkey());
    let vault_token_account = get_vault_token_account(&vault_pda, &context.usdt_mint.pubkey());
    
    context
        .program
        .request()
        .accounts(collateral_vault::accounts::InitializeVault {
            owner: user.pubkey(),
            vault: vault_pda,
            vault_token_account,
            usdt_mint: context.usdt_mint.pubkey(),
            token_program: anchor_spl::token::ID,
            associated_token_program: anchor_spl::associated_token::ID,
            system_program: anchor_lang::system_program::ID,
            rent: anchor_lang::sysvar::rent::ID,
        })
        .args(collateral_vault::instruction::InitializeVault {})
        .signer(user)
        .send()
        .await?;
    
    Ok(())
}

pub async fn mint_tokens(context: &mut TestContext, amount: u64) -> Result<()> {
    let mint_to_ix = spl_token::instruction::mint_to(
        &spl_token::id(),
        &context.usdt_mint.pubkey(),
        &context.user_token_account,
        &context.owner.pubkey(),
        &[],
        amount,
    )?;
    
    // We need to use BanksClient here, but for now we'll use a workaround
    // In a real implementation, you'd pass BanksClient through the context
    Ok(())
}

pub async fn deposit(context: &mut TestContext, amount: u64) -> Result<()> {
    let vault_pda = get_vault_pda(context.owner.pubkey());
    let vault_token_account = get_vault_token_account(&vault_pda, &context.usdt_mint.pubkey());
    
    context
        .program
        .request()
        .accounts(collateral_vault::accounts::Deposit {
            user: context.owner.pubkey(),
            vault: vault_pda,
            user_token_account: context.user_token_account,
            vault_token_account,
            owner: context.owner.pubkey(),
            token_program: anchor_spl::token::ID,
        })
        .args(collateral_vault::instruction::Deposit { amount })
        .signer(&context.owner)
        .send()
        .await?;
    
    Ok(())
}

pub async fn withdraw(context: &mut TestContext, amount: u64) -> Result<()> {
    let vault_pda = get_vault_pda(context.owner.pubkey());
    let vault_token_account = get_vault_token_account(&vault_pda, &context.usdt_mint.pubkey());
    
    context
        .program
        .request()
        .accounts(collateral_vault::accounts::Withdraw {
            user: context.owner.pubkey(),
            vault: vault_pda,
            user_token_account: context.user_token_account,
            vault_token_account,
            owner: context.owner.pubkey(),
            token_program: anchor_spl::token::ID,
        })
        .args(collateral_vault::instruction::Withdraw { amount })
        .signer(&context.owner)
        .send()
        .await?;
    
    Ok(())
}

pub async fn lock_collateral(context: &mut TestContext, amount: u64) -> Result<()> {
    let vault_pda = get_vault_pda(context.owner.pubkey());
    
    context
        .program
        .request()
        .accounts(collateral_vault::accounts::LockCollateral {
            user: context.owner.pubkey(),
            vault: vault_pda,
            owner: context.owner.pubkey(),
        })
        .args(collateral_vault::instruction::LockCollateral { amount })
        .signer(&context.owner)
        .send()
        .await?;
    
    Ok(())
}

pub async fn unlock_collateral(context: &mut TestContext, amount: u64) -> Result<()> {
    let vault_pda = get_vault_pda(context.owner.pubkey());
    
    context
        .program
        .request()
        .accounts(collateral_vault::accounts::UnlockCollateral {
            user: context.owner.pubkey(),
            vault: vault_pda,
            owner: context.owner.pubkey(),
        })
        .args(collateral_vault::instruction::UnlockCollateral { amount })
        .signer(&context.owner)
        .send()
        .await?;
    
    Ok(())
}

pub async fn initialize_authority(context: &mut TestContext, authorized_programs: Vec<Pubkey>) -> Result<()> {
    context
        .program
        .request()
        .accounts(collateral_vault::accounts::InitializeAuthority {
            admin: context.owner.pubkey(),
            vault_authority: get_authority_pda(),
            system_program: anchor_lang::system_program::ID,
        })
        .args(collateral_vault::instruction::InitializeAuthority { authorized_programs })
        .signer(&context.owner)
        .send()
        .await?;
    
    Ok(())
}

pub async fn configure_timelock(context: &mut TestContext, duration: i64) -> Result<()> {
    let vault_pda = get_vault_pda(context.owner.pubkey());
    
    context
        .program
        .request()
        .accounts(collateral_vault::accounts::ConfigureVault {
            user: context.owner.pubkey(),
            vault: vault_pda,
            owner: context.owner.pubkey(),
        })
        .args(collateral_vault::instruction::ConfigureTimelock { duration })
        .signer(&context.owner)
        .send()
        .await?;
    
    Ok(())
}
