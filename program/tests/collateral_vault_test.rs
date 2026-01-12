use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount};
use anchor_spl::associated_token::AssociatedToken;
use solana_program_test::*;
use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::Transaction,
    pubkey::Pubkey,
    system_instruction,
    sysvar::rent::Rent,
};
use std::str::FromStr;

use collateral_vault::{
    self,
    state::CollateralVault,
    errors::VaultError,
};

declare_id!("pjYYA2y9UL5N4EDd8wKLySDCvb3N6zCoPtFU8WYsnDP");

pub mod utils;
use utils::*;

#[tokio::test]
async fn test_initialize_vault() -> Result<()> {
    let mut context = setup_test_context().await?;
    
    let owner = &context.owner;
    let vault_pda = get_vault_pda(owner.pubkey());
    
    let tx = context
        .program
        .request()
        .accounts(collateral_vault::accounts::InitializeVault {
            owner: owner.pubkey(),
            vault: vault_pda,
            vault_token_account: get_vault_token_account(&vault_pda, &context.usdt_mint.pubkey()),
            usdt_mint: context.usdt_mint.pubkey(),
            token_program: anchor_spl::token::ID,
            associated_token_program: anchor_spl::associated_token::ID,
            system_program: anchor_lang::system_program::ID,
            rent: anchor_lang::sysvar::rent::ID,
        })
        .args(collateral_vault::instruction::InitializeVault {})
        .signer(owner)
        .send()
        .await?;
    
    assert!(tx.is_success());
    
    let vault_account: CollateralVault = context
        .program
        .account(vault_pda)
        .await?;
    
    assert_eq!(vault_account.owner, owner.pubkey());
    assert_eq!(vault_account.total_balance, 0);
    assert_eq!(vault_account.locked_balance, 0);
    assert_eq!(vault_account.available_balance, 0);
    
    Ok(())
}

#[tokio::test]
async fn test_deposit() -> Result<()> {
    let mut context = setup_test_context().await?;
    initialize_vault(&mut context).await?;
    
    let deposit_amount = 1_000_000_000; // 1000 tokens (6 decimals)
    mint_tokens(&mut context, deposit_amount).await?;
    
    let tx = context
        .program
        .request()
        .accounts(collateral_vault::accounts::Deposit {
            user: context.owner.pubkey(),
            vault: get_vault_pda(context.owner.pubkey()),
            user_token_account: context.user_token_account,
            vault_token_account: get_vault_token_account(&get_vault_pda(context.owner.pubkey()), &context.usdt_mint.pubkey()),
            owner: context.owner.pubkey(),
            token_program: anchor_spl::token::ID,
        })
        .args(collateral_vault::instruction::Deposit { amount: deposit_amount })
        .signer(&context.owner)
        .send()
        .await?;
    
    assert!(tx.is_success());
    
    let vault_account: CollateralVault = context
        .program
        .account(get_vault_pda(context.owner.pubkey()))
        .await?;
    
    assert_eq!(vault_account.total_balance, deposit_amount);
    assert_eq!(vault_account.available_balance, deposit_amount);
    assert_eq!(vault_account.total_deposited, deposit_amount);
    
    Ok(())
}

#[tokio::test]
async fn test_deposit_invalid_amount() -> Result<()> {
    let mut context = setup_test_context().await?;
    initialize_vault(&mut context).await?;
    
    let result = context
        .program
        .request()
        .accounts(collateral_vault::accounts::Deposit {
            user: context.owner.pubkey(),
            vault: get_vault_pda(context.owner.pubkey()),
            user_token_account: context.user_token_account,
            vault_token_account: get_vault_token_account(&get_vault_pda(context.owner.pubkey()), &context.usdt_mint.pubkey()),
            owner: context.owner.pubkey(),
            token_program: anchor_spl::token::ID,
        })
        .args(collateral_vault::instruction::Deposit { amount: 0 })
        .signer(&context.owner)
        .send()
        .await;
    
    assert!(result.is_err());
    
    Ok(())
}

