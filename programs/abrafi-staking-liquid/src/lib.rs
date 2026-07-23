use anchor_lang::prelude::*;

mod constants;
mod error;
mod events;
mod instructions;
mod state;
mod utils;

use instructions::*;

include!("declare_id.rs");

/// Generic liquid staking program (sUSDAF, sSOLAF, sBTCAF)
#[program]
pub mod abrafi_staking_liquid {
    use super::*;

    /// Initialize the liquid staking program
    /// Creates the liquid staking token mint and sets up the vault
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        initialize_handler(ctx)
    }

    pub fn stake(ctx: Context<Stake>, underlying_amount: u64) -> Result<()> {
        stake_handler(ctx, underlying_amount)
    }

    pub fn request_unstake(ctx: Context<RequestUnstake>, liquid_staking_amount: u64) -> Result<()> {
        request_unstake_handler(ctx, liquid_staking_amount)
    }

    pub fn claim_unstake(ctx: Context<ClaimUnstake>, claim_underlying_amount: u64) -> Result<()> {
        claim_unstake_handler(ctx, claim_underlying_amount)
    }

    pub fn restake(ctx: Context<Restake>, restake_underlying_amount: u64) -> Result<()> {
        restake_handler(ctx, restake_underlying_amount)
    }

    pub fn remove_yield(ctx: Context<RemoveYield>, underlying_amount: u64) -> Result<()> {
        remove_yield_handler(ctx, underlying_amount)
    }

    pub fn set_external_treasury(ctx: Context<SetExternalTreasury>) -> Result<()> {
        set_external_treasury_handler(ctx)
    }

    pub fn update_minimum_stake_amount(ctx: Context<UpdateMinimumStakeAmount>, new_minimum_amount: u64) -> Result<()> {
        update_minimum_stake_amount_handler(ctx, new_minimum_amount)
    }

    pub fn update_minimum_unstake_amount(ctx: Context<UpdateMinimumUnstakeAmount>, new_minimum_amount: u64) -> Result<()> {
        update_minimum_unstake_amount_handler(ctx, new_minimum_amount)
    }

    pub fn update_max_removal_percentage(ctx: Context<UpdateMaxRemovalPercentage>, new_max_removal_percentage: u16) -> Result<()> {
        update_max_removal_percentage_handler(ctx, new_max_removal_percentage)
    }

    pub fn update_withdrawal_delay(ctx: Context<UpdateWithdrawalDelay>, new_delay_seconds: i64) -> Result<()> {
        update_withdrawal_delay_handler(ctx, new_delay_seconds)
    }

    pub fn update_token_metadata(ctx: Context<UpdateTokenMetadata>, name: String, symbol: String, metadata_uri: String) -> Result<()> {
        update_token_metadata_handler(ctx, name, symbol, metadata_uri)
    }

    pub fn transfer_mint_authority(ctx: Context<TransferMintAuthority>) -> Result<()> {
        transfer_mint_authority_handler(ctx)
    }

    pub fn finalize_mint_authority(ctx: Context<FinalizeMintAuthority>) -> Result<()> {
        finalize_mint_authority_handler(ctx)
    }

    pub fn update_authority(ctx: Context<UpdateAuthority>) -> Result<()> {
        update_authority_handler(ctx)
    }

    pub fn finalize_authority(ctx: Context<FinalizeAuthority>) -> Result<()> {
        finalize_authority_handler(ctx)
    }

    pub fn update_operations_authority(ctx: Context<UpdateAuthority>) -> Result<()> {
        update_operations_authority_handler(ctx)
    }

    pub fn update_compliance_authority(ctx: Context<UpdateAuthority>) -> Result<()> {
        update_compliance_authority_handler(ctx)
    }

    pub fn freeze_account(ctx: Context<FreezeAccount>, reason_code: u32) -> Result<()> {
        freeze_account_handler(ctx, reason_code)
    }

    pub fn thaw_account(ctx: Context<ThawAccount>) -> Result<()> {
        thaw_account_handler(ctx)
    }

    pub fn set_staking_enabled(ctx: Context<SetStakingEnabled>, enabled: bool) -> Result<()> {
        set_staking_enabled_handler(ctx, enabled)
    }

    pub fn set_unstaking_request_enabled(ctx: Context<SetUnstakingRequestEnabled>, enabled: bool) -> Result<()> {
        set_unstaking_request_enabled_handler(ctx, enabled)
    }

    pub fn set_unstaking_claim_enabled(ctx: Context<SetUnstakingClaimEnabled>, enabled: bool) -> Result<()> {
        set_unstaking_claim_enabled_handler(ctx, enabled)
    }

    pub fn calculate_stake_amount(ctx: Context<GetConversionRate>, underlying_amount: u64) -> Result<u64> {
        calculate_stake_amount_handler(ctx, underlying_amount)
    }

    pub fn calculate_unstake_amount(ctx: Context<GetConversionRate>, liquid_staking_amount: u64) -> Result<u64> {
        calculate_unstake_amount_handler(ctx, liquid_staking_amount)
    }

    #[cfg(feature = "test")]
    pub fn set_pending_authority_timestamp_for_testing(
        ctx: Context<SetPendingAuthorityTimestampForTesting>,
        new_timestamp: i64,
    ) -> Result<()> {
        set_pending_authority_timestamp_for_testing_handler(ctx, new_timestamp)
    }

    #[cfg(feature = "test")]
    pub fn set_pending_mint_authority_timestamp_for_testing(
        ctx: Context<SetPendingMintAuthorityTimestampForTesting>,
        new_timestamp: i64,
    ) -> Result<()> {
        set_pending_mint_authority_timestamp_for_testing_handler(ctx, new_timestamp)
    }

    #[cfg(feature = "test")]
    pub fn set_remove_yield_timestamp_for_testing(
        ctx: Context<SetRemoveYieldTimestampForTesting>,
        new_timestamp: i64,
    ) -> Result<()> {
        set_remove_yield_timestamp_for_testing_handler(ctx, new_timestamp)
    }

    #[cfg(feature = "test")]
    pub fn set_treasury_timestamp_for_testing(
        ctx: Context<SetTreasuryTimestampForTesting>,
        new_timestamp: i64,
    ) -> Result<()> {
        set_treasury_timestamp_for_testing_handler(ctx, new_timestamp)
    }
}
