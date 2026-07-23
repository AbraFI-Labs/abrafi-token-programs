use anchor_lang::prelude::*;

/// Event emitted when the program is initialized
#[event]
pub struct ProgramInitialized {
    pub version: u8,
    pub authority: Pubkey,
    pub compliance_authority: Pubkey,
    pub operations_authority: Pubkey,
    pub liquid_staking_token_mint: Pubkey,
    pub underlying_token_mint: Pubkey,
}

/// Authority type enum for authority update events
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum AuthorityType {
    Program,
    Compliance,
    Operations,
}

/// Event emitted when a pending authority is set (step 1 of authority transfer)
#[event]
pub struct AuthorityUpdatePending {
    pub version: u8,
    pub authority_type: AuthorityType,
    pub current_authority: Pubkey,
    pub pending_authority: Pubkey,
    pub expiration_timestamp: i64,
}

/// Event emitted when an authority transfer is finalized (step 2 of authority transfer)
#[event]
pub struct AuthorityUpdated {
    pub version: u8,
    pub authority_type: AuthorityType,
    pub new_authority: Pubkey,
}

/// Event emitted when a pending authority update is cancelled
#[event]
pub struct AuthorityUpdateCancelled {
    pub version: u8,
    pub authority_type: AuthorityType,
    pub current_authority: Pubkey,
    pub cancelled_pending_authority: Pubkey,
}

/// Event emitted when staking is toggled
#[event]
pub struct StakingEnabled {
    pub version: u8,
    pub is_enabled: bool,
}

/// Event emitted when unstaking request is toggled
#[event]
pub struct UnstakingRequestEnabled {
    pub version: u8,
    pub is_enabled: bool,
}

/// Event emitted when unstaking claim is toggled
#[event]
pub struct UnstakingClaimEnabled {
    pub version: u8,
    pub is_enabled: bool,
}

/// Event emitted when the minimum stake amount is updated
#[event]
pub struct MinimumStakeAmountUpdated {
    pub version: u8,
    pub new_minimum_amount: u64,
}

/// Event emitted when the minimum unstake amount is updated
#[event]
pub struct MinimumUnstakeAmountUpdated {
    pub version: u8,
    pub new_minimum_amount: u64,
}

/// Event emitted when the withdrawal delay is updated
#[event]
pub struct WithdrawalDelayUpdated {
    pub version: u8,
    pub new_withdrawal_delay_seconds: i64,
}

/// Event emitted whenever underlying tokens are staked for liquid staking tokens
#[event]
pub struct UnderlyingStaked {
    pub version: u8,
    pub user: Pubkey,
    pub underlying_amount: u64,
    pub liquid_staking_minted: u64,
}

/// Event emitted when an unstake is requested
#[event]
pub struct UnstakeRequested {
    pub version: u8,
    pub user: Pubkey,
    pub liquid_staking_burned: u64,
    pub underlying_amount: u64,
    pub request_timestamp: i64,
    pub withdrawal_delay_end_timestamp: i64,
}

/// Event emitted when an unstake is claimed
#[event]
pub struct UnstakeClaimed {
    pub version: u8,
    pub user: Pubkey,
    pub underlying_claimed: u64,
    pub remaining_unstake_request: u64,
}

/// Event emitted when an unstake request is restaked
#[event]
pub struct UnderlyingRestaked {
    pub version: u8,
    pub user: Pubkey,
    pub underlying_amount: u64,
    pub liquid_staking_minted: u64,
}

/// Event emitted when a token account is frozen
#[event]
pub struct AccountFrozenEvent {
    pub version: u8,
    pub reason_code: u32,
    pub token_account: Pubkey,
    pub user: Pubkey,
}

/// Event emitted when a token account is thawed
#[event]
pub struct AccountThawedEvent {
    pub version: u8,
    pub token_account: Pubkey,
    pub user: Pubkey,
}

/// Event emitted when yield is removed to external treasury
#[event]
pub struct YieldRemoved {
    pub version: u8,
    pub external_treasury: Pubkey,
    pub amount: u64,
}

/// Event emitted when external treasury is updated
#[event]
pub struct ExternalTreasuryUpdated {
    pub version: u8,
    pub authority: Pubkey,
    pub new_treasury: Pubkey,
}

/// Event emitted when the maximum removal percentage is updated
#[event]
pub struct MaxRemovalPercentageUpdated {
    pub version: u8,
    pub new_max_removal_percentage: u16,
}

/// Event emitted when a pending mint authority is set (step 1 of mint authority transfer)
#[event]
pub struct MintAuthorityTransferPending {
    pub version: u8,
    pub current_authority: Pubkey,
    pub pending_mint_authority: Pubkey,
    pub expiration_timestamp: i64,
    pub liquid_staking_token_mint: Pubkey,
}

/// Event emitted when mint authority transfer is cancelled
#[event]
pub struct MintAuthorityTransferCancelled {
    pub version: u8,
    pub current_authority: Pubkey,
    pub cancelled_pending_mint_authority: Pubkey,
    pub liquid_staking_token_mint: Pubkey,
}

/// Event emitted when mint authority is transferred (step 2: finalized)
#[event]
pub struct MintAuthorityTransferred {
    pub version: u8,
    pub new_mint_authority: Pubkey,
    pub liquid_staking_token_mint: Pubkey,
    /// Vault balance at the time of ownership transfer
    pub vault_balance: u64,
}

/// Event emitted when token metadata is created or updated
#[event]
pub struct TokenMetadataUpdated {
    pub version: u8,
    pub liquid_staking_token_mint: Pubkey,
    pub metadata_account: Pubkey,
    pub name: String,
    pub symbol: String,
    pub metadata_uri: String,
    pub was_created: bool,
}
