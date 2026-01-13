use anchor_lang::prelude::*;

pub mod errors;
pub mod instructions;
pub mod state;

use instructions::*;
// devnet program public key, so fine for committing to github
declare_id!("J4AH5hKsnigMxdcGoLAffr7XxKVLHw22y6RG3qEsi9Dd");

#[program]
pub mod collateral_vault {
    use super::*;

    pub fn initialize_vault(ctx: Context<InitializeVault>) -> Result<()> {
        instructions::initialize_vault(ctx)
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        instructions::deposit(ctx, amount)
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        instructions::withdraw(ctx, amount)
    }

    pub fn lock_collateral(ctx: Context<LockCollateral>, amount: u64) -> Result<()> {
        instructions::lock_collateral(ctx, amount)
    }

    pub fn unlock_collateral(ctx: Context<UnlockCollateral>, amount: u64) -> Result<()> {
        instructions::unlock_collateral(ctx, amount)
    }

    pub fn transfer_collateral(
        ctx: Context<TransferCollateral>,
        amount: u64,
    ) -> Result<()> {
        instructions::transfer_collateral(ctx, amount)
    }

    pub fn initialize_authority(
        ctx: Context<InitializeAuthority>,
        authorized_programs: Vec<Pubkey>,
    ) -> Result<()> {
        instructions::initialize_authority(ctx, authorized_programs)
    }

    pub fn add_authorized_program(
        ctx: Context<UpdateAuthority>,
        program: Pubkey,
    ) -> Result<()> {
        instructions::add_authorized_program(ctx, program)
    }

    pub fn remove_authorized_program(
        ctx: Context<UpdateAuthority>,
        program: Pubkey,
    ) -> Result<()> {
        instructions::remove_authorized_program(ctx, program)
    }

    pub fn batch_deposit(ctx: Context<BatchDeposit>, amounts: Vec<u64>) -> Result<()> {
        instructions::batch_deposit(ctx, amounts)
    }

    pub fn batch_withdraw(ctx: Context<BatchWithdraw>, amounts: Vec<u64>) -> Result<()> {
        instructions::batch_withdraw(ctx, amounts)
    }

    pub fn compound_yield(ctx: Context<CompoundYield>) -> Result<()> {
        instructions::compound_yield(ctx)
    }

    pub fn auto_compound(ctx: Context<AutoCompound>) -> Result<()> {
        instructions::auto_compound(ctx)
    }

    pub fn configure_yield(ctx: Context<ConfigureYield>, enabled: bool) -> Result<()> {
        instructions::configure_yield(ctx, enabled)
    }

    pub fn configure_multisig(
        ctx: Context<ConfigureMultiSig>,
        threshold: u8,
        signers: Vec<Pubkey>,
    ) -> Result<()> {
        instructions::configure_multisig(ctx, threshold, signers)
    }

    pub fn add_delegate(ctx: Context<ManageDelegate>, user: Pubkey) -> Result<()> {
        instructions::add_delegate(ctx, user)
    }

    pub fn remove_delegate(ctx: Context<ManageDelegate>, user: Pubkey) -> Result<()> {
        instructions::remove_delegate(ctx, user)
    }

    pub fn add_to_whitelist(ctx: Context<ManageWhitelist>, address: Pubkey) -> Result<()> {
        instructions::add_to_whitelist(ctx, address)
    }

    pub fn remove_from_whitelist(ctx: Context<ManageWhitelist>, address: Pubkey) -> Result<()> {
        instructions::remove_from_whitelist(ctx, address)
    }

    pub fn toggle_whitelist(ctx: Context<ConfigureVault>, enabled: bool) -> Result<()> {
        instructions::toggle_whitelist(ctx, enabled)
    }

    pub fn configure_rate_limit(
        ctx: Context<ConfigureVault>,
        max_amount: u64,
        time_window: i64,
    ) -> Result<()> {
        instructions::configure_rate_limit(ctx, max_amount, time_window)
    }

    pub fn configure_timelock(ctx: Context<ConfigureVault>, duration: i64) -> Result<()> {
        instructions::configure_timelock(ctx, duration)
    }

    pub fn toggle_emergency_mode(ctx: Context<ConfigureVault>, enabled: bool) -> Result<()> {
        instructions::toggle_emergency_mode(ctx, enabled)
    }

    pub fn request_withdrawal(
        ctx: Context<RequestWithdrawal>,
        amount: u64,
        recipient: Pubkey,
    ) -> Result<()> {
        instructions::request_withdrawal(ctx, amount, recipient)
    }

    pub fn cancel_withdrawal(ctx: Context<CancelWithdrawal>) -> Result<()> {
        instructions::cancel_withdrawal(ctx)
    }

    pub fn execute_withdrawal(ctx: Context<ExecuteWithdrawal>) -> Result<()> {
        instructions::execute_withdrawal(ctx)
    }
}