#[tokio::test]
async fn test_withdraw() -> Result<()> {
    let mut context = setup_test_context().await?;
    initialize_vault(&mut context).await?;
    
    let deposit_amount = 1_000_000_000;
    mint_tokens(&mut context, deposit_amount).await?;
    deposit(&mut context, deposit_amount).await?;
    
    let withdraw_amount = 500_000_000;
    
    let tx = context
        .program
        .request()
        .accounts(collateral_vault::accounts::Withdraw {
            user: context.owner.pubkey(),
            vault: get_vault_pda(context.owner.pubkey()),
            user_token_account: context.user_token_account,
            vault_token_account: get_vault_token_account(&get_vault_pda(context.owner.pubkey()), &context.usdt_mint.pubkey()),
            owner: context.owner.pubkey(),
            token_program: anchor_spl::token::ID,
        })
        .args(collateral_vault::instruction::Withdraw { amount: withdraw_amount })
        .signer(&context.owner)
        .send()
        .await?;
    
    assert!(tx.is_success());
    
    let vault_account: CollateralVault = context
        .program
        .account(get_vault_pda(context.owner.pubkey()))
        .await?;
    
    assert_eq!(vault_account.total_balance, deposit_amount - withdraw_amount);
    assert_eq!(vault_account.available_balance, deposit_amount - withdraw_amount);
    assert_eq!(vault_account.total_withdrawn, withdraw_amount);
    
    Ok(())
}

#[tokio::test]
async fn test_withdraw_insufficient_balance() -> Result<()> {
    let mut context = setup_test_context().await?;
    initialize_vault(&mut context).await?;
    
    let deposit_amount = 1_000_000_000;
    mint_tokens(&mut context, deposit_amount).await?;
    deposit(&mut context, deposit_amount).await?;
    
    let result = context
        .program
        .request()
        .accounts(collateral_vault::accounts::Withdraw {
            user: context.owner.pubkey(),
            vault: get_vault_pda(context.owner.pubkey()),
            user_token_account: context.user_token_account,
            vault_token_account: get_vault_token_account(&get_vault_pda(context.owner.pubkey()), &context.usdt_mint.pubkey()),
            owner: context.owner.pubkey(),
            token_program: anchor_spl::token::ID,
        })
        .args(collateral_vault::instruction::Withdraw { amount: deposit_amount + 1 })
        .signer(&context.owner)
        .send()
        .await;
    
    assert!(result.is_err());
    
    Ok(())
}

#[tokio::test]
async fn test_lock_collateral() -> Result<()> {
    let mut context = setup_test_context().await?;
    initialize_vault(&mut context).await?;
    
    let deposit_amount = 1_000_000_000;
    mint_tokens(&mut context, deposit_amount).await?;
    deposit(&mut context, deposit_amount).await?;
    
    let lock_amount = 500_000_000;
    
    let tx = context
        .program
        .request()
        .accounts(collateral_vault::accounts::LockCollateral {
            user: context.owner.pubkey(),
            vault: get_vault_pda(context.owner.pubkey()),
            owner: context.owner.pubkey(),
        })
        .args(collateral_vault::instruction::LockCollateral { amount: lock_amount })
        .signer(&context.owner)
        .send()
        .await?;
    
    assert!(tx.is_success());
    
    let vault_account: CollateralVault = context
        .program
        .account(get_vault_pda(context.owner.pubkey()))
        .await?;
    
    assert_eq!(vault_account.locked_balance, lock_amount);
    assert_eq!(vault_account.available_balance, deposit_amount - lock_amount);
    assert_eq!(vault_account.total_balance, deposit_amount);
    
    Ok(())
}

