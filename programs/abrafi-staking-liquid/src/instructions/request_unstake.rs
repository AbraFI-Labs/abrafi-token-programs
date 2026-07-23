use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, Token, TokenAccount, TransferChecked},
    token_interface::{self, BurnChecked},
};

use crate::constants::*;
use crate::error::ErrorCode;
use crate::events::UnstakeRequested;
use crate::state::*;
use crate::utils::*;

/// Request to unstake liquid staking tokens (starts withdrawal delay, creates/updates UserUnstakeRequest)
#[derive(Accounts)]
pub struct RequestUnstake<'info> {
    /// Liquid staking state account
    #[account(
        mut,
        seeds = [STATE_SEED],
        bump = state.state_bump,
        has_one = liquid_staking_token_mint,
        has_one = underlying_token_mint,
    )]
    pub state: Account<'info, ProgramState>,

    /// User who wants to unstake liquid staking tokens
    #[account(mut)]
    pub user: Signer<'info>,

    /// The liquid staking token mint
    #[account(mut)]
    pub liquid_staking_token_mint: Account<'info, Mint>,

    /// The underlying token mint
    pub underlying_token_mint: Account<'info, Mint>,

    /// User's liquid staking token account
    #[account(
        mut,
        associated_token::authority = user,
        associated_token::mint = liquid_staking_token_mint,
        constraint = !user_liquid_staking_token_account.is_frozen() @ ErrorCode::AccountFrozen,
    )]
    pub user_liquid_staking_token_account: Account<'info, TokenAccount>,

    /// Vault account for storing underlying tokens
    #[account(
        mut,
        associated_token::authority = state,
        associated_token::mint = state.underlying_token_mint,
    )]
    pub vault_token_account: Account<'info, TokenAccount>,

    /// Escrow token account for holding underlying tokens during unstake process (will be created if it doesn't exist)
    #[account(
        init_if_needed,
        payer = user,
        associated_token::authority = user_unstake_request,
        associated_token::mint = underlying_token_mint,
    )]
    pub escrow_token_account: Account<'info, TokenAccount>,

    /// User's unstake request account (will be created if it doesn't exist)
    #[account(
        init_if_needed,
        payer = user,
        seeds = [USER_UNSTAKE_REQUEST_SEED, user.key().as_ref()],
        space = 8 + UserUnstakeRequest::INIT_SPACE,
        bump
    )]
    pub user_unstake_request: Account<'info, UserUnstakeRequest>,

    /// Associated token program for account creation
    pub associated_token_program: Program<'info, AssociatedToken>,

    /// Token program for transfers and minting
    pub token_program: Program<'info, Token>,

    /// System program for account creation
    pub system_program: Program<'info, System>,
}

/// Request to unstake liquid staking tokens (burns liquid staking tokens immediately, escrows underlying amount)
pub fn request_unstake_handler(ctx: Context<RequestUnstake>, liquid_staking_amount: u64) -> Result<()> {
    let state = &ctx.accounts.state;
    let user_unstake_request = &mut ctx.accounts.user_unstake_request;

    let clock = Clock::get()?;

    // Check if unstaking request is enabled
    require!(state.is_unstaking_request_enabled, ErrorCode::UnstakingDisabled);

    // Check for zero amount and if user has enough liquid staking tokens to burn
    validate_sufficient_balance(
        liquid_staking_amount,
        ctx.accounts.user_liquid_staking_token_account.amount,
        ErrorCode::InvalidAmount,
    )?;

    // Check if amount meets minimum unstake requirement
    validate_amount_meets_minimum(liquid_staking_amount, state.minimum_unstake_amount, ErrorCode::AmountBelowMinimumUnstake)?;

    let underlying_amount = convert_to_assets(
        liquid_staking_amount,
        ctx.accounts.vault_token_account.amount,
        ctx.accounts.liquid_staking_token_mint.supply,
    )?;

    // Check if vault has enough balance to escrow
    validate_sufficient_balance(
        underlying_amount,
        ctx.accounts.vault_token_account.amount,
        ErrorCode::InsufficientVaultBalance,
    )?;

    // Burn liquid staking tokens from user's account
    token_interface::burn_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            BurnChecked {
                mint: ctx.accounts.liquid_staking_token_mint.to_account_info(),
                from: ctx.accounts.user_liquid_staking_token_account.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        ),
        liquid_staking_amount,
        ctx.accounts.liquid_staking_token_mint.decimals,
    )?;

    // Reload user's liquid staking token account to get updated balance after burn
    ctx.accounts.user_liquid_staking_token_account.reload()?;

    // Ensure user's remaining balance is either zero or at least the minimum unstake amount
    let remaining_balance = ctx.accounts.user_liquid_staking_token_account.amount;
    validate_balance_zero_or_above_minimum(
        remaining_balance,
        state.minimum_unstake_amount,
        ErrorCode::AmountBelowMinimumUnstake,
    )?;

    // Transfer underlying tokens from vault to escrow account
    token::transfer_checked(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.vault_token_account.to_account_info(),
                mint: ctx.accounts.underlying_token_mint.to_account_info(),
                to: ctx.accounts.escrow_token_account.to_account_info(),
                authority: state.to_account_info(),
            },
            &[&[STATE_SEED, &[state.state_bump]]],
        ),
        underlying_amount,
        ctx.accounts.underlying_token_mint.decimals,
    )?;

    // Update user's unstake request details
    user_unstake_request.version = 1;
    user_unstake_request.request_timestamp = clock.unix_timestamp;
    // withdrawal_delay_end_timestamp is set below
    user_unstake_request.bump = ctx.bumps.user_unstake_request;

    // Reload escrow account to get updated balance after transfer
    ctx.accounts.escrow_token_account.reload()?;

    // Calculate withdrawal delay end timestamp
    user_unstake_request.withdrawal_delay_end_timestamp = safe_add_delay(
        user_unstake_request.request_timestamp,
        state.withdrawal_delay_seconds,
        ErrorCode::CalculationOverflow,
    )?;

    emit!(UnstakeRequested {
        version: 1,
        user: ctx.accounts.user.key(),
        liquid_staking_burned: liquid_staking_amount,
        underlying_amount: ctx.accounts.escrow_token_account.amount,
        request_timestamp: user_unstake_request.request_timestamp,
        withdrawal_delay_end_timestamp: user_unstake_request.withdrawal_delay_end_timestamp,
    });

    Ok(())
}
