use anchor_lang::prelude::*;

/// Error codes for the abrafi token program
/// Common errors are duplicated here for use in constraint attributes
#[error_code]
pub enum ErrorCode {
    // TOKEN CONFIGURATION ERRORS
    #[msg("Token is already configured")]
    TokenAlreadyConfigured,
    #[msg("Token is not configured")]
    TokenNotConfigured,
    #[msg("Token is disabled and cannot be used for mint or unmint requests")]
    TokenDisabled,
    #[msg("Invalid mint account provided")]
    InvalidMintAccount,
    #[msg("Invalid treasury account provided")]
    InvalidTreasuryAccount,
    #[msg("Maximum number of collateral tokens reached (limit: 10)")]
    CapacityExceeded,
    #[msg("Invalid token name provided")]
    InvalidTokenName,
    #[msg("Invalid metadata provided")]
    InvalidMetadata,

    // MINTING ERRORS
    #[msg("Invalid mint")]
    InvalidMint,
    #[msg("Minting is currently disabled")]
    MintingDisabled,
    #[msg("Unminting is currently disabled")]
    UnmintingDisabled,
    #[msg("Invalid amount provided")]
    InvalidAmount,
    #[msg("Amount below minimum mint amount")]
    AmountBelowMinimum,
    #[msg("Insufficient collateral token balance for minting")]
    InsufficientCollateralBalance,
    #[msg("Invalid minimum amount provided")]
    InvalidMinimumAmount,
    #[msg("Operation would leave balance below minimum")]
    BalanceBelowMinimum,

    // ACCOUNT RESTRICTION ERRORS
    #[msg("Account is frozen")]
    AccountFrozen,
    #[msg("Account is not frozen")]
    AccountNotFrozen,

    // UNMINTING REQUEST ERRORS
    #[msg("Unmint request has expired and must be re-requested")]
    RequestExpired,

    // UNMINTING CLAIM ERRORS
    #[msg("Insufficient withdrawal vault balance for unminting")]
    InsufficientVaultBalance,
    #[msg("Withdrawal vault must be empty before removing collateral token")]
    VaultNotEmpty,
    #[msg("Cooldown period not expired")]
    CooldownNotExpired,

    // CALCULATION ERRORS
    #[msg("Calculation overflow occurred")]
    CalculationOverflow,

    // CONFIGURATION ERRORS
    #[msg("Invalid configuration value provided")]
    InvalidConfiguration,
    #[msg("Pending authority transfer has expired")]
    PendingAuthorityExpired,

    // WHITELIST ERRORS
    #[msg("Address is not whitelisted")]
    AddressNotWhitelisted,
    #[msg("Address is already whitelisted")]
    AddressAlreadyWhitelisted,
}