#[tokio::test]
async fn test_unlock_collateral() -> Result<()> {
    let mut context = setup_test_context().await?;
    initialize_vault(&mut context).await?;
    
    let deposit_amount = 1_000_000_000;
    mint_tokens(&mut context, deposit_amount).await?;
    deposit(&mut context, deposit_amount).await?;
    
    let lock_amount = 500_000_000;
    lock_collateral(&mut context, lock_amount).await?;
    
    let unlock_amount = 300_000_000;
    
    let tx = context
        .program
        .request()
        .accounts(collateral_vault::accounts::UnlockCollateral {
            user: context.owner.pubkey(),
            vault: get_vault_pda(context.owner.pubkey()),
            owner: context.owner.pubkey(),
        })
        .args(collateral_vault::instruction::UnlockCollateral { amount: unlock_amount })
        .signer(&context.owner)
        .send()
        .await?;
    
    assert!(tx.is_success());
    
    let vault_account: CollateralVault = context
        .program
        .account(get_vault_pda(context.owner.pubkey()))
        .await?;
    
    assert_eq!(vault_account.locked_balance, lock_amount - unlock_amount);
    assert_eq!(vault_account.available_balance, deposit_amount - lock_amount + unlock_amount);
    
    Ok(())
}

#[tokio::test]
async fn test_transfer_collateral() -> Result<()> {
    let mut context = setup_test_context().await?;
    initialize_vault(&mut context).await?;
    
    let deposit_amount = 1_000_000_000;
    mint_tokens(&mut context, deposit_amount).await?;
    deposit(&mut context, deposit_amount).await?;
    
    let recipient = Keypair::new();
    let recipient_vault_pda = get_vault_pda(recipient.pubkey());
    
    // Initialize recipient vault
    initialize_vault_for_user(&mut context, &recipient).await?;
    
    let transfer_amount = 300_000_000;
    
    let tx = context
        .program
        .request()
        .accounts(collateral_vault::accounts::TransferCollateral {
            from_user: context.owner.pubkey(),
            from_vault: get_vault_pda(context.owner.pubkey()),
            to_vault: recipient_vault_pda,
            from_vault_token_account: get_vault_token_account(&get_vault_pda(context.owner.pubkey()), &context.usdt_mint.pubkey()),
            to_vault_token_account: get_vault_token_account(&recipient_vault_pda, &context.usdt_mint.pubkey()),
            owner: context.owner.pubkey(),
            token_program: anchor_spl::token::ID,
        })
        .args(collateral_vault::instruction::TransferCollateral { amount: transfer_amount })
        .signer(&context.owner)
        .send()
        .await?;
    
    assert!(tx.is_success());
    
    let from_vault: CollateralVault = context
        .program
        .account(get_vault_pda(context.owner.pubkey()))
        .await?;
    
    let to_vault: CollateralVault = context
        .program
        .account(recipient_vault_pda)
        .await?;
    
    assert_eq!(from_vault.total_balance, deposit_amount - transfer_amount);
    assert_eq!(to_vault.total_balance, transfer_amount);
    
    Ok(())
}

#[tokio::test]
async fn test_batch_deposit() -> Result<()> {
    let mut context = setup_test_context().await?;
    initialize_vault(&mut context).await?;
    
    let amounts = vec![100_000_000, 200_000_000, 300_000_000];
    let total_amount: u64 = amounts.iter().sum();
    
    mint_tokens(&mut context, total_amount).await?;
    
    let tx = context
        .program
        .request()
        .accounts(collateral_vault::accounts::BatchDeposit {
            user: context.owner.pubkey(),
            vault: get_vault_pda(context.owner.pubkey()),
            user_token_account: context.user_token_account,
            vault_token_account: get_vault_token_account(&get_vault_pda(context.owner.pubkey()), &context.usdt_mint.pubkey()),
            owner: context.owner.pubkey(),
            token_program: anchor_spl::token::ID,
        })
        .args(collateral_vault::instruction::BatchDeposit { amounts: amounts.clone() })
        .signer(&context.owner)
        .send()
        .await?;
    
    assert!(tx.is_success());
    
    let vault_account: CollateralVault = context
        .program
        .account(get_vault_pda(context.owner.pubkey()))
        .await?;
    
    assert_eq!(vault_account.total_balance, total_amount);
    assert_eq!(vault_account.total_deposited, total_amount);
    
    Ok(())
}

