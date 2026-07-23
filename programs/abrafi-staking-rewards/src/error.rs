use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    // STAKE / UNSTAKE ERRORS
    #[msg("Staking is currently disabled")]
    StakingDisabled,
    #[msg("Unstake requests are currently disabled")]
    UnstakeRequestDisabled,
    #[msg("Unstake claims are currently disabled")]
    UnstakeClaimDisabled,
    #[msg("Invalid amount provided")]
    InvalidAmount,
    #[msg("Insufficient staked balance")]
    InsufficientStakedBalance,
    #[msg("Withdrawal delay has not elapsed yet")]
    WithdrawalDelayNotElapsed,

    // ACCOUNT RESTRICTION ERRORS
    #[msg("Token account is frozen")]
    AccountFrozen,

    // CONFIGURATION ERRORS
    #[msg("Invalid configuration value provided")]
    InvalidConfiguration,
    #[msg("Value must differ from the current setting")]
    NoChange,
    #[msg("Pending authority transfer has expired")]
    PendingAuthorityExpired,
    #[msg("No pending authority transfer to finalize")]
    NoPendingAuthorityTransfer,

    // CALCULATION ERRORS
    #[msg("Calculation overflow occurred")]
    CalculationOverflow,
    #[msg("Reward index invariant violated: user snapshot exceeds global index")]
    RewardIndexInvariantViolated,

    // YIELD ERRORS
    #[msg("Claim yield is currently disabled")]
    ClaimYieldDisabled,
    #[msg("No stakers available to receive yield")]
    NoStakersToReceiveYield,
    #[msg("No pending rewards to claim")]
    NoPendingRewards,
}
