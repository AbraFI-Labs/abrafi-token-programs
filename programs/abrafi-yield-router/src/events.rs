use anchor_lang::prelude::*;
use crate::state::RecipientType;

#[event]
pub struct RouterInitialized {
    pub version: u8,
    pub authority: Pubkey,
    pub operations_authority: Pubkey,
    pub yield_token_mint: Pubkey,
    pub router_vault: Pubkey,
    pub min_distribution_amount: u64,
}

#[event]
pub struct RecipientAdded {
    pub version: u8,
    pub index: u8,
    pub destination: Pubkey,
    pub balance_source: Pubkey,
    pub staking_program_id: Pubkey,
    pub recipient_type: RecipientType,
}

#[event]
pub struct RecipientEnabledUpdated {
    pub version: u8,
    pub index: u8,
    pub enabled: bool,
}

#[event]
pub struct YieldDistributed {
    pub version: u8,
    /// Vault balance at the start of this distribution round (before any transfers).
    /// Dust (vault_balance - transferred) stays in the vault for the next round.
    pub vault_balance: u64,
    /// Total actually transferred to recipients this round (sum of amounts_per_recipient).
    pub transferred: u64,
    pub total_distributed: u64,
    /// Per-recipient amounts in Vec order; 0 for disabled or zero-balance recipients.
    pub amounts_per_recipient: Vec<u64>,
}

#[event]
pub struct DistributeEnabledUpdated {
    pub version: u8,
    pub enabled: bool,
}

#[event]
pub struct AuthorityUpdateProposed {
    pub version: u8,
    pub pending_authority: Pubkey,
    pub expiration_timestamp: i64,
}

#[event]
pub struct AuthorityUpdated {
    pub version: u8,
    pub old_authority: Pubkey,
    pub new_authority: Pubkey,
}

#[event]
pub struct OperationsAuthorityUpdated {
    pub version: u8,
    pub old_operations_authority: Pubkey,
    pub new_operations_authority: Pubkey,
}

#[event]
pub struct MinDistributionAmountUpdated {
    pub version: u8,
    pub old_amount: u64,
    pub new_amount: u64,
}