#[tokio::test]
async fn test_batch_withdraw() -> Result<()> {
    let mut context = setup_test_context().await?;
    initialize_vault(&mut context).await?;
    
    let deposit_amount = 1_000_000_000;
    mint_tokens(&mut context, deposit_amount).await?;
    deposit(&mut context, deposit_amount).await?;
    
    let amounts = vec![100_000_000, 200_000_000, 300_000_000];
    let total_withdraw: u64 = amounts.iter().sum();
    
    let tx = context
        .program
        .request()
        .accounts(collateral_vault::accounts::BatchWithdraw {
            user: context.owner.pubkey(),
            vault: get_vault_pda(context.owner.pubkey()),
            user_token_account: context.user_token_account,
            vault_token_account: get_vault_token_account(&get_vault_pda(context.owner.pubkey()), &context.usdt_mint.pubkey()),
            owner: context.owner.pubkey(),
            token_program: anchor_spl::token::ID,
        })
        .args(collateral_vault::instruction::BatchWithdraw { amounts: amounts.clone() })
        .signer(&context.owner)
        .send()
        .await?;
    
    assert!(tx.is_success());
    
    let vault_account: CollateralVault = context
        .program
        .account(get_vault_pda(context.owner.pubkey()))
        .await?;
    
    assert_eq!(vault_account.total_balance, deposit_amount - total_withdraw);
    assert_eq!(vault_account.total_withdrawn, total_withdraw);
    
    Ok(())
}

#[tokio::test]
async fn test_initialize_authority() -> Result<()> {
    let mut context = setup_test_context().await?;
    
    let authorized_programs = vec![
        Pubkey::from_str("11111111111111111111111111111111").unwrap(),
        Pubkey::from_str("22222222222222222222222222222222").unwrap(),
    ];
    
    let tx = context
        .program
        .request()
        .accounts(collateral_vault::accounts::InitializeAuthority {
            admin: context.owner.pubkey(),
            vault_authority: get_authority_pda(),
            system_program: anchor_lang::system_program::ID,
        })
        .args(collateral_vault::instruction::InitializeAuthority { authorized_programs: authorized_programs.clone() })
        .signer(&context.owner)
        .send()
        .await?;
    
    assert!(tx.is_success());
    
    Ok(())
}

#[tokio::test]
async fn test_add_authorized_program() -> Result<()> {
    let mut context = setup_test_context().await?;
    initialize_authority(&mut context, vec![]).await?;
    
    let new_program = Pubkey::from_str("33333333333333333333333333333333").unwrap();
    
    let tx = context
        .program
        .request()
        .accounts(collateral_vault::accounts::UpdateAuthority {
            admin: context.owner.pubkey(),
            vault_authority: get_authority_pda(),
        })
        .args(collateral_vault::instruction::AddAuthorizedProgram { program: new_program })
        .signer(&context.owner)
        .send()
        .await?;
    
    assert!(tx.is_success());
    
    Ok(())
}

#[tokio::test]
async fn test_configure_multisig() -> Result<()> {
    let mut context = setup_test_context().await?;
    initialize_vault(&mut context).await?;
    
    let signers = vec![
        Pubkey::from_str("11111111111111111111111111111111").unwrap(),
        Pubkey::from_str("22222222222222222222222222222222").unwrap(),
        Pubkey::from_str("33333333333333333333333333333333").unwrap(),
    ];
    let threshold = 2;
    
    let tx = context
        .program
        .request()
        .accounts(collateral_vault::accounts::ConfigureMultiSig {
            user: context.owner.pubkey(),
            vault: get_vault_pda(context.owner.pubkey()),
            owner: context.owner.pubkey(),
        })
        .args(collateral_vault::instruction::ConfigureMultisig { threshold, signers: signers.clone() })
        .signer(&context.owner)
        .send()
        .await?;
    
    assert!(tx.is_success());
    
    let vault_account: CollateralVault = context
        .program
        .account(get_vault_pda(context.owner.pubkey()))
        .await?;
    
    assert_eq!(vault_account.multisig_threshold, threshold);
    assert_eq!(vault_account.authorized_signers.len(), signers.len());
    
    Ok(())
}

