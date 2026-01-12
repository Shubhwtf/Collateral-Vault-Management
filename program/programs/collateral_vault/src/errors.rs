use anchor_lang::prelude::*;

#[error_code]
pub enum VaultError {
    #[msg("Invalid amount: must be greater than zero")]
    InvalidAmount,

    #[msg("Insufficient available balance")]
    InsufficientBalance,

    #[msg("Insufficient available balance (funds are locked)")]
    InsufficientAvailableBalance,

    #[msg("Vault has open positions, cannot withdraw")]
    HasOpenPositions,

    #[msg("Unauthorized program attempting restricted operation")]
    UnauthorizedProgram,

    #[msg("Numerical overflow occurred")]
    NumericalOverflow,

    #[msg("Vault not initialized")]
    VaultNotInitialized,

    #[msg("Invalid vault authority")]
    InvalidAuthority,

    #[msg("Program already authorized")]
    ProgramAlreadyAuthorized,

    #[msg("Program not authorized")]
    ProgramNotAuthorized,

    #[msg("Maximum authorized programs reached")]
    MaxAuthorizedProgramsReached,

    #[msg("Invalid token account")]
    InvalidTokenAccount,

    #[msg("Withdrawal amount exceeds available balance")]
    WithdrawalExceedsBalance,

    #[msg("Insufficient signatures for multi-sig operation")]
    InsufficientSignatures,
    
    #[msg("Signer not authorized for this vault")]
    SignerNotAuthorized,
    
    #[msg("Signer already authorized")]
    SignerAlreadyAuthorized,
    
    #[msg("Maximum signers reached (max 10)")]
    MaxSignersReached,
    
    #[msg("Invalid multi-sig threshold")]
    InvalidMultiSigThreshold,
    
    #[msg("User not authorized to perform this operation")]
    UserNotAuthorized,
    
    #[msg("User already delegated")]
    UserAlreadyDelegated,
    
    #[msg("User not delegated")]
    UserNotDelegated,
    
    #[msg("Maximum delegated users reached (max 5)")]
    MaxDelegatedUsersReached,
    
    #[msg("Withdrawal time lock has not expired")]
    TimeLockNotExpired,
    
    #[msg("No pending withdrawal request")]
    NoPendingWithdrawal,
    
    #[msg("Pending withdrawal already exists")]
    PendingWithdrawalExists,
    
    #[msg("Cannot cancel withdrawal, time lock expired")]
    CannotCancelExpiredWithdrawal,
    
    #[msg("Withdrawal recipient not whitelisted")]
    RecipientNotWhitelisted,
    
    #[msg("Address already whitelisted")]
    AddressAlreadyWhitelisted,
    
    #[msg("Address not whitelisted")]
    AddressNotWhitelisted,
    
    #[msg("Maximum whitelist addresses reached (max 20)")]
    MaxWhitelistReached,
    
    #[msg("Withdrawal rate limit exceeded for this time window")]
    RateLimitExceeded,
    
    #[msg("Invalid rate limit configuration")]
    InvalidRateLimitConfig,
    
    #[msg("Yield generation not enabled for this vault")]
    YieldNotEnabled,
    
    #[msg("No yield to claim")]
    NoYieldToClaim,
    
    #[msg("Yield strategy not found")]
    YieldStrategyNotFound,
    
    #[msg("Insufficient funds for yield investment")]
    InsufficientFundsForYield,
    
    #[msg("Emergency mode is not active")]
    EmergencyModeNotActive,
    
    #[msg("Cannot perform operation in emergency mode")]
    OperationBlockedInEmergencyMode,
    
    #[msg("Batch operation limit exceeded")]
    BatchLimitExceeded,
    
    #[msg("Invalid batch operation")]
    InvalidBatchOperation,
    
    #[msg("Feature not enabled")]
    FeatureNotEnabled,
    
    #[msg("Invalid configuration")]
    InvalidConfiguration,
    
    #[msg("Operation not allowed")]
    OperationNotAllowed,
}

