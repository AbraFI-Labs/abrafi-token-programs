use anchor_lang::prelude::*;

/// State account for the liquid staking program
#[account]
#[derive(InitSpace)]
pub struct ProgramState {
    /// Version for future compatibility
    pub version: u8,
    /// Authority that can add yield and update configuration
    pub authority: Pubkey,
    /// Pending authority waiting for acceptance (set by current authority)
    pub pending_authority: Pubkey,
    /// Timestamp when pending_authority expires (must be finalized before this time)
    pub pending_authority_expiration_timestamp: i64,
    /// Operations authority for managing specific operations
    pub operations_authority: Pubkey,
    /// Compliance authority for compliance-related operations
    pub compliance_authority: Pubkey,

    /// Mint address of the underlying token
    pub underlying_token_mint: Pubkey,
    /// Mint address of the liquid staking token
    pub liquid_staking_token_mint: Pubkey,

    /// Bump seed for the state account PDA
    pub state_bump: u8,

    /// Boolean indicating if staking is enabled
    pub is_staking_enabled: bool,
    /// Boolean indicating if requesting unstaking is enabled
    pub is_unstaking_request_enabled: bool,
    /// Boolean indicating if claiming unstaking is enabled
    pub is_unstaking_claim_enabled: bool,
    /// Minimum amount that can be staked
    pub minimum_stake_amount: u64,
    /// Minimum amount that can be unstaked
    pub minimum_unstake_amount: u64,
    /// Maximum percentage of vault balance that can be removed per remove_yield call (in basis points, default: 500 = 5%)
    pub max_removal_percentage: u16,

    /// External treasury address for underlying tokens (can be set by authority)
    pub external_treasury_token_account: Pubkey,
    /// Timestamp when the treasury update cooldown ends (allows yield removal after this time)
    pub treasury_cooldown_end_timestamp: i64,
    /// Timestamp when the remove yield cooldown ends (allows next yield removal after this time)
    pub remove_yield_cooldown_end_timestamp: i64,

    /// Unstake delay for unstaking (default: 604800 = 7 days)
    pub withdrawal_delay_seconds: i64,

    /// Pending mint authority waiting for acceptance (set by current authority)
    pub pending_mint_authority: Pubkey,
    /// Timestamp when pending_mint_authority expires (must be finalized before this time)
    pub pending_mint_authority_expiration_timestamp: i64,
}

/// Per-user account to track unstake requests
#[account]
#[derive(InitSpace)]
pub struct UserUnstakeRequest {
    /// Version for future compatibility
    pub version: u8,
    /// Timestamp when the request was created
    pub request_timestamp: i64,
    /// Timestamp when the withdrawal delay ends (request_timestamp + unstake_cooldown_seconds)
    pub withdrawal_delay_end_timestamp: i64,
    /// Bump seed for the user unstake request account PDA
    pub bump: u8,
}