#[tokio::test]
async fn test_add_delegate() -> Result<()> {
    let mut context = setup_test_context().await?;
    initialize_vault(&mut context).await?;
    
    let delegate = Pubkey::from_str("44444444444444444444444444444444").unwrap();
    
    let tx = context
        .program
        .request()
        .accounts(collateral_vault::accounts::ManageDelegate {
            user: context.owner.pubkey(),
            vault: get_vault_pda(context.owner.pubkey()),
            owner: context.owner.pubkey(),
        })
        .args(collateral_vault::instruction::AddDelegate { user: delegate })
        .signer(&context.owner)
        .send()
        .await?;
    
    assert!(tx.is_success());
    
    let vault_account: CollateralVault = context
        .program
        .account(get_vault_pda(context.owner.pubkey()))
        .await?;
    
    assert!(vault_account.delegated_users.contains(&delegate));
    
    Ok(())
}

#[tokio::test]
async fn test_add_to_whitelist() -> Result<()> {
    let mut context = setup_test_context().await?;
    initialize_vault(&mut context).await?;
    
    let whitelist_address = Pubkey::from_str("55555555555555555555555555555555").unwrap();
    
    let tx = context
        .program
        .request()
        .accounts(collateral_vault::accounts::ManageWhitelist {
            user: context.owner.pubkey(),
            vault: get_vault_pda(context.owner.pubkey()),
            owner: context.owner.pubkey(),
        })
        .args(collateral_vault::instruction::AddToWhitelist { address: whitelist_address })
        .signer(&context.owner)
        .send()
        .await?;
    
    assert!(tx.is_success());
    
    let vault_account: CollateralVault = context
        .program
        .account(get_vault_pda(context.owner.pubkey()))
        .await?;
    
    assert!(vault_account.withdrawal_whitelist.contains(&whitelist_address));
    
    Ok(())
}

#[tokio::test]
async fn test_toggle_whitelist() -> Result<()> {
    let mut context = setup_test_context().await?;
    initialize_vault(&mut context).await?;
    
    let tx = context
        .program
        .request()
        .accounts(collateral_vault::accounts::ConfigureVault {
            user: context.owner.pubkey(),
            vault: get_vault_pda(context.owner.pubkey()),
            owner: context.owner.pubkey(),
        })
        .args(collateral_vault::instruction::ToggleWhitelist { enabled: true })
        .signer(&context.owner)
        .send()
        .await?;
    
    assert!(tx.is_success());
    
    let vault_account: CollateralVault = context
        .program
        .account(get_vault_pda(context.owner.pubkey()))
        .await?;
    
    assert!(vault_account.whitelist_enabled);
    
    Ok(())
}

#[tokio::test]
async fn test_configure_rate_limit() -> Result<()> {
    let mut context = setup_test_context().await?;
    initialize_vault(&mut context).await?;
    
    let max_amount = 1_000_000_000;
    let time_window = 86400; // 1 day
    
    let tx = context
        .program
        .request()
        .accounts(collateral_vault::accounts::ConfigureVault {
            user: context.owner.pubkey(),
            vault: get_vault_pda(context.owner.pubkey()),
            owner: context.owner.pubkey(),
        })
        .args(collateral_vault::instruction::ConfigureRateLimit { max_amount, time_window })
        .signer(&context.owner)
        .send()
        .await?;
    
    assert!(tx.is_success());
    
    let vault_account: CollateralVault = context
        .program
        .account(get_vault_pda(context.owner.pubkey()))
        .await?;
    
    assert_eq!(vault_account.rate_limit_amount, max_amount);
    assert_eq!(vault_account.rate_limit_window, time_window);
    
    Ok(())
}

