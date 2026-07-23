use anchor_lang::prelude::*;

#[event]
pub struct ProgramInitialized {
    pub version: u8,
    pub authority: Pubkey,
    pub operations_authority: Pubkey,
    pub stake_mint: Pubkey,
    pub staking_vault: Pubkey,
}

#[event]
pub struct Staked {
    pub version: u8,
    pub user: Pubkey,
    pub amount: u64,
    pub compounded_rewards: u64,
    pub new_staked_amount: u64,
    pub total_staked: u64,
}

#[event]
pub struct YieldCompounded {
    pub version: u8,
    pub user: Pubkey,
    pub compounded_amount: u64,
    pub new_staked_amount: u64,
}

#[event]
pub struct UnstakeRequested {
    pub version: u8,
    pub user: Pubkey,
    pub amount: u64,
    pub claim_after_ts: i64,
    pub compounded_rewards: u64,
}

#[event]
pub struct UnstakeClaimed {
    pub version: u8,
    pub user: Pubkey,
    pub amount: u64,
}

#[event]
pub struct StakeEnabledChanged {
    pub version: u8,
    pub enabled: bool,
}

#[event]
pub struct UnstakeRequestEnabledChanged {
    pub version: u8,
    pub enabled: bool,
}

#[event]
pub struct UnstakeClaimEnabledChanged {
    pub version: u8,
    pub enabled: bool,
}

#[event]
pub struct WithdrawalDelayUpdated {
    pub version: u8,
    pub new_delay: i64,
}

#[event]
pub struct AuthorityUpdatePending {
    pub version: u8,
    pub current_authority: Pubkey,
    pub pending_authority: Pubkey,
    pub expiration_timestamp: i64,
}

#[event]
pub struct AuthorityUpdateCancelled {
    pub version: u8,
    pub current_authority: Pubkey,
    pub cancelled_pending_authority: Pubkey,
}

#[event]
pub struct AuthorityUpdated {
    pub version: u8,
    pub new_authority: Pubkey,
}

#[event]
pub struct OperationsAuthorityUpdated {
    pub version: u8,
    pub new_authority: Pubkey,
}

#[event]
pub struct YieldPosted {
    pub version: u8,
    pub reward_mint: Pubkey,
    pub effective_amount: u64,
    pub new_global_reward_index: u128,
}

#[event]
pub struct ClaimYieldEnabledChanged {
    pub version: u8,
    pub enabled: bool,
}
