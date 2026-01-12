use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

use crate::state::{CollateralVault, WithdrawEvent};
use crate::errors::VaultError;

pub fn execute_withdrawal(ctx: Context<ExecuteWithdrawal>) -> Result<()> {
    let vault = &mut ctx.accounts.vault;
    let clock = Clock::get()?;

    let (amount, recipient) = vault.execute_pending_withdrawal(&clock)?;

    require!(
        recipient == ctx.accounts.recipient.key(),
        VaultError::InvalidAuthority
    );

    let owner_key = ctx.accounts.owner.key();
    let seeds = &[
        b"vault",
        owner_key.as_ref(),
        &[vault.bump],
    ];
    let signer = &[&seeds[..]];

    let cpi_accounts = Transfer {
        from: ctx.accounts.vault_token_account.to_account_info(),
        to: ctx.accounts.recipient_token_account.to_account_info(),
        authority: vault.to_account_info(),
    };

    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
    token::transfer(cpi_ctx, amount)?;

    emit!(WithdrawEvent {
        user: ctx.accounts.owner.key(),
        amount,
        new_balance: vault.total_balance,
        timestamp: clock.unix_timestamp,
    });

    msg!("Executed withdrawal: {} tokens", amount);
    Ok(())
}

#[derive(Accounts)]
pub struct ExecuteWithdrawal<'info> {
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
    pub recipient_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = vault_token_account.key() == vault.token_account @ VaultError::InvalidTokenAccount
    )]
    pub vault_token_account: Account<'info, TokenAccount>,

    /// CHECK: Verified against pending withdrawal
    pub recipient: UncheckedAccount<'info>,

    /// CHECK: Verified through has_one constraint
    pub owner: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
}
