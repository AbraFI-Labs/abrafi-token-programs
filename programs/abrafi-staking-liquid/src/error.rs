use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    // VALIDATION ERRORS
    #[msg("Invalid amount provided")]
    InvalidAmount,
    #[msg("Calculation overflow occurred")]
    CalculationOverflow,
    #[msg("Invalid authority provided")]
    InvalidAuthority,
    #[msg("Invalid mint")]
    InvalidMint,
    #[msg("Invalid mint account provided")]
    InvalidMintAccount,
    #[msg("Invalid metadata provided")]
    InvalidMetadata,

    // VAULT AND BALANCE ERRORS
    #[msg("Insufficient vault balance")]
    InsufficientVaultBalance,
    #[msg("Insufficient liquidity")]
    InsufficientLiquidity,
    #[msg("Vault balance must be zero when first staking. Remove yield before staking")]
    InvalidVaultBalance,

    // STAKING ERRORS
    #[msg("Staking is currently disabled")]
    StakingDisabled,
    #[msg("Unstaking is currently disabled")]
    UnstakingDisabled,
    #[msg("Amount below minimum stake amount")]
    AmountBelowMinimum,

    // ACCOUNT RESTRICTION ERRORS
    #[msg("Account is frozen")]
    AccountFrozen,
    #[msg("Account is not frozen")]
    AccountNotFrozen,

    // UNSTAKING REQUEST ERRORS
    #[msg("Amount below minimum unstake amount")]
    AmountBelowMinimumUnstake,
    #[msg("Invalid minimum amount provided")]
    InvalidMinimumAmount,
    #[msg("No unstake request found for user")]
    NoUnstakeRequest,

    // UNSTAKING CLAIM ERRORS
    #[msg("Withdrawal delay not expired")]
    WithdrawalDelayNotExpired,

    // TREASURY MANAGEMENT ERRORS
    #[msg("External treasury address not set")]
    ExternalTreasuryNotSet,
    #[msg("Invalid treasury account provided")]
    InvalidTreasuryAccount,
    #[msg("Treasury update cooldown not expired")]
    TreasuryUpdateCooldownNotExpired,
    #[msg("Remove yield cooldown not expired")]
    RemoveYieldCooldownNotExpired,
    #[msg("Removal amount exceeds maximum removal percentage")]
    RemovalAmountExceedsMaxPercentage,

    // CONFIGURATION ERRORS
    #[msg("Invalid configuration value provided")]
    InvalidConfiguration,
    #[msg("Pending authority transfer has expired")]
    PendingAuthorityExpired,
}
