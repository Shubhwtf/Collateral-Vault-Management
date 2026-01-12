use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

use crate::state::{CollateralVault, WithdrawEvent};
use crate::errors::VaultError;

pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
    require!(amount > 0, VaultError::InvalidAmount);

    let vault = &mut ctx.accounts.vault;

    // only checking available_balance here - locked collateral can't be withdrawn
    // this prevents users from pulling funds that are backing active positions
    require!(
        vault.available_balance >= amount,
        VaultError::InsufficientAvailableBalance
    );

    // need PDA seeds to sign the CPI since vault owns the token account
    let owner_key = ctx.accounts.owner.key();
    let seeds = &[
        b"vault",
        owner_key.as_ref(),
        &[vault.bump],
    ];
    let signer = &[&seeds[..]];

    let cpi_accounts = Transfer {
        from: ctx.accounts.vault_token_account.to_account_info(),
        to: ctx.accounts.user_token_account.to_account_info(),
        authority: vault.to_account_info(),
    };
    
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
    
    token::transfer(cpi_ctx, amount)?;

    vault.sub_withdrawal(amount)?;

    let clock = Clock::get()?;
    emit!(WithdrawEvent {
        user: ctx.accounts.owner.key(),
        amount,
        new_balance: vault.total_balance,
        timestamp: clock.unix_timestamp,
    });

    msg!("Withdrawn {} from vault. New balance: {}", amount, vault.total_balance);

    Ok(())
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
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

    /// CHECK: Verified via has_one constraint
    pub owner: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
}

