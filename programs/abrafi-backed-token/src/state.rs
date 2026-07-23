
use anchor_lang::prelude::*;

use crate::constants::*;

/// Configuration for a collateral token including its treasury address
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Debug, InitSpace)]
pub struct CollateralTokenConfig {
    /// Version for future compatibility
    pub version: u8,
    /// Human-readable name of the collateral token
    #[max_len(MAX_TOKEN_NAME_LEN)]
    pub name: String,
    /// Mint address of the collateral token
    pub token_mint: Pubkey,
    /// Treasury address where tokens will be sent when minting abrafi token
    pub treasury_token_account: Pubkey,
    /// Whether this token is disabled
    /// Disabled tokens cannot be used for new mint or unmint requests
    pub disabled: bool,
}

/// Program state account structure
#[account]
#[derive(Default, InitSpace)]
pub struct ProgramState {
    /// Version for future compatibility
    pub version: u8,
    /// Authority that can update program configuration
    pub authority: Pubkey,
    /// Pending authority waiting for acceptance (set by current authority)
    pub pending_authority: Pubkey,
    /// Timestamp when pending_authority expires (must be finalized before this time)
    pub pending_authority_expiration_timestamp: i64,
    /// Operations authority for managing specific operations
    pub operations_authority: Pubkey,
    /// Compliance authority for compliance-related operations
    pub compliance_authority: Pubkey,
    /// This field is not used but is kept to maintain compatibility
    pub _unused: Pubkey,

    /// Mint address of the abrafi token
    pub abrafi_backed_token_mint: Pubkey,
    /// List of collateral tokens and their treasury addresses
    #[max_len(MAX_COLLATERAL_TOKENS)]
    pub collateral_tokens: Vec<CollateralTokenConfig>,

    /// Bump seed for the state account PDA
    pub state_bump: u8,
    /// Bump seed for the vault authority PDA
    pub vault_authority_bump: u8,

    /// Boolean indicating if minting is enabled
    pub is_minting_enabled: bool,
    /// Boolean indicating if requesting unminting is enabled
    pub is_unminting_request_enabled: bool,
    /// Boolean indicating if claiming unminting is enabled
    pub is_unminting_claim_enabled: bool,
    /// Boolean indicating if mint whitelist checks are enabled
    pub is_mint_whitelist_enabled: bool,
    /// Minimum amount that can be minted
    pub minimum_mint_amount: u64,
    /// Minimum amount that can be unminted
    pub minimum_unmint_amount: u64,

    /// Unmint cooldown period in seconds
    pub unmint_cooldown_seconds: i64,
    /// Request expiration period in seconds
    pub request_expiration_seconds: i64,

    /// Pending mint authority waiting for acceptance (set by current authority)
    pub pending_mint_authority: Pubkey,
    /// Timestamp when pending_mint_authority expires (must be finalized before this time)
    pub pending_mint_authority_expiration_timestamp: i64,
}

/// User account structure to store unmint details in addition to the escrow token account
#[account]
#[derive(Default, InitSpace)]
pub struct UserUnmintDetails {
    /// Version for future compatibility
    pub version: u8,
    /// Token mint address to claim (which collateral token)
    pub claim_token_mint: Pubkey,
    /// Timestamp when the request was created
    pub request_timestamp: i64,
    /// Timestamp when the withdrawal delay ends (request_timestamp + unmint_cooldown_seconds)
    pub withdrawal_delay_end_timestamp: i64,
    /// Timestamp when the request expires (withdrawal_delay_end_timestamp + request_expiration_seconds)
    pub request_expiration_timestamp: i64,
    /// Bump seed for the user details account PDA
    pub bump: u8,
}

/// Mint whitelist entry account structure
/// Each whitelisted address has its own PDA account to support unlimited growth
/// This whitelist restricts minting and unminting operations
#[account]
#[derive(InitSpace)]
pub struct MintWhitelistEntry {
    /// Version for future compatibility
    pub version: u8,
    /// Timestamp when the address was added to the mint whitelist
    pub added_timestamp: i64,
    /// Whether a custom unmint cooldown has been configured for this address.
    /// When false, the global unmint_cooldown_seconds from ProgramState applies.
    pub has_custom_cooldown: bool,
    /// Custom unmint cooldown duration in seconds.
    /// Zero is permitted — allows instant withdrawal for trusted automation addresses.
    /// Only meaningful when has_custom_cooldown is true.
    pub custom_cooldown_seconds: i64,
    /// Unix timestamp when the custom cooldown becomes effective.
    /// Set to added_timestamp + CUSTOM_COOLDOWN_ACTIVATION_DELAY (24 hours) when configured.
    /// Until this timestamp passes, the global cooldown is used even if has_custom_cooldown is true.
    pub custom_cooldown_effective_timestamp: i64,
}
