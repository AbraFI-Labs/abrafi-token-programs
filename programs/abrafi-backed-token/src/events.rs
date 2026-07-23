// programs/abrafi-backed-token/src/events.rs
// All event definitions for the abrafi token program

use anchor_lang::prelude::*;

/// Event emitted when the program is initialized
#[event]
pub struct ProgramInitialized {
    pub version: u8,
    pub authority: Pubkey,
    pub compliance_authority: Pubkey,
    pub operations_authority: Pubkey,
    pub abrafi_backed_token_mint: Pubkey,
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

/// Event emitted when minting is toggled
#[event]
pub struct MintingEnabled {
    pub version: u8,
    pub is_enabled: bool,
}

/// Event emitted when unminting request is toggled
#[event]
pub struct UnmintingRequestEnabled {
    pub version: u8,
    pub is_enabled: bool,
}

/// Event emitted when unminting claim is toggled
#[event]
pub struct UnmintingClaimEnabled {
    pub version: u8,
    pub is_enabled: bool,
}

/// Event emitted when mint whitelist is toggled
#[event]
pub struct MintWhitelistEnabled {
    pub version: u8,
    pub is_enabled: bool,
}

/// Event emitted when the minimum mint amount is updated
#[event]
pub struct MinimumMintAmountUpdated {
    pub version: u8,
    pub new_minimum_amount: u64,
}

/// Event emitted when the minimum unmint amount is updated
#[event]
pub struct MinimumUnmintAmountUpdated {
    pub version: u8,
    pub new_minimum_amount: u64,
}

/// Event emitted when the unmint cooldown period is updated
#[event]
pub struct UnmintCooldownUpdated {
    pub version: u8,
    pub new_cooldown_seconds: i64,
}

/// Event emitted when the request expiration period is updated
#[event]
pub struct RequestExpirationUpdated {
    pub version: u8,
    pub new_expiration_seconds: i64,
}

/// Event emitted when abrafi tokens are minted
#[event]
pub struct TokensMinted {
    pub version: u8,
    pub collateral_mint: Pubkey,
    pub user_abrafi_backed_token_account: Pubkey,
    pub abrafi_backed_token_amount: u64,
    pub collateral_amount: u64,
    pub user: Pubkey,
}

/// Event emitted when a user requests an unmint
#[event]
pub struct UnmintRequested {
    pub version: u8,
    pub user: Pubkey,
    pub requested_amount: u64,
    pub request_timestamp: i64,
    pub withdrawal_delay_end_timestamp: i64,
    pub claim_token_mint: Pubkey,
    pub escrow_account: Pubkey,
    pub request_expiration_timestamp: i64,
}

/// Event emitted when a user claims unminted tokens
#[event]
pub struct UnmintClaimed {
    pub version: u8,
    pub user: Pubkey,
    pub claimed_amount: u64,
    pub collateral_amount: u64,
    pub claim_token_mint: Pubkey,
    pub user_claim_token_account: Pubkey,
}

/// Event emitted when a user's unmint request is cancelled (expired or explicit)
#[event]
pub struct UnmintCancelled {
    pub version: u8,
    pub cancelled_amount: u64,
    pub user: Pubkey,
}

/// Event emitted when a collateral token is added
#[event]
pub struct CollateralTokenAdded {
    pub version: u8,
    pub name: String,
    pub num_collateral_tokens: u8,
    pub token_mint: Pubkey,
    pub treasury_token_account: Pubkey,
    pub vault_token_account: Pubkey,
}

/// Event emitted when a collateral token is disabled
#[event]
pub struct CollateralTokenDisabled {
    pub version: u8,
    pub name: String,
    pub token_mint: Pubkey,
}

/// Event emitted when a collateral token is removed
#[event]
pub struct CollateralTokenRemoved {
    pub version: u8,
    pub name: String,
    pub num_collateral_tokens: u8,
    pub token_mint: Pubkey,
    pub treasury_token_account: Pubkey,
}

/// Event emitted when tokens are withdrawn from a vault
#[event]
pub struct VaultWithdrawal {
    pub version: u8,
    pub name: String,
    pub amount: u64,
    pub withdraw_token_mint: Pubkey,
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

/// Event emitted when a pending mint authority is set (step 1 of mint authority transfer)
#[event]
pub struct MintAuthorityTransferPending {
    pub version: u8,
    pub current_authority: Pubkey,
    pub pending_mint_authority: Pubkey,
    pub expiration_timestamp: i64,
    pub abrafi_backed_token_mint: Pubkey,
}

/// Event emitted when mint authority transfer is cancelled
#[event]
pub struct MintAuthorityTransferCancelled {
    pub version: u8,
    pub current_authority: Pubkey,
    pub cancelled_pending_mint_authority: Pubkey,
    pub abrafi_backed_token_mint: Pubkey,
}

/// Event emitted when mint authority is transferred (step 2: finalized)
#[event]
pub struct MintAuthorityTransferred {
    pub version: u8,
    pub new_mint_authority: Pubkey,
    pub abrafi_backed_token_mint: Pubkey,
}

/// Event emitted when an address is added to the mint whitelist
#[event]
pub struct MintWhitelistAdded {
    pub version: u8,
    pub user: Pubkey,
    /// None = use global cooldown; Some(n) = custom cooldown in seconds (active after 24h)
    pub custom_cooldown_seconds: Option<i64>,
    /// None when no custom cooldown is set; Some(ts) = Unix timestamp when the custom cooldown activates
    pub custom_cooldown_effective_timestamp: Option<i64>,
}

/// Event emitted when an address is removed from the mint whitelist
#[event]
pub struct MintWhitelistRemoved {
    pub version: u8,
    pub user: Pubkey,
}

/// Event emitted when token metadata is created or updated
#[event]
pub struct TokenMetadataUpdated {
    pub version: u8,
    pub abrafi_backed_token_mint: Pubkey,
    pub metadata_account: Pubkey,
    pub name: String,
    pub symbol: String,
    pub metadata_uri: String,
    pub was_created: bool,
}
