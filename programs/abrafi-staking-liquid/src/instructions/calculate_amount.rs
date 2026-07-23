use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount};

use crate::constants::*;
use crate::state::*;
use crate::utils::*;

/// Get conversion rate (read-only instruction)
#[derive(Accounts)]
pub struct GetConversionRate<'info> {
    /// Liquid staking state account (read-only)
    #[account(
        seeds = [STATE_SEED],
        bump = state.state_bump,
        has_one = liquid_staking_token_mint,
    )]
    pub state: Account<'info, ProgramState>,

    /// Liquid staking token mint (required for conversion calculations)
    pub liquid_staking_token_mint: Account<'info, Mint>,

    /// Vault account for storing underlying tokens
    #[account(
        associated_token::authority = state,
        associated_token::mint = state.underlying_token_mint,
    )]
    pub vault_token_account: Account<'info, TokenAccount>,
}

/// Calculate expected liquid staking token amount for staking a given amount of underlying tokens
pub fn calculate_stake_amount_handler(
    ctx: Context<GetConversionRate>,
    underlying_amount: u64,
) -> Result<u64> {

    // Return zero immediately for zero input
    if underlying_amount == 0 {
        return Ok(0);
    }

    // Use the same conversion logic as the actual stake operation
    let expected_liquid_staking_amount = convert_to_shares(
        underlying_amount,
        ctx.accounts.vault_token_account.amount,
        ctx.accounts.liquid_staking_token_mint.supply,
    )?;

    Ok(expected_liquid_staking_amount)
}

/// Calculate expected underlying token amount for unstaking a given amount of liquid staking tokens
pub fn calculate_unstake_amount_handler(
    ctx: Context<GetConversionRate>,
    liquid_staking_amount: u64,
) -> Result<u64> {

    // Return zero immediately for zero input
    if liquid_staking_amount == 0 {
        return Ok(0);
    }

    // Use the same conversion logic as the actual unstake operation
    let expected_underlying_amount = convert_to_assets(
        liquid_staking_amount,
        ctx.accounts.vault_token_account.amount,
        ctx.accounts.liquid_staking_token_mint.supply,
    )?;

    Ok(expected_underlying_amount)
}
