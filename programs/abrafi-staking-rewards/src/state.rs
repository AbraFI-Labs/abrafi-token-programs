use anchor_lang::prelude::*;

/// Global state. PDA seeds: ["abrafi_staking_rewards_state"]
#[account]
#[derive(InitSpace)]
pub struct ProgramState {
    pub version: u8,
    pub bump: u8,
    pub authority: Pubkey,
    pub pending_authority: Pubkey,
    pub pending_authority_expiration_timestamp: i64,
    pub operations_authority: Pubkey,
    pub stake_mint: Pubkey,
    /// ATA of the state PDA for stake_mint. Holds all staked tokens and in-kind reward tokens.
    pub staking_vault: Pubkey,
    /// abrafi-yield-router reads this field directly to determine the portion of yield
    /// that should be credited to the staking rewards contract. Do not rename or reorder
    /// without updating the router's read_balance deserialization.
    pub total_staked: u64,
    /// Yield posted but not yet compounded into staked positions.
    /// Vault invariant: staking_vault.amount == total_staked + total_yield_allocated
    ///                                        + total_pending_unstake + dust
    pub total_yield_allocated: u64,
    /// Tokens reserved for unstake requests that have not yet been claimed.
    pub total_pending_unstake: u64,
    /// Cumulative reward-per-staked-token accumulator, scaled by PRECISION_FACTOR (10^18).
    pub global_reward_index: u128,
    /// Seconds a user must wait between request_unstake and claim_unstake.
    pub withdrawal_delay: i64,
    pub stake_enabled: bool,
    pub unstake_request_enabled: bool,
    pub unstake_claim_enabled: bool,
    pub claim_yield_enabled: bool,
}

/// Per-user staking position. PDA seeds: ["user_stake", user.key()]
#[account]
#[derive(InitSpace)]
pub struct UserStakeAccount {
    pub version: u8,
    pub bump: u8,
    pub user: Pubkey,
    pub staked_amount: u64,
    pub reward_index_snapshot: u128,
    pub pending_rewards: u64,
}

/// Active unstake request. PDA seeds: ["unstake_request", user.key()]
#[account]
#[derive(InitSpace)]
pub struct UnstakeRequest {
    pub version: u8,
    pub bump: u8,
    pub user: Pubkey,
    pub amount: u64,
    pub claim_after_ts: i64,
}
