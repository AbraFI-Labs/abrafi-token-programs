use anchor_lang::prelude::*;
use crate::constants::MAX_RECIPIENTS;

/// Global router state. PDA seeds: ["abrafi_yield_router_state"]
#[account]
#[derive(InitSpace)]
pub struct RouterState {
    pub version: u8,
    pub bump: u8,

    // Authorities
    pub authority: Pubkey,
    pub pending_authority: Pubkey,
    pub pending_authority_expiration_timestamp: i64,
    pub operations_authority: Pubkey,

    // Token this router handles (one router per yield token)
    pub yield_token_mint: Pubkey,
    /// ATA(yield_token_mint, router_state_pda) — yield accumulates here between distribution calls.
    /// Callers must first transfer yield tokens into this vault, then call distribute_yield to
    /// forward the full vault balance proportionally to registered recipients.
    pub router_vault: Pubkey,

    // Recipients
    #[max_len(MAX_RECIPIENTS)]
    pub recipients: Vec<RecipientConfig>,

    // Totals for audit
    pub total_distributed: u64,

    pub distribute_enabled: bool,
    /// Minimum vault balance required per distribute_yield call — acts as a dust guard.
    /// If vault.amount < min_distribution_amount, distribute_yield reverts with AmountBelowMinimum.
    /// Updatable by the operations authority via update_min_distribution_amount.
    pub min_distribution_amount: u64,
}

/// Configuration for a single yield recipient.
/// Nested struct — no version field (RouterState.version covers migrations).
#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub struct RecipientConfig {
    /// Destination SPL token account — receives yield transfers.
    pub destination: Pubkey,
    /// Label for audit/display; not enforced on-chain.
    pub recipient_type: RecipientType,
    /// Account used to read proportional balance at distribution time.
    /// StakingRewards: abrafi-staking-rewards ProgramState PDA (total_staked field).
    /// LiquidStaking / External: same as destination (token account amount field).
    pub balance_source: Pubkey,
    /// For StakingRewards: the deployed program ID of the staking rewards contract
    /// whose state PDA is referenced by balance_source. Stored here so one yield-router
    /// binary can serve multiple token sets (each with a different staking program ID)
    /// and to allow two staking deployments to coexist during migration.
    /// For LiquidStaking / External: unused (Pubkey::default()).
    pub staking_program_id: Pubkey,
    pub enabled: bool,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace, PartialEq)]
pub enum RecipientType {
    StakingRewards, // abrafi-staking-rewards state PDA — balance read from total_staked
    LiquidStaking,  // abrafi-staking-liquid vault — balance read from token account amount
    External,       // qualified custodian or treasury wallet — balance read from token account amount
}

/// Returns the count of enabled recipients.
pub fn enabled_recipient_count(recipients: &[RecipientConfig]) -> usize {
    recipients.iter().filter(|r| r.enabled).count()
}

