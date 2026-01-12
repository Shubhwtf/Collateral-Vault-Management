use anchor_lang::prelude::*;

use crate::state::{CollateralVault, DelegationEvent, EmergencyModeToggled, MultiSigConfigured};
use crate::errors::VaultError;

pub fn configure_multisig(
    ctx: Context<ConfigureMultiSig>,
    threshold: u8,
    signers: Vec<Pubkey>,
) -> Result<()> {
    let vault = &mut ctx.accounts.vault;
    let clock = Clock::get()?;

    require!(threshold > 0, VaultError::InvalidMultiSigThreshold);
    require!(
        signers.len() >= threshold as usize,
        VaultError::InvalidMultiSigThreshold
    );
    require!(signers.len() <= 10, VaultError::MaxSignersReached);

    vault.multisig_threshold = threshold;
    vault.authorized_signers = signers.clone();

    emit!(MultiSigConfigured {
        vault: vault.key(),
        threshold,
        signers_count: signers.len() as u8,
        timestamp: clock.unix_timestamp,
    });

    msg!("Multi-sig configured: {} of {} signers required", threshold, signers.len());
    Ok(())
}

pub fn add_delegate(ctx: Context<ManageDelegate>, user: Pubkey) -> Result<()> {
    let vault = &mut ctx.accounts.vault;
    let clock = Clock::get()?;

    vault.add_delegated_user(user)?;

    emit!(DelegationEvent {
        vault: vault.key(),
        user,
        action: "added".to_string(),
        timestamp: clock.unix_timestamp,
    });

    msg!("Added delegate: {}", user);
    Ok(())
}

pub fn remove_delegate(ctx: Context<ManageDelegate>, user: Pubkey) -> Result<()> {
    let vault = &mut ctx.accounts.vault;
    let clock = Clock::get()?;

    vault.remove_delegated_user(&user)?;

    emit!(DelegationEvent {
        vault: vault.key(),
        user,
        action: "removed".to_string(),
        timestamp: clock.unix_timestamp,
    });

    msg!("Removed delegate: {}", user);
    Ok(())
}

pub fn add_to_whitelist(ctx: Context<ManageWhitelist>, address: Pubkey) -> Result<()> {
    let vault = &mut ctx.accounts.vault;
    vault.add_to_whitelist(address)?;
    msg!("Added {} to withdrawal whitelist", address);
    Ok(())
}

pub fn remove_from_whitelist(ctx: Context<ManageWhitelist>, address: Pubkey) -> Result<()> {
    let vault = &mut ctx.accounts.vault;
    
    if let Some(pos) = vault.withdrawal_whitelist.iter().position(|x| x == &address) {
        vault.withdrawal_whitelist.remove(pos);
        msg!("Removed {} from withdrawal whitelist", address);
        Ok(())
    } else {
        Err(error!(VaultError::AddressNotWhitelisted))
    }
}

pub fn toggle_whitelist(ctx: Context<ConfigureVault>, enabled: bool) -> Result<()> {
    let vault = &mut ctx.accounts.vault;
    vault.whitelist_enabled = enabled;
    msg!("Withdrawal whitelist {}", if enabled { "enabled" } else { "disabled" });
    Ok(())
}

pub fn configure_rate_limit(
    ctx: Context<ConfigureVault>,
    max_amount: u64,
    time_window: i64,
) -> Result<()> {
    let vault = &mut ctx.accounts.vault;
    let clock = Clock::get()?;

    require!(time_window > 0, VaultError::InvalidRateLimitConfig);

    vault.rate_limit_amount = max_amount;
    vault.rate_limit_window = time_window;
    vault.rate_limit_window_start = clock.unix_timestamp;
    vault.rate_limit_withdrawn = 0;

    msg!("Rate limit configured: {} per {} seconds", max_amount, time_window);
    Ok(())
}

pub fn configure_timelock(ctx: Context<ConfigureVault>, duration: i64) -> Result<()> {
    let vault = &mut ctx.accounts.vault;
    vault.withdrawal_timelock = duration;
    msg!("Withdrawal timelock set to {} seconds", duration);
    Ok(())
}

// emergency mode bypasses timelock but not whitelist
// this is intentional - whitelist is for regulatory/compliance, timelock is just for safety
pub fn toggle_emergency_mode(ctx: Context<ConfigureVault>, enabled: bool) -> Result<()> {
    let vault = &mut ctx.accounts.vault;
    let clock = Clock::get()?;

    vault.emergency_mode = enabled;

    emit!(EmergencyModeToggled {
        vault: vault.key(),
        enabled,
        timestamp: clock.unix_timestamp,
    });

    msg!("Emergency mode {}", if enabled { "activated" } else { "deactivated" });
    Ok(())
}

#[derive(Accounts)]
pub struct ConfigureMultiSig<'info> {
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
pub struct ManageDelegate<'info> {
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
pub struct ManageWhitelist<'info> {
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
pub struct ConfigureVault<'info> {
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
