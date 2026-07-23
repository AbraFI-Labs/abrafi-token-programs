use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, TransferChecked};

use crate::constants::*;
use crate::error::ErrorCode;
use crate::events::YieldRemoved;
use crate::state::*;
use crate::utils::*;

/// Remove yield to external treasury (operations authority only)
#[derive(Accounts)]
pub struct RemoveYield<'info> {
    /// Liquid staking state account
    #[account(
        mut,
        seeds = [STATE_SEED],
        bump = state.state_bump,
        has_one = operations_authority,
        has_one = external_treasury_token_account,
        has_one = underlying_token_mint,
    )]
    pub state: Account<'info, ProgramState>,

    /// Operations authority that can remove yield
    pub operations_authority: Signer<'info>,

    /// The underlying token mint
    pub underlying_token_mint: Account<'info, Mint>,

    /// Vault account for storing underlying tokens
    #[account(
        mut,
        associated_token::authority = state,
        associated_token::mint = underlying_token_mint,
    )]
    pub vault_token_account: Account<'info, TokenAccount>,

    /// External treasury account for receiving underlying tokens
    #[account(
        mut,
        constraint = external_treasury_token_account.mint == underlying_token_mint.key() @ ErrorCode::InvalidTreasuryAccount,
    )]
    pub external_treasury_token_account: Account<'info, TokenAccount>,

    /// Token program for transfers
    pub token_program: Program<'info, Token>,
}

/// Remove yield by transferring underlying tokens to external treasury (operations authority only)
/// Requires 24 hours to have elapsed since treasury address was last updated
/// Requires 24 hours to have elapsed since last remove_yield call
/// Limits removal amount to max_removal_percentage of vault balance
pub fn remove_yield_handler(ctx: Context<RemoveYield>, underlying_amount: u64) -> Result<()> {
    let state = &mut ctx.accounts.state;
    let current_timestamp = Clock::get()?.unix_timestamp;
    let vault_balance = ctx.accounts.vault_token_account.amount;

    // Check if external treasury address is set
    require!(
        state.external_treasury_token_account != Pubkey::default(),
        ErrorCode::ExternalTreasuryNotSet
    );

    // Check if 24 hours have elapsed since treasury update
    validate_timestamp_has_passed(
        current_timestamp,
        state.treasury_cooldown_end_timestamp,
        ErrorCode::TreasuryUpdateCooldownNotExpired,
    )?;

    // Check if 24 hours have elapsed since last remove_yield call
    validate_timestamp_has_passed(
        current_timestamp,
        state.remove_yield_cooldown_end_timestamp,
        ErrorCode::RemoveYieldCooldownNotExpired,
    )?;

    // Check for zero amount and if vault has enough underlying tokens
    validate_sufficient_balance(
        underlying_amount,
        vault_balance,
        ErrorCode::InsufficientVaultBalance,
    )?;

    // Check that removal amount doesn't exceed max_removal_percentage of vault balance
    // Calculate max allowed amount: (vault_balance * max_removal_percentage) / 10000
    // max_removal_percentage is in basis points (0 = disabled, 500 = 5%, 10000 = 100%)
    // If max_removal_percentage is 0, max_allowed_amount will be 0, effectively disabling removal
    let max_allowed_amount = (vault_balance as u128)
        .checked_mul(state.max_removal_percentage as u128)
        .ok_or(ErrorCode::CalculationOverflow)?
        .checked_div(10000u128)
        .ok_or(ErrorCode::CalculationOverflow)?;
    let max_allowed_amount_u64 = u64::try_from(max_allowed_amount)
        .map_err(|_| ErrorCode::CalculationOverflow)?;

    require!(
        underlying_amount <= max_allowed_amount_u64,
        ErrorCode::RemovalAmountExceedsMaxPercentage
    );

    // Transfer underlying tokens from vault to external treasury
    let state_bump = state.state_bump;
    token::transfer_checked(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.vault_token_account.to_account_info(),
                mint: ctx.accounts.underlying_token_mint.to_account_info(),
                to: ctx
                    .accounts
                    .external_treasury_token_account
                    .to_account_info(),
                authority: state.to_account_info(),
            },
            &[&[STATE_SEED, &[state_bump]]],
        ),
        underlying_amount,
        ctx.accounts.underlying_token_mint.decimals,
    )?;

    // Update remove_yield_cooldown_end_timestamp to current timestamp + cooldown period
    state.remove_yield_cooldown_end_timestamp = current_timestamp
        .checked_add(REMOVE_YIELD_COOLDOWN_SECONDS)
        .ok_or(ErrorCode::CalculationOverflow)?;

    emit!(YieldRemoved {
        version: 1,
        external_treasury: ctx.accounts.external_treasury_token_account.key(),
        amount: underlying_amount,
    });

    Ok(())
}
