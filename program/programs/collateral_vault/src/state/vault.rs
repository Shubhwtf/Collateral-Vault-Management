use anchor_lang::prelude::*;

#[account]
pub struct CollateralVault {
    pub owner: Pubkey,
    pub token_account: Pubkey,
    pub total_balance: u64,
    pub locked_balance: u64,
    pub available_balance: u64,
    pub total_deposited: u64,
    pub total_withdrawn: u64,
    pub created_at: i64,
    pub bump: u8,

    pub multisig_threshold: u8,
    pub authorized_signers: Vec<Pubkey>,
    pub delegated_users: Vec<Pubkey>,
    pub withdrawal_timelock: i64,
    pub pending_withdrawal: Option<PendingWithdrawal>,
    pub emergency_mode: bool,
    pub yield_enabled: bool,
    pub total_yield_earned: u64,
    pub last_yield_compound: i64,
    pub whitelist_enabled: bool,
    pub withdrawal_whitelist: Vec<Pubkey>,
    pub rate_limit_amount: u64,
    pub rate_limit_window: i64,
    pub rate_limit_window_start: i64,
    pub rate_limit_withdrawn: u64,
    pub last_update: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct PendingWithdrawal {
    pub amount: u64,
    pub requested_at: i64,
    pub executable_at: i64,
    pub recipient: Pubkey,
}

impl CollateralVault {
    // account size calculation includes max vec lengths to prevent realloc issues
    pub const LEN: usize = 8 + 32 + 32 + 8 + 8 + 8 + 8 + 8 + 8 + 1 + 1 + 4 + (32 * 10) + 4 + (32 * 5) + 8 + 1 + (8 + 8 + 8 + 32) + 1 + 1 + 8 + 8 + 1 + 4 + (32 * 20) + 8 + 8 + 8 + 8 + 8;

    pub fn add_deposit(&mut self, amount: u64) -> Result<()> {
        self.total_balance = self.total_balance
            .checked_add(amount)
            .ok_or(error!(crate::errors::VaultError::NumericalOverflow))?;
        
        self.available_balance = self.available_balance
            .checked_add(amount)
            .ok_or(error!(crate::errors::VaultError::NumericalOverflow))?;
        
        self.total_deposited = self.total_deposited
            .checked_add(amount)
            .ok_or(error!(crate::errors::VaultError::NumericalOverflow))?;
        
        Ok(())
    }

    pub fn sub_withdrawal(&mut self, amount: u64) -> Result<()> {
        self.total_balance = self.total_balance
            .checked_sub(amount)
            .ok_or(error!(crate::errors::VaultError::InsufficientBalance))?;
        
        self.available_balance = self.available_balance
            .checked_sub(amount)
            .ok_or(error!(crate::errors::VaultError::InsufficientAvailableBalance))?;
        
        self.total_withdrawn = self.total_withdrawn
            .checked_add(amount)
            .ok_or(error!(crate::errors::VaultError::NumericalOverflow))?;
        
        Ok(())
    }

    pub fn lock(&mut self, amount: u64) -> Result<()> {
        require!(
            self.available_balance >= amount,
            crate::errors::VaultError::InsufficientAvailableBalance
        );

        self.available_balance = self.available_balance
            .checked_sub(amount)
            .ok_or(error!(crate::errors::VaultError::InsufficientBalance))?;
        
        self.locked_balance = self.locked_balance
            .checked_add(amount)
            .ok_or(error!(crate::errors::VaultError::NumericalOverflow))?;
        
        Ok(())
    }

    pub fn unlock(&mut self, amount: u64) -> Result<()> {
        self.locked_balance = self.locked_balance
            .checked_sub(amount)
            .ok_or(error!(crate::errors::VaultError::InsufficientBalance))?;
        
        self.available_balance = self.available_balance
            .checked_add(amount)
            .ok_or(error!(crate::errors::VaultError::NumericalOverflow))?;
        
        Ok(())
    }

    pub fn initialize_advanced_features(&mut self, clock: &Clock) {
        self.multisig_threshold = 0;
        self.authorized_signers = Vec::new();
        self.delegated_users = Vec::new();
        self.withdrawal_timelock = 0;
        self.pending_withdrawal = None;
        self.emergency_mode = false;
        self.yield_enabled = false;
        self.total_yield_earned = 0;
        self.last_yield_compound = clock.unix_timestamp;
        self.whitelist_enabled = false;
        self.withdrawal_whitelist = Vec::new();
        self.rate_limit_amount = u64::MAX;
        self.rate_limit_window = 86400;
        self.rate_limit_window_start = clock.unix_timestamp;
        self.rate_limit_withdrawn = 0;
        self.last_update = clock.unix_timestamp;
    }

    pub fn is_authorized(&self, user: &Pubkey) -> bool {
        &self.owner == user || self.delegated_users.contains(user)
    }

    pub fn is_withdrawal_allowed(&self, recipient: &Pubkey) -> bool {
        if !self.whitelist_enabled {
            return true;
        }
        self.withdrawal_whitelist.contains(recipient)
    }

    pub fn check_and_update_rate_limit(&mut self, amount: u64, clock: &Clock) -> Result<()> {
        if clock.unix_timestamp >= self.rate_limit_window_start + self.rate_limit_window {
            self.rate_limit_window_start = clock.unix_timestamp;
            self.rate_limit_withdrawn = 0;
        }

        let new_total = self.rate_limit_withdrawn
            .checked_add(amount)
            .ok_or(error!(crate::errors::VaultError::NumericalOverflow))?;

        require!(
            new_total <= self.rate_limit_amount,
            crate::errors::VaultError::RateLimitExceeded
        );

        self.rate_limit_withdrawn = new_total;
        Ok(())
    }

