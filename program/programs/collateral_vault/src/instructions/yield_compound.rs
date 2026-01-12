use anchor_lang::prelude::*;

use crate::state::{CollateralVault, YieldEarned};
use crate::errors::VaultError;

pub fn compound_yield(ctx: Context<CompoundYield>) -> Result<()> {
    let vault = &mut ctx.accounts.vault;
    let clock = Clock::get()?;

    require!(vault.yield_enabled, VaultError::YieldNotEnabled);

    let time_elapsed = clock.unix_timestamp - vault.last_yield_compound;
    let annual_rate = 10000000;
    let seconds_per_year = 31_536_000i64;
    
    if time_elapsed > 0 && vault.total_balance > 0 {
        let yield_amount = (vault.total_balance as u128)
            .checked_mul(annual_rate as u128)
            .and_then(|v| v.checked_mul(time_elapsed as u128))
            .and_then(|v| v.checked_div(10000u128))
            .and_then(|v| v.checked_div(seconds_per_year as u128))
            .ok_or(error!(VaultError::NumericalOverflow))? as u64;

        if yield_amount > 0 {
            vault.add_yield(yield_amount)?;
            vault.last_yield_compound = clock.unix_timestamp;
            vault.last_update = clock.unix_timestamp;

            emit!(YieldEarned {
                vault: vault.key(),
                amount: yield_amount,
                total_yield: vault.total_yield_earned,
                timestamp: clock.unix_timestamp,
            });

            msg!("Compounded yield: {}, Total yield earned: {}", yield_amount, vault.total_yield_earned);
        }
    }

    Ok(())
}

pub fn auto_compound(ctx: Context<AutoCompound>) -> Result<()> {
    let vault = &mut ctx.accounts.vault;
    let clock = Clock::get()?;

    require!(vault.yield_enabled, VaultError::YieldNotEnabled);

    let min_compound_interval = 10i64;
    let time_since_last = clock.unix_timestamp - vault.last_yield_compound;

    require!(
        time_since_last >= min_compound_interval,
        VaultError::OperationNotAllowed
    );

    let annual_rate = 10000000;
    let seconds_per_year = 31_536_000i64;
    
    if vault.total_balance > 0 {
        let yield_amount = (vault.total_balance as u128)
            .checked_mul(annual_rate as u128)
            .and_then(|v| v.checked_mul(time_since_last as u128))
            .and_then(|v| v.checked_div(10000u128))
            .and_then(|v| v.checked_div(seconds_per_year as u128))
            .ok_or(error!(VaultError::NumericalOverflow))? as u64;

        if yield_amount > 0 {
            vault.add_yield(yield_amount)?;
            vault.last_yield_compound = clock.unix_timestamp;
            vault.last_update = clock.unix_timestamp;

            emit!(YieldEarned {
                vault: vault.key(),
                amount: yield_amount,
                total_yield: vault.total_yield_earned,
                timestamp: clock.unix_timestamp,
            });

            msg!("Auto-compounded yield: {}", yield_amount);
        }
    }

    Ok(())
}

pub fn configure_yield(ctx: Context<ConfigureYield>, enabled: bool) -> Result<()> {
    let vault = &mut ctx.accounts.vault;
    let clock = Clock::get()?;

    vault.yield_enabled = enabled;
    
    if enabled {
        vault.last_yield_compound = clock.unix_timestamp;
        msg!("Yield generation enabled for vault");
    } else {
        msg!("Yield generation disabled for vault");
    }

    Ok(())
}

#[derive(Accounts)]
pub struct CompoundYield<'info> {
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
pub struct AutoCompound<'info> {
    #[account(mut)]
    pub caller: Signer<'info>,

    #[account(
        mut,
        seeds = [b"vault", vault.owner.as_ref()],
        bump = vault.bump,
    )]
    pub vault: Account<'info, CollateralVault>,
}

#[derive(Accounts)]
pub struct ConfigureYield<'info> {
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
