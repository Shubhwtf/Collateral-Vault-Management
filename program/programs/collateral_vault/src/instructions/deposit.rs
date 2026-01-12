use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

use crate::state::{CollateralVault, DepositEvent};
use crate::errors::VaultError;

pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
    require!(amount > 0, VaultError::InvalidAmount);

    let cpi_accounts = Transfer {
        from: ctx.accounts.user_token_account.to_account_info(),
        to: ctx.accounts.vault_token_account.to_account_info(),
        authority: ctx.accounts.user.to_account_info(),
    };
    
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    
    token::transfer(cpi_ctx, amount)?;

    let vault = &mut ctx.accounts.vault;
    vault.add_deposit(amount)?;

    let clock = Clock::get()?;
    emit!(DepositEvent {
        user: ctx.accounts.user.key(),
        amount,
        new_balance: vault.total_balance,
        timestamp: clock.unix_timestamp,
    });

    msg!("Deposited {} to vault. New balance: {}", amount, vault.total_balance);

    Ok(())
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"vault", user.key().as_ref()],
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

    /// CHECK: This is checked in the constraint above
    pub owner: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
}

