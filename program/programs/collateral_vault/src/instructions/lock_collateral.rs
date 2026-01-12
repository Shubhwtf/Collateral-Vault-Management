use anchor_lang::prelude::*;

use crate::state::{CollateralVault, VaultAuthority, LockEvent};
use crate::errors::VaultError;

pub fn lock_collateral(ctx: Context<LockCollateral>, amount: u64) -> Result<()> {
    require!(amount > 0, VaultError::InvalidAmount);

    // this is meant to be called via CPI from other programs (like a position manager)
    // checking caller_program against the authorized list prevents random programs from locking user funds
    let caller_program = ctx.accounts.caller_program.key();
    let authority = &ctx.accounts.vault_authority;
    
    require!(
        authority.is_authorized(&caller_program),
        VaultError::UnauthorizedProgram
    );

    let vault = &mut ctx.accounts.vault;
    vault.lock(amount)?;

    let clock = Clock::get()?;
    emit!(LockEvent {
        user: vault.owner,
        amount,
        locked_balance: vault.locked_balance,
        available_balance: vault.available_balance,
        timestamp: clock.unix_timestamp,
    });

    msg!(
        "Locked {} collateral. Locked: {}, Available: {}",
        amount,
        vault.locked_balance,
        vault.available_balance
    );

    Ok(())
}

#[derive(Accounts)]
pub struct LockCollateral<'info> {
    #[account(
        mut,
        seeds = [b"vault", vault.owner.as_ref()],
        bump = vault.bump,
    )]
    pub vault: Account<'info, CollateralVault>,

    #[account(
        seeds = [b"vault_authority"],
        bump = vault_authority.bump,
    )]
    pub vault_authority: Account<'info, VaultAuthority>,
    
    /// CHECK: This account is verified against the vault_authority.authorized_programs list
    /// in the instruction logic. Only programs in the authorized list can lock user collateral.
    pub caller_program: UncheckedAccount<'info>,
}

