use anchor_lang::prelude::*;

use crate::state::{CollateralVault, WithdrawalRequested};
use crate::errors::VaultError;

pub fn request_withdrawal(
    ctx: Context<RequestWithdrawal>,
    amount: u64,
    recipient: Pubkey,
) -> Result<()> {
    require!(amount > 0, VaultError::InvalidAmount);

    let vault = &mut ctx.accounts.vault;
    let clock = Clock::get()?;

    require!(
        vault.withdrawal_timelock > 0,
        VaultError::FeatureNotEnabled
    );

    if vault.whitelist_enabled {
        require!(
            vault.is_withdrawal_allowed(&recipient),
            VaultError::RecipientNotWhitelisted
        );
    }

    vault.request_withdrawal(amount, recipient, &clock)?;

    emit!(WithdrawalRequested {
        vault: vault.key(),
        amount,
        executable_at: vault.pending_withdrawal.as_ref().unwrap().executable_at,
        timestamp: clock.unix_timestamp,
    });

    msg!("Withdrawal requested: {} tokens, executable at: {}", 
        amount, 
        vault.pending_withdrawal.as_ref().unwrap().executable_at
    );

    Ok(())
}

// preventing cancellation after timelock expires to avoid race conditions
// where user cancels right as someone else tries to execute
pub fn cancel_withdrawal(ctx: Context<CancelWithdrawal>) -> Result<()> {
    let vault = &mut ctx.accounts.vault;
    let clock = Clock::get()?;

    let pending = vault.pending_withdrawal
        .as_ref()
        .ok_or(error!(VaultError::NoPendingWithdrawal))?;

    require!(
        clock.unix_timestamp < pending.executable_at,
        VaultError::CannotCancelExpiredWithdrawal
    );

    vault.pending_withdrawal = None;
    msg!("Pending withdrawal cancelled");

    Ok(())
}

#[derive(Accounts)]
pub struct RequestWithdrawal<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"vault", owner.key().as_ref()],
        bump = vault.bump,
        has_one = owner @ VaultError::InvalidAuthority,
    )]
    pub vault: Account<'info, CollateralVault>,

    /// CHECK: Verified through has_one constraint
    pub owner: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct CancelWithdrawal<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"vault", owner.key().as_ref()],
        bump = vault.bump,
        has_one = owner @ VaultError::InvalidAuthority,
    )]
    pub vault: Account<'info, CollateralVault>,

    /// CHECK: Verified through has_one constraint
    pub owner: UncheckedAccount<'info>,
}
