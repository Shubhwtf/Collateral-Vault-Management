use anchor_lang::prelude::*;

use crate::state::VaultAuthority;
use crate::errors::VaultError;

pub fn initialize_authority(
    ctx: Context<InitializeAuthority>,
    authorized_programs: Vec<Pubkey>,
) -> Result<()> {
    require!(
        authorized_programs.len() <= VaultAuthority::MAX_AUTHORIZED,
        VaultError::MaxAuthorizedProgramsReached
    );

    let authority = &mut ctx.accounts.vault_authority;
    authority.authorized_programs = authorized_programs;
    authority.admin = ctx.accounts.admin.key();
    authority.bump = ctx.bumps.vault_authority;

    msg!("Vault authority initialized by admin: {}", ctx.accounts.admin.key());

    Ok(())
}

pub fn add_authorized_program(ctx: Context<UpdateAuthority>, program: Pubkey) -> Result<()> {
    let authority = &mut ctx.accounts.vault_authority;
    authority.add_program(program)?;

    msg!("Added authorized program: {}", program);

    Ok(())
}

pub fn remove_authorized_program(ctx: Context<UpdateAuthority>, program: Pubkey) -> Result<()> {
    let authority = &mut ctx.accounts.vault_authority;
    authority.remove_program(&program)?;

    msg!("Removed authorized program: {}", program);

    Ok(())
}

#[derive(Accounts)]
pub struct InitializeAuthority<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        init,
        payer = admin,
        space = VaultAuthority::LEN,
        seeds = [b"vault_authority"],
        bump
    )]
    pub vault_authority: Account<'info, VaultAuthority>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateAuthority<'info> {
    #[account(
        mut,
        constraint = admin.key() == vault_authority.admin @ VaultError::InvalidAuthority
    )]
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [b"vault_authority"],
        bump = vault_authority.bump,
    )]
    pub vault_authority: Account<'info, VaultAuthority>,
}

