use anchor_lang::prelude::*;

use crate::state::{CollateralVault, VaultAuthority, UnlockEvent};
use crate::errors::VaultError;

pub fn unlock_collateral(ctx: Context<UnlockCollateral>, amount: u64) -> Result<()> {
    require!(amount > 0, VaultError::InvalidAmount);

    let caller_program = ctx.accounts.caller_program.key();
    let authority = &ctx.accounts.vault_authority;
    
    require!(
        authority.is_authorized(&caller_program),
        VaultError::UnauthorizedProgram
    );

    let vault = &mut ctx.accounts.vault;
    vault.unlock(amount)?;

    let clock = Clock::get()?;
    emit!(UnlockEvent {
        user: vault.owner,
        amount,
        locked_balance: vault.locked_balance,
        available_balance: vault.available_balance,
        timestamp: clock.unix_timestamp,
    });

    msg!(
        "Unlocked {} collateral. Locked: {}, Available: {}",
        amount,
        vault.locked_balance,
        vault.available_balance
    );

    Ok(())
}

#[derive(Accounts)]
pub struct UnlockCollateral<'info> {
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

    /// CHECK: Verified against vault_authority.authorized_programs
    pub caller_program: UncheckedAccount<'info>,
}

