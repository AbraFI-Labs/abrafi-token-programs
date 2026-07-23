use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, Token, TokenAccount, TransferChecked},
};

use crate::constants::*;
use crate::error::ErrorCode;
use crate::events::*;
use crate::state::*;
use crate::utils::*;

/// Request unmint instruction accounts
#[derive(Accounts)]
pub struct RequestUnmint<'info> {
    /// Program state account
    #[account(
        mut,
        seeds = [STATE_SEED],
        bump = state.state_bump,
        has_one = abrafi_backed_token_mint,
    )]
    pub state: Account<'info, ProgramState>,

    /// User who wants to unmint abrafi tokens
    #[account(mut)]
    pub user: Signer<'info>,

    /// The abrafi token mint
    pub abrafi_backed_token_mint: Account<'info, Mint>,

    /// Claim token mint (the collateral token the user wants to receive)
    pub claim_token_mint: Account<'info, Mint>,

    /// User's abrafi token account
    #[account(
        mut,
        associated_token::authority = user,
        associated_token::mint = abrafi_backed_token_mint,
        constraint = !user_abrafi_backed_token_account.is_frozen() @ ErrorCode::AccountFrozen,
    )]
    pub user_abrafi_backed_token_account: Account<'info, TokenAccount>,

    /// Escrow token account for holding abrafi tokens during unmint process (will be created if it doesn't exist)
    #[account(
        init_if_needed,
        payer = user,
        associated_token::authority = user_unmint_details,
        associated_token::mint = abrafi_backed_token_mint,
    )]
    pub escrow_token_account: Account<'info, TokenAccount>,

    /// User's unmint details account (will be created if it doesn't exist)
    #[account(
        init_if_needed,
        payer = user,
        seeds = [UNMINT_DETAILS_SEED, user.key().as_ref(), claim_token_mint.key().as_ref()],
        space = 8 + UserUnmintDetails::INIT_SPACE,
        bump
    )]
    pub user_unmint_details: Account<'info, UserUnmintDetails>,

    /// Mint whitelist entry account (required if whitelist is enabled)
    /// CHECK: Validated in handler if whitelist is enabled
    pub mint_whitelist: UncheckedAccount<'info>,

    /// Associated token program for account creation
    pub associated_token_program: Program<'info, AssociatedToken>,

    /// Token program for transfers and minting
    pub token_program: Program<'info, Token>,

    /// System program for account creation
    pub system_program: Program<'info, System>,
}

