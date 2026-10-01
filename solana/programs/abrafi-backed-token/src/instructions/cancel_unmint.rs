// SPDX-License-Identifier: BUSL-1.1
// Copyright (C) 2026 AbraFi Ltd., Business Source License 1.1.
// Change Date: 2nd anniversary of this version's publication.
// Change License: Apache-2.0. See LICENSE at repo root.

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

    pub system_program: Program<'info, System>,
}

/// Cancel an unmint request (partial or full)
/// This instruction can be called at any time to immediately cancel part or all of the request
/// It reduces the requested amount and closes the account if fully canceled
pub fn cancel_unmint_handler(ctx: Context<CancelUnmint>, cancel_amount: u64) -> Result<()> {
    let state = &mut ctx.accounts.state;
    let user = &ctx.accounts.user;
    let user_unmint_details = &mut ctx.accounts.user_unmint_details;

    require!(state.is_unminting_claim_enabled, ErrorCode::UnmintingDisabled);

    let recorded_amount = user_unmint_details.requested_amount;

    // External parties can only deposit into this PDA-owned escrow, never withdraw.
    // This program always decrements requested_amount by the same amount it withdraws,
    // so escrow_balance >= requested_amount must always hold.
    validate_sufficient_balance(recorded_amount, ctx.accounts.escrow_token_account.amount, ErrorCode::InvalidAmount)?;

    // Validate against the recorded requested_amount, not the live escrow balance.
    // The escrow ATA is an SPL token account that anyone can deposit into; using the live
    // balance would allow a third party to inflate the effective "full" amount and block
    // partial cancels that fall below the minimum.
    validate_sufficient_balance(cancel_amount, recorded_amount, ErrorCode::InvalidAmount)?;

    // Ensure cancel amount is either the entire recorded amount or >= minimum mint amount
    // (We use minimum_mint_amount because tokens are going back to user's account)
    validate_amount_full_or_above_minimum(
        cancel_amount,
        recorded_amount,
        state.minimum_mint_amount,
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

    user_unmint_details.requested_amount = user_unmint_details.requested_amount
        .checked_sub(cancel_amount)
        .ok_or(Error::from(ErrorCode::CalculationOverflow))?;

    // Reload escrow account to get updated balance after transfer
    ctx.accounts.escrow_token_account.reload()?;
    let escrow_balance_after = ctx.accounts.escrow_token_account.amount;

    if user_unmint_details.requested_amount == 0 {
        // Full cancel completed. The escrow ATA may still hold tokens deposited externally.
        // Return any remaining balance to the user before closing so the accounts are
        // always cleaned up on a full cancel.
        if escrow_balance_after > 0 {
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
                escrow_balance_after,
                ctx.accounts.abrafi_backed_token_mint.decimals,
            )?;
        }
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
    } else {
        // Partial cancel: remaining recorded amount must be zero or above minimum.
        // Escrow balance is ignored here — external deposits are swept back on full exit
        // and do not represent the user's legitimate remaining position.
        validate_balance_zero_or_above_minimum(
            user_unmint_details.requested_amount,
            state.minimum_unmint_amount,
            ErrorCode::BalanceBelowMinimum,
        )?;
    }

    emit!(UnmintCancelled {
        version: 1,
        cancelled_amount: cancel_amount,
        user: user.key(),
    });

    Ok(())
}
