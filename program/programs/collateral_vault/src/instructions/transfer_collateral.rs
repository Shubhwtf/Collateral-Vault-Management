use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

use crate::state::{CollateralVault, VaultAuthority, TransferEvent};
use crate::errors::VaultError;

pub fn transfer_collateral(ctx: Context<TransferCollateral>, amount: u64) -> Result<()> {
    require!(amount > 0, VaultError::InvalidAmount);

    let caller_program = ctx.accounts.caller_program.key();
    let authority = &ctx.accounts.vault_authority;
    
    require!(
        authority.is_authorized(&caller_program),
        VaultError::UnauthorizedProgram
    );

    let from_vault = &mut ctx.accounts.from_vault;
    let to_vault = &mut ctx.accounts.to_vault;

    require!(
        from_vault.total_balance >= amount,
        VaultError::InsufficientBalance
    );

    // transferring between two vault PDAs - this is for things like liquidations
    // where collateral needs to move from liquidated user to liquidator
    let from_owner = from_vault.owner;
    let seeds = &[
        b"vault",
        from_owner.as_ref(),
        &[from_vault.bump],
    ];
    let signer = &[&seeds[..]];

    let cpi_accounts = Transfer {
        from: ctx.accounts.from_token_account.to_account_info(),
        to: ctx.accounts.to_token_account.to_account_info(),
        authority: from_vault.to_account_info(),
    };
    
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
    
    token::transfer(cpi_ctx, amount)?;

    from_vault.sub_withdrawal(amount)?;
    to_vault.add_deposit(amount)?;

    let clock = Clock::get()?;
    emit!(TransferEvent {
        from: from_vault.owner,
        to: to_vault.owner,
        amount,
        timestamp: clock.unix_timestamp,
    });

    msg!(
        "Transferred {} from {} to {}",
        amount,
        from_vault.owner,
        to_vault.owner
    );

    Ok(())
}

#[derive(Accounts)]
pub struct TransferCollateral<'info> {
    #[account(
        mut,
        seeds = [b"vault", from_vault.owner.as_ref()],
        bump = from_vault.bump,
    )]
    pub from_vault: Account<'info, CollateralVault>,

    #[account(
        mut,
        seeds = [b"vault", to_vault.owner.as_ref()],
        bump = to_vault.bump,
    )]
    pub to_vault: Account<'info, CollateralVault>,

    #[account(
        mut,
        constraint = from_token_account.key() == from_vault.token_account @ VaultError::InvalidTokenAccount
    )]
    pub from_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = to_token_account.key() == to_vault.token_account @ VaultError::InvalidTokenAccount
    )]
    pub to_token_account: Account<'info, TokenAccount>,

    #[account(
        seeds = [b"vault_authority"],
        bump = vault_authority.bump,
    )]
    pub vault_authority: Account<'info, VaultAuthority>,

    /// CHECK: Verified against vault_authority.authorized_programs
    pub caller_program: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
}