/// Request to unmint abrafi tokens back to a collateral token
pub fn request_unmint_handler(ctx: Context<RequestUnmint>, amount: u64) -> Result<()> {
    let state = &ctx.accounts.state;
    let user_unmint_details = &mut ctx.accounts.user_unmint_details;

    let clock = Clock::get()?;

    // Check if unminting request is enabled
    require!(state.is_unminting_request_enabled, ErrorCode::UnmintingDisabled);

    // Check whitelist if enabled
    verify_mint_whitelist(
        ctx.program_id,
        &ctx.accounts.user.key(),
        &ctx.accounts.mint_whitelist.to_account_info(),
        state.is_mint_whitelist_enabled,
    )?;

    // Resolve the cooldown for this wallet: custom (if active) or global default.
    // Must be called after verify_mint_whitelist so the account is already validated.
    let effective_cooldown = resolve_unmint_cooldown(
        &ctx.accounts.mint_whitelist.to_account_info(),
        state.is_mint_whitelist_enabled,
        state.unmint_cooldown_seconds,
        clock.unix_timestamp,
    )?;

    // Validate that the claim token is configured and enabled
    // We use find_enabled_token_config to ensure the token exists and is not disabled
    find_enabled_token_config(state, &ctx.accounts.claim_token_mint.key())
        .ok_or(ErrorCode::TokenNotConfigured)?;

    // Check if there's an existing request and determine if it's active or expired
    let has_existing_request = ctx.accounts.escrow_token_account.amount > 0;
    let has_active_request = has_existing_request
        && clock.unix_timestamp < user_unmint_details.request_expiration_timestamp;
    let has_expired_request = has_existing_request && !has_active_request;

    // Handle expired requests first - return all escrow funds and clear the request
    if has_expired_request {
        // Transfer all escrow funds back to user
        let escrow_amount = ctx.accounts.escrow_token_account.amount;
        if escrow_amount > 0 {
            token::transfer_checked(
                CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    TransferChecked {
                        from: ctx.accounts.escrow_token_account.to_account_info(),
                        mint: ctx.accounts.abrafi_backed_token_mint.to_account_info(),
                        to: ctx.accounts.user_abrafi_backed_token_account.to_account_info(),
                        authority: user_unmint_details.to_account_info(),
                    },
                    &[&[
                        UNMINT_DETAILS_SEED,
                        ctx.accounts.user.key().as_ref(),
                        ctx.accounts.claim_token_mint.key().as_ref(),
                        &[user_unmint_details.bump],
                    ]],
                ),
                escrow_amount,
                ctx.accounts.abrafi_backed_token_mint.decimals,
            )?;
        }

        // Clear the remaining expired request details
        user_unmint_details.claim_token_mint = Pubkey::default();
        user_unmint_details.request_timestamp = 0;
        user_unmint_details.withdrawal_delay_end_timestamp = 0;
        user_unmint_details.request_expiration_timestamp = 0;

        // Emit event for expired request cancellation
        emit!(UnmintCancelled {
            version: 1,
            cancelled_amount: escrow_amount,
            user: ctx.accounts.user.key(),
        });

        // Reload the user's token account to get the updated balance after the transfer
        ctx.accounts.user_abrafi_backed_token_account.reload()?;
    }

    // Check for zero amount and user's abrafi token balance (now includes any returned escrow funds)
    validate_sufficient_balance(
        amount,
        ctx.accounts.user_abrafi_backed_token_account.amount,
        ErrorCode::InvalidAmount,
    )?;

    // Validate amount against minimum unmint amount
    validate_amount_meets_minimum(amount, state.minimum_unmint_amount, ErrorCode::AmountBelowMinimum)?;

    if has_active_request {
        // Transfer additional abrafi tokens to escrow to merge the requests
        token::transfer_checked(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                TransferChecked {
                    from: ctx.accounts.user_abrafi_backed_token_account.to_account_info(),
                    mint: ctx.accounts.abrafi_backed_token_mint.to_account_info(),
                    to: ctx.accounts.escrow_token_account.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            amount,
            ctx.accounts.abrafi_backed_token_mint.decimals,
        )?;

        // Update request timestamp to current time (restart cooldown)
        user_unmint_details.request_timestamp = clock.unix_timestamp;
    } else {
        // Handle new request
        token::transfer_checked(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                TransferChecked {
                    from: ctx.accounts.user_abrafi_backed_token_account.to_account_info(),
                    mint: ctx.accounts.abrafi_backed_token_mint.to_account_info(),
                    to: ctx.accounts.escrow_token_account.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            amount,
            ctx.accounts.abrafi_backed_token_mint.decimals,
        )?;

        // Update request details
        user_unmint_details.version = 1;
        user_unmint_details.claim_token_mint = ctx.accounts.claim_token_mint.key();
        user_unmint_details.request_timestamp = clock.unix_timestamp;
        // withdrawal_delay_end_timestamp and request_expiration_timestamp are set below
        user_unmint_details.bump = ctx.bumps.user_unmint_details;
    }

    // Compute withdrawal delay and request expiration using the resolved cooldown.
    // effective_cooldown is the wallet's custom value (if active) or the global default.
    user_unmint_details.withdrawal_delay_end_timestamp = safe_add_delay(
        user_unmint_details.request_timestamp,
        effective_cooldown,
        ErrorCode::CalculationOverflow,
    )?;

    user_unmint_details.request_expiration_timestamp = safe_add_delay(
        user_unmint_details.withdrawal_delay_end_timestamp,
        state.request_expiration_seconds,
        ErrorCode::CalculationOverflow,
    )?;

    // Reload escrow account to get updated balance after transfer
    ctx.accounts.escrow_token_account.reload()?;

    // Reload user account to get updated balance after transfer
    ctx.accounts.user_abrafi_backed_token_account.reload()?;

    // Ensure user's account balance after transfer is either zero or >= minimum
    // (We don't need to check escrow since we're adding at least the minimum amount to it)
    let user_balance_after = ctx.accounts.user_abrafi_backed_token_account.amount;
    validate_balance_zero_or_above_minimum(
        user_balance_after,
        state.minimum_unmint_amount,
        ErrorCode::BalanceBelowMinimum,
    )?;

    emit!(UnmintRequested {
        version: 1,
        user: ctx.accounts.user.key(),
        requested_amount: ctx.accounts.escrow_token_account.amount,
        request_timestamp: user_unmint_details.request_timestamp,
        withdrawal_delay_end_timestamp: user_unmint_details.withdrawal_delay_end_timestamp,
        claim_token_mint: ctx.accounts.claim_token_mint.key(),
        escrow_account: ctx.accounts.escrow_token_account.key(),
        request_expiration_timestamp: user_unmint_details.request_expiration_timestamp,
    });

    Ok(())
}