#[tokio::test]
async fn test_configure_timelock() -> Result<()> {
    let mut context = setup_test_context().await?;
    initialize_vault(&mut context).await?;
    
    let duration = 3600; // 1 hour
    
    let tx = context
        .program
        .request()
        .accounts(collateral_vault::accounts::ConfigureVault {
            user: context.owner.pubkey(),
            vault: get_vault_pda(context.owner.pubkey()),
            owner: context.owner.pubkey(),
        })
        .args(collateral_vault::instruction::ConfigureTimelock { duration })
        .signer(&context.owner)
        .send()
        .await?;
    
    assert!(tx.is_success());
    
    let vault_account: CollateralVault = context
        .program
        .account(get_vault_pda(context.owner.pubkey()))
        .await?;
    
    assert_eq!(vault_account.withdrawal_timelock, duration);
    
    Ok(())
}

#[tokio::test]
async fn test_toggle_emergency_mode() -> Result<()> {
    let mut context = setup_test_context().await?;
    initialize_vault(&mut context).await?;
    
    let tx = context
        .program
        .request()
        .accounts(collateral_vault::accounts::ConfigureVault {
            user: context.owner.pubkey(),
            vault: get_vault_pda(context.owner.pubkey()),
            owner: context.owner.pubkey(),
        })
        .args(collateral_vault::instruction::ToggleEmergencyMode { enabled: true })
        .signer(&context.owner)
        .send()
        .await?;
    
    assert!(tx.is_success());
    
    let vault_account: CollateralVault = context
        .program
        .account(get_vault_pda(context.owner.pubkey()))
        .await?;
    
    assert!(vault_account.emergency_mode);
    
    Ok(())
}

#[tokio::test]
async fn test_request_withdrawal() -> Result<()> {
    let mut context = setup_test_context().await?;
    initialize_vault(&mut context).await?;
    
    let deposit_amount = 1_000_000_000;
    mint_tokens(&mut context, deposit_amount).await?;
    deposit(&mut context, deposit_amount).await?;
    
    // Configure timelock first (required for request_withdrawal)
    configure_timelock(&mut context, 3600).await?;
    
    let withdrawal_amount = 500_000_000;
    let recipient = Pubkey::from_str("66666666666666666666666666666666").unwrap();
    
    let tx = context
        .program
        .request()
        .accounts(collateral_vault::accounts::RequestWithdrawal {
            user: context.owner.pubkey(),
            vault: get_vault_pda(context.owner.pubkey()),
            owner: context.owner.pubkey(),
        })
        .args(collateral_vault::instruction::RequestWithdrawal { amount: withdrawal_amount, recipient })
        .signer(&context.owner)
        .send()
        .await?;
    
    assert!(tx.is_success());
    
    let vault_account: CollateralVault = context
        .program
        .account(get_vault_pda(context.owner.pubkey()))
        .await?;
    
    assert!(vault_account.pending_withdrawal.is_some());
    let pending = vault_account.pending_withdrawal.unwrap();
    assert_eq!(pending.amount, withdrawal_amount);
    assert_eq!(pending.recipient, recipient);
    
    Ok(())
}

#[tokio::test]
async fn test_configure_yield() -> Result<()> {
    let mut context = setup_test_context().await?;
    initialize_vault(&mut context).await?;
    
    let tx = context
        .program
        .request()
        .accounts(collateral_vault::accounts::ConfigureYield {
            user: context.owner.pubkey(),
            vault: get_vault_pda(context.owner.pubkey()),
            owner: context.owner.pubkey(),
        })
        .args(collateral_vault::instruction::ConfigureYield { enabled: true })
        .signer(&context.owner)
        .send()
        .await?;
    
    assert!(tx.is_success());
    
    let vault_account: CollateralVault = context
        .program
        .account(get_vault_pda(context.owner.pubkey()))
        .await?;
    
    assert!(vault_account.yield_enabled);
    
    Ok(())
}

