use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, TransferChecked};

use crate::constants::*;
use crate::error::ErrorCode;
use crate::events::*;
use crate::state::*;
use crate::utils::*;

/// Cancel unmint instruction accounts
#[derive(Accounts)]
pub struct CancelUnmint<'info> {
    /// Program state account
    #[account(
        mut,
        seeds = [STATE_SEED],
        bump = state.state_bump,
        has_one = abrafi_backed_token_mint,
    )]
    pub state: Account<'info, ProgramState>,

    /// User who wants to cancel their unmint request
    #[account(mut)]
    pub user: Signer<'info>,

    /// The abrafi token mint
    pub abrafi_backed_token_mint: Account<'info, Mint>,

    /// Claim token mint (the collateral token the user planned to receive)
    pub claim_token_mint: Account<'info, Mint>,

    /// User's abrafi token account (to receive returned tokens from escrow)
    #[account(
        mut,
        associated_token::authority = user,
        associated_token::mint = abrafi_backed_token_mint,
        constraint = !user_abrafi_backed_token_account.is_frozen() @ ErrorCode::AccountFrozen,
    )]
    pub user_abrafi_backed_token_account: Account<'info, TokenAccount>,

    /// Escrow token account for holding abrafi tokens during unmint process
    #[account(
        mut,
        associated_token::authority = user_unmint_details,
        associated_token::mint = abrafi_backed_token_mint,
    )]
    pub escrow_token_account: Account<'info, TokenAccount>,

    /// User's unmint details account to cancel
    #[account(
        mut,
        seeds = [UNMINT_DETAILS_SEED, user.key().as_ref(), claim_token_mint.key().as_ref()],
        bump = user_unmint_details.bump,
        has_one = claim_token_mint,
    )]
    pub user_unmint_details: Account<'info, UserUnmintDetails>,

    /// Token program for transfers
    pub token_program: Program<'info, Token>,
}

/// Cancel an unmint request (partial or full)
/// This instruction can be called at any time to immediately cancel part or all of the request
/// It reduces the requested amount and closes the account if fully canceled
pub fn cancel_unmint_handler(ctx: Context<CancelUnmint>, cancel_amount: u64) -> Result<()> {
    let state = &ctx.accounts.state;
    let user = &ctx.accounts.user;
    let user_unmint_details = &mut ctx.accounts.user_unmint_details;

    // Check if unminting claim is enabled (canceling recovers tokens, similar to claiming)
    require!(state.is_unminting_claim_enabled, ErrorCode::UnmintingDisabled);

    let escrow_balance_before = ctx.accounts.escrow_token_account.amount;

    // Check for zero amount and escrow balance
    validate_sufficient_balance(cancel_amount, escrow_balance_before, ErrorCode::InvalidAmount)?;

    // Ensure cancel amount is either the entire escrow balance or >= minimum mint amount
    // (We use minimum_mint_amount because tokens are going back to user's account)
    validate_amount_full_or_above_minimum(
        cancel_amount,
        escrow_balance_before,
        ctx.accounts.state.minimum_mint_amount,
        ErrorCode::AmountBelowMinimum,
    )?;

    // Transfer abrafi tokens from escrow back to user
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
                user.key().as_ref(),
                ctx.accounts.claim_token_mint.key().as_ref(),
                &[user_unmint_details.bump],
            ]],
        ),
        cancel_amount,
        ctx.accounts.abrafi_backed_token_mint.decimals,
    )?;

    // Reload escrow account to get updated balance after transfer
    ctx.accounts.escrow_token_account.reload()?;

    // Ensure remaining escrow balance is either zero or >= minimum unmint amount
    let escrow_balance_after = ctx.accounts.escrow_token_account.amount;
    validate_balance_zero_or_above_minimum(
        escrow_balance_after,
        ctx.accounts.state.minimum_unmint_amount,
        ErrorCode::BalanceBelowMinimum,
    )?;

    // Close the accounts if the escrow is empty
    if escrow_balance_after == 0 {
        close_escrow_token_account(
            ctx.accounts.token_program.to_account_info(),
            &ctx.accounts.escrow_token_account,
            ctx.accounts.user.to_account_info(),
            user_unmint_details.to_account_info(),
            &user.key(),
            &ctx.accounts.claim_token_mint.key(),
            user_unmint_details.bump,
        )?;

        user_unmint_details.close(user.to_account_info())?;
    }

    emit!(UnmintCancelled {
        version: 1,
        cancelled_amount: cancel_amount,
        user: user.key(),
    });

    Ok(())
}
