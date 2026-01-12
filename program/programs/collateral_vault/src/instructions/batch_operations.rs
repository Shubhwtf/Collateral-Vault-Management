use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

use crate::state::{CollateralVault, DepositEvent, WithdrawEvent};
use crate::errors::VaultError;

const MAX_BATCH_SIZE: usize = 10;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct BatchDepositItem {
    pub amount: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct BatchWithdrawItem {
    pub amount: u64,
}

pub fn batch_deposit(ctx: Context<BatchDeposit>, amounts: Vec<u64>) -> Result<()> {
    require!(
        amounts.len() > 0 && amounts.len() <= MAX_BATCH_SIZE,
        VaultError::BatchLimitExceeded
    );

    let vault = &mut ctx.accounts.vault;
    let clock = Clock::get()?;
    let mut total_deposited = 0u64;

    for amount in amounts.iter() {
        require!(*amount > 0, VaultError::InvalidAmount);

        let cpi_accounts = Transfer {
            from: ctx.accounts.user_token_account.to_account_info(),
            to: ctx.accounts.vault_token_account.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };

        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, *amount)?;

        vault.add_deposit(*amount)?;
        total_deposited = total_deposited.checked_add(*amount)
            .ok_or(error!(VaultError::NumericalOverflow))?;

        emit!(DepositEvent {
            user: ctx.accounts.owner.key(),
            amount: *amount,
            new_balance: vault.total_balance,
            timestamp: clock.unix_timestamp,
        });
    }

    msg!("Batch deposited {} items, total: {}", amounts.len(), total_deposited);
    Ok(())
}

pub fn batch_withdraw(ctx: Context<BatchWithdraw>, amounts: Vec<u64>) -> Result<()> {
    require!(
        amounts.len() > 0 && amounts.len() <= MAX_BATCH_SIZE,
        VaultError::BatchLimitExceeded
    );

    let vault = &mut ctx.accounts.vault;
    let clock = Clock::get()?;
    let mut total_withdrawn = 0u64;

    // calculating total first to avoid partial withdrawals if the last one would fail
    for amount in amounts.iter() {
        require!(*amount > 0, VaultError::InvalidAmount);
        total_withdrawn = total_withdrawn.checked_add(*amount)
            .ok_or(error!(VaultError::NumericalOverflow))?;
    }

    require!(
        vault.available_balance >= total_withdrawn,
        VaultError::InsufficientAvailableBalance
    );

    if vault.rate_limit_amount < u64::MAX {
        vault.check_and_update_rate_limit(total_withdrawn, &clock)?;
    }

    if vault.whitelist_enabled {
        require!(
            vault.is_withdrawal_allowed(&ctx.accounts.owner.key()),
            VaultError::RecipientNotWhitelisted
        );
    }

    let owner_key = ctx.accounts.owner.key();
    let seeds = &[
        b"vault",
        owner_key.as_ref(),
        &[vault.bump],
    ];
    let signer = &[&seeds[..]];

    for amount in amounts.iter() {
        let cpi_accounts = Transfer {
            from: ctx.accounts.vault_token_account.to_account_info(),
            to: ctx.accounts.user_token_account.to_account_info(),
            authority: vault.to_account_info(),
        };

        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::transfer(cpi_ctx, *amount)?;

        vault.sub_withdrawal(*amount)?;

        emit!(WithdrawEvent {
            user: ctx.accounts.owner.key(),
            amount: *amount,
            new_balance: vault.total_balance,
            timestamp: clock.unix_timestamp,
        });
    }

    msg!("Batch withdrew {} items, total: {}", amounts.len(), total_withdrawn);
    Ok(())
}

#[derive(Accounts)]
pub struct BatchDeposit<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"vault", owner.key().as_ref()],
        bump = vault.bump,
        has_one = owner @ VaultError::InvalidAuthority,
    )]
    pub vault: Account<'info, CollateralVault>,

    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = vault_token_account.key() == vault.token_account @ VaultError::InvalidTokenAccount
    )]
    pub vault_token_account: Account<'info, TokenAccount>,

    /// CHECK: Verified through has_one constraint
    pub owner: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct BatchWithdraw<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"vault", owner.key().as_ref()],
        bump = vault.bump,
        has_one = owner @ VaultError::InvalidAuthority,
    )]
    pub vault: Account<'info, CollateralVault>,

    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = vault_token_account.key() == vault.token_account @ VaultError::InvalidTokenAccount
    )]
    pub vault_token_account: Account<'info, TokenAccount>,

    /// CHECK: Verified through has_one constraint
    pub owner: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
}

#[event]
pub struct BatchOperationEvent {
    pub user: Pubkey,
    pub operation_type: String,
    pub count: u8,
    pub total_amount: u64,
    pub timestamp: i64,
}