    pub fn request_withdrawal(&mut self, amount: u64, recipient: Pubkey, clock: &Clock) -> Result<()> {
        require!(
            self.pending_withdrawal.is_none(),
            crate::errors::VaultError::PendingWithdrawalExists
        );

        require!(
            self.available_balance >= amount,
            crate::errors::VaultError::InsufficientAvailableBalance
        );

        let executable_at = clock.unix_timestamp + self.withdrawal_timelock;
        
        self.pending_withdrawal = Some(PendingWithdrawal {
            amount,
            requested_at: clock.unix_timestamp,
            executable_at,
            recipient,
        });

        Ok(())
    }

    pub fn execute_pending_withdrawal(&mut self, clock: &Clock) -> Result<(u64, Pubkey)> {
        let pending = self.pending_withdrawal
            .as_ref()
            .ok_or(error!(crate::errors::VaultError::NoPendingWithdrawal))?;

        if !self.emergency_mode {
            require!(
                clock.unix_timestamp >= pending.executable_at,
                crate::errors::VaultError::TimeLockNotExpired
            );
        }

        let amount = pending.amount;
        let recipient = pending.recipient;

        self.sub_withdrawal(amount)?;
        self.pending_withdrawal = None;

        Ok((amount, recipient))
    }

    pub fn add_yield(&mut self, yield_amount: u64) -> Result<()> {
        self.total_balance = self.total_balance
            .checked_add(yield_amount)
            .ok_or(error!(crate::errors::VaultError::NumericalOverflow))?;
        
        self.available_balance = self.available_balance
            .checked_add(yield_amount)
            .ok_or(error!(crate::errors::VaultError::NumericalOverflow))?;
        
        self.total_yield_earned = self.total_yield_earned
            .checked_add(yield_amount)
            .ok_or(error!(crate::errors::VaultError::NumericalOverflow))?;
        
        Ok(())
    }

    pub fn add_delegated_user(&mut self, user: Pubkey) -> Result<()> {
        require!(
            !self.delegated_users.contains(&user),
            crate::errors::VaultError::UserAlreadyDelegated
        );
        
        require!(
            self.delegated_users.len() < 5,
            crate::errors::VaultError::MaxDelegatedUsersReached
        );
        
        self.delegated_users.push(user);
        Ok(())
    }

    pub fn remove_delegated_user(&mut self, user: &Pubkey) -> Result<()> {
        if let Some(pos) = self.delegated_users.iter().position(|x| x == user) {
            self.delegated_users.remove(pos);
            Ok(())
        } else {
            Err(error!(crate::errors::VaultError::UserNotDelegated))
        }
    }

    pub fn add_to_whitelist(&mut self, address: Pubkey) -> Result<()> {
        require!(
            !self.withdrawal_whitelist.contains(&address),
            crate::errors::VaultError::AddressAlreadyWhitelisted
        );
        
        require!(
            self.withdrawal_whitelist.len() < 20,
            crate::errors::VaultError::MaxWhitelistReached
        );
        
        self.withdrawal_whitelist.push(address);
        Ok(())
    }

    pub fn add_signer(&mut self, signer: Pubkey) -> Result<()> {
        require!(
            !self.authorized_signers.contains(&signer),
            crate::errors::VaultError::SignerAlreadyAuthorized
        );
        
        require!(
            self.authorized_signers.len() < 10,
            crate::errors::VaultError::MaxSignersReached
        );
        
        self.authorized_signers.push(signer);
        Ok(())
    }
}

#[event]
pub struct DepositEvent {
    pub user: Pubkey,
    pub amount: u64,
    pub new_balance: u64,
    pub timestamp: i64,
}

#[event]
pub struct WithdrawEvent {
    pub user: Pubkey,
    pub amount: u64,
    pub new_balance: u64,
    pub timestamp: i64,
}

#[event]
pub struct LockEvent {
    pub user: Pubkey,
    pub amount: u64,
    pub locked_balance: u64,
    pub available_balance: u64,
    pub timestamp: i64,
}

#[event]
pub struct UnlockEvent {
    pub user: Pubkey,
    pub amount: u64,
    pub locked_balance: u64,
    pub available_balance: u64,
    pub timestamp: i64,
}

#[event]
pub struct TransferEvent {
    pub from: Pubkey,
    pub to: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct MultiSigConfigured {
    pub vault: Pubkey,
    pub threshold: u8,
    pub signers_count: u8,
    pub timestamp: i64,
}

#[event]
pub struct DelegationEvent {
    pub vault: Pubkey,
    pub user: Pubkey,
    pub action: String,
    pub timestamp: i64,
}

#[event]
pub struct WithdrawalRequested {
    pub vault: Pubkey,
    pub amount: u64,
    pub executable_at: i64,
    pub timestamp: i64,
}

#[event]
pub struct YieldEarned {
    pub vault: Pubkey,
    pub amount: u64,
    pub total_yield: u64,
    pub timestamp: i64,
}

#[event]
pub struct EmergencyModeToggled {
    pub vault: Pubkey,
    pub enabled: bool,
    pub timestamp: i64,
}

