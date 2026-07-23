use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, Token, TokenAccount, TransferChecked},
};

use crate::constants::*;
use crate::error::ErrorCode;
use crate::events::UnstakeClaimed;
use crate::state::*;
use crate::utils::*;

/// Claim unstake instruction accounts
#[derive(Accounts)]
pub struct ClaimUnstake<'info> {
    /// Liquid staking state account
    #[account(
        mut,
        seeds = [STATE_SEED],
        bump = state.state_bump,
        has_one = underlying_token_mint,
        has_one = liquid_staking_token_mint,
    )]
    pub state: Account<'info, ProgramState>,

    /// User who wants to claim unstaked underlying tokens
    #[account(mut)]
    pub user: Signer<'info>,

    /// Underlying token mint
    pub underlying_token_mint: Account<'info, Mint>,

    /// Liquid staking token mint
    #[account(mut)]
    pub liquid_staking_token_mint: Account<'info, Mint>,

    /// User's liquid staking token account (checked for frozen status)
    #[account(
        mut,
        associated_token::authority = user,
        associated_token::mint = liquid_staking_token_mint,
        constraint = !user_liquid_staking_token_account.is_frozen() @ ErrorCode::AccountFrozen,
    )]
    pub user_liquid_staking_token_account: Account<'info, TokenAccount>,

    /// User's underlying token account (destination for claimed tokens)
    #[account(
        mut,
        associated_token::authority = user,
        associated_token::mint = underlying_token_mint,
        constraint = !user_underlying_token_account.is_frozen() @ ErrorCode::AccountFrozen,
    )]
    pub user_underlying_token_account: Account<'info, TokenAccount>,

    /// Escrow token account for holding underlying tokens during unstake process
    #[account(
        mut,
        associated_token::authority = user_unstake_request,
        associated_token::mint = underlying_token_mint,
    )]
    pub escrow_token_account: Account<'info, TokenAccount>,

    /// User's unstake request account
    #[account(
        mut,
        seeds = [USER_UNSTAKE_REQUEST_SEED, user.key().as_ref()],
        bump = user_unstake_request.bump,
    )]
    pub user_unstake_request: Account<'info, UserUnstakeRequest>,

    /// Associated token program for account closure
    pub associated_token_program: Program<'info, AssociatedToken>,

    /// Token program for transfers
    pub token_program: Program<'info, Token>,
}

/// Claim unstaked underlying tokens after withdrawal delay
pub fn claim_unstake_handler(ctx: Context<ClaimUnstake>, claim_underlying_amount: u64) -> Result<()> {
    let state = &ctx.accounts.state;
    let user_unstake_request = &mut ctx.accounts.user_unstake_request;

    let clock = Clock::get()?;

    // Check if unstaking claim is enabled
    require!(state.is_unstaking_claim_enabled, ErrorCode::UnstakingDisabled);

    let escrow_balance = ctx.accounts.escrow_token_account.amount;

    // Check for zero amount and escrow balance
    validate_sufficient_balance(claim_underlying_amount, escrow_balance, ErrorCode::InvalidAmount)?;

    // Check if claim amount is either the entire escrow balance or at least the minimum unstake amount
    validate_amount_full_or_above_minimum(
        claim_underlying_amount,
        escrow_balance,
        state.minimum_unstake_amount,
        ErrorCode::AmountBelowMinimumUnstake,
    )?;

    // Ensure withdrawal delay has passed
    validate_timestamp_has_passed(
        clock.unix_timestamp,
        user_unstake_request.withdrawal_delay_end_timestamp,
        ErrorCode::WithdrawalDelayNotExpired,
    )?;

    // Transfer underlying tokens from escrow to user using UserUnstakeRequest PDA as authority
    token::transfer_checked(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.escrow_token_account.to_account_info(),
                mint: ctx.accounts.underlying_token_mint.to_account_info(),
                to: ctx.accounts.user_underlying_token_account.to_account_info(),
                authority: user_unstake_request.to_account_info(),
            },
            &[&[
                USER_UNSTAKE_REQUEST_SEED,
                ctx.accounts.user.key().as_ref(),
                &[user_unstake_request.bump],
            ]],
        ),
        claim_underlying_amount,
        ctx.accounts.underlying_token_mint.decimals,
    )?;

    // Reload escrow account to get updated balance after transfer
    ctx.accounts.escrow_token_account.reload()?;

    // Ensure remaining escrow balance is either zero or at least the minimum unstake amount
    let remaining_escrow = ctx.accounts.escrow_token_account.amount;
    validate_balance_zero_or_above_minimum(
        remaining_escrow,
        state.minimum_unstake_amount,
        ErrorCode::AmountBelowMinimumUnstake,
    )?;

    // Close the accounts if the escrow is empty
    if remaining_escrow == 0 {
        close_escrow_token_account(
            ctx.accounts.token_program.to_account_info(),
            &ctx.accounts.escrow_token_account,
            ctx.accounts.user.to_account_info(),
            user_unstake_request.to_account_info(),
            &ctx.accounts.user.key(),
            user_unstake_request.bump,
        )?;

        user_unstake_request.close(ctx.accounts.user.to_account_info())?;
    }

    emit!(UnstakeClaimed {
        version: 1,
        user: ctx.accounts.user.key(),
        underlying_claimed: claim_underlying_amount,
        remaining_unstake_request: remaining_escrow,
    });

    Ok(())
}