#[tokio::test]
async fn test_lock_insufficient_balance() -> Result<()> {
    let mut context = setup_test_context().await?;
    initialize_vault(&mut context).await?;
    
    let deposit_amount = 1_000_000_000;
    mint_tokens(&mut context, deposit_amount).await?;
    deposit(&mut context, deposit_amount).await?;
    
    let result = context
        .program
        .request()
        .accounts(collateral_vault::accounts::LockCollateral {
            user: context.owner.pubkey(),
            vault: get_vault_pda(context.owner.pubkey()),
            owner: context.owner.pubkey(),
        })
        .args(collateral_vault::instruction::LockCollateral { amount: deposit_amount + 1 })
        .signer(&context.owner)
        .send()
        .await;
    
    assert!(result.is_err());
    
    Ok(())
}

#[tokio::test]
async fn test_withdraw_locked_collateral() -> Result<()> {
    let mut context = setup_test_context().await?;
    initialize_vault(&mut context).await?;
    
    let deposit_amount = 1_000_000_000;
    mint_tokens(&mut context, deposit_amount).await?;
    deposit(&mut context, deposit_amount).await?;
    
    let lock_amount = 600_000_000;
    lock_collateral(&mut context, lock_amount).await?;
    
    // Try to withdraw more than available (locked funds shouldn't be withdrawable)
    let result = context
        .program
        .request()
        .accounts(collateral_vault::accounts::Withdraw {
            user: context.owner.pubkey(),
            vault: get_vault_pda(context.owner.pubkey()),
            user_token_account: context.user_token_account,
            vault_token_account: get_vault_token_account(&get_vault_pda(context.owner.pubkey()), &context.usdt_mint.pubkey()),
            owner: context.owner.pubkey(),
            token_program: anchor_spl::token::ID,
        })
        .args(collateral_vault::instruction::Withdraw { amount: deposit_amount - lock_amount + 1 })
        .signer(&context.owner)
        .send()
        .await;
    
    assert!(result.is_err());
    
    Ok(())
}

#[tokio::test]
async fn test_multiple_deposits() -> Result<()> {
    let mut context = setup_test_context().await?;
    initialize_vault(&mut context).await?;
    
    let deposit1 = 500_000_000;
    let deposit2 = 300_000_000;
    let deposit3 = 200_000_000;
    
    mint_tokens(&mut context, deposit1 + deposit2 + deposit3).await?;
    deposit(&mut context, deposit1).await?;
    deposit(&mut context, deposit2).await?;
    deposit(&mut context, deposit3).await?;
    
    let vault_account: CollateralVault = context
        .program
        .account(get_vault_pda(context.owner.pubkey()))
        .await?;
    
    assert_eq!(vault_account.total_balance, deposit1 + deposit2 + deposit3);
    assert_eq!(vault_account.total_deposited, deposit1 + deposit2 + deposit3);
    
    Ok(())
}

#[tokio::test]
async fn test_complex_workflow() -> Result<()> {
    let mut context = setup_test_context().await?;
    initialize_vault(&mut context).await?;
    
    // Initial deposit
    let deposit_amount = 2_000_000_000;
    mint_tokens(&mut context, deposit_amount).await?;
    deposit(&mut context, deposit_amount).await?;
    
    // Lock some collateral
    let lock_amount = 800_000_000;
    lock_collateral(&mut context, lock_amount).await?;
    
    // Withdraw available funds
    let withdraw_amount = 500_000_000;
    withdraw(&mut context, withdraw_amount).await?;
    
    // Unlock some collateral
    let unlock_amount = 300_000_000;
    unlock_collateral(&mut context, unlock_amount).await?;
    
    // Final state check
    let vault_account: CollateralVault = context
        .program
        .account(get_vault_pda(context.owner.pubkey()))
        .await?;
    
    let expected_total = deposit_amount - withdraw_amount;
    let expected_locked = lock_amount - unlock_amount;
    let expected_available = expected_total - expected_locked;
    
    assert_eq!(vault_account.total_balance, expected_total);
    assert_eq!(vault_account.locked_balance, expected_locked);
    assert_eq!(vault_account.available_balance, expected_available);
    
    Ok(())
}
