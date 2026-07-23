use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, Token, TokenAccount, TransferChecked},
    token_interface::{self, BurnChecked},
};

use crate::constants::*;
use crate::error::ErrorCode;
use crate::events::*;
use crate::state::*;
use crate::utils::*;

/// Claim unmint instruction accounts
#[derive(Accounts)]
pub struct ClaimUnmint<'info> {
    /// Program state account
    #[account(
        mut,
        seeds = [STATE_SEED],
        bump = state.state_bump,
        has_one = abrafi_backed_token_mint,
    )]
    pub state: Account<'info, ProgramState>,

    /// User who requested the unmint
    #[account(mut)]
    pub user: Signer<'info>,

    /// Vault authority PDA that controls all withdrawal vaults
    /// CHECK: Safe because we derive it with PDA and just store it
    #[account(
        seeds = [VAULT_AUTHORITY_SEED, state.key().as_ref()],
        bump = state.vault_authority_bump,
    )]
    pub vault_authority: UncheckedAccount<'info>,

    /// The abrafi token mint
    #[account(mut)]
    pub abrafi_backed_token_mint: Account<'info, Mint>,

    /// Claim token mint
    /// This is the token mint that the user wants to claim
    #[account(mut)]
    pub claim_token_mint: Account<'info, Mint>,

    /// User's abrafi token account
    #[account(
        mut,
        associated_token::authority = user,
        associated_token::mint = abrafi_backed_token_mint,
        constraint = !user_abrafi_backed_token_account.is_frozen() @ ErrorCode::AccountFrozen,
    )]
    pub user_abrafi_backed_token_account: Account<'info, TokenAccount>,

    /// User's claimed token account (will be created if it doesn't exist)
    #[account(
        init_if_needed,
        payer = user,
        associated_token::authority = user,
        associated_token::mint = claim_token_mint,
        constraint = !user_claim_token_account.is_frozen() @ ErrorCode::AccountFrozen,
    )]
    pub user_claim_token_account: Account<'info, TokenAccount>,

    /// Escrow token account for holding abrafi tokens during unmint process
    #[account(
        mut,
        associated_token::authority = user_unmint_details,
        associated_token::mint = abrafi_backed_token_mint,
    )]
    pub escrow_token_account: Account<'info, TokenAccount>,

    /// Withdrawal vault account for the claimed token
    #[account(
        mut,
        associated_token::mint = claim_token_mint,
        associated_token::authority = vault_authority,
    )]
    pub vault_token_account: Account<'info, TokenAccount>,

    /// User's unmint details account
    #[account(
        mut,
        seeds = [UNMINT_DETAILS_SEED, user.key().as_ref(), claim_token_mint.key().as_ref()],
        bump = user_unmint_details.bump,
        has_one = claim_token_mint
    )]
    pub user_unmint_details: Account<'info, UserUnmintDetails>,

    /// Mint whitelist entry account (required if whitelist is enabled)
    /// CHECK: Validated in handler if whitelist is enabled
    pub mint_whitelist: UncheckedAccount<'info>,

    /// Associated token program for account creation
    pub associated_token_program: Program<'info, AssociatedToken>,

    /// Token program for transfers and burning
    pub token_program: Program<'info, Token>,

    /// System program for account creation
    pub system_program: Program<'info, System>,
}

/// Claim unminted tokens after the cooldown period has completed
/// This burns the abrafi tokens and transfers the claimed tokens to the user
/// Users can claim partial amounts, reducing the requested amount accordingly
pub fn claim_unmint_handler(ctx: Context<ClaimUnmint>, claim_amount: u64) -> Result<()> {
    let state = &ctx.accounts.state;
    let user = &ctx.accounts.user;
    let user_unmint_details = &mut ctx.accounts.user_unmint_details;

    let clock = Clock::get()?;

    // Check if unminting claim is enabled
    require!(state.is_unminting_claim_enabled, ErrorCode::UnmintingDisabled);

    // Check whitelist if enabled
    verify_mint_whitelist(
        ctx.program_id,
        &ctx.accounts.user.key(),
        &ctx.accounts.mint_whitelist.to_account_info(),
        state.is_mint_whitelist_enabled,
    )?;

    let escrow_balance_before = ctx.accounts.escrow_token_account.amount;

    // Check for zero amount and escrow balance
    validate_sufficient_balance(claim_amount, escrow_balance_before, ErrorCode::InvalidAmount)?;

    // Ensure claim amount is either the entire escrow balance or >= minimum unmint amount
    validate_amount_full_or_above_minimum(
        claim_amount,
        escrow_balance_before,
        state.minimum_unmint_amount,
        ErrorCode::AmountBelowMinimum,
    )?;

    // Check if the withdrawal delay has passed
    validate_timestamp_has_passed(
        clock.unix_timestamp,
        user_unmint_details.withdrawal_delay_end_timestamp,
        ErrorCode::CooldownNotExpired,
    )?;

    // Check if request has expired
    validate_timestamp_has_not_passed(
        clock.unix_timestamp,
        user_unmint_details.request_expiration_timestamp,
        ErrorCode::RequestExpired,
    )?;

    // Burn abrafi tokens from escrow account using user_unmint_details PDA as authority
    token_interface::burn_checked(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            BurnChecked {
                mint: ctx.accounts.abrafi_backed_token_mint.to_account_info(),
                from: ctx.accounts.escrow_token_account.to_account_info(),
                authority: user_unmint_details.to_account_info(),
            },
            &[&[
                UNMINT_DETAILS_SEED,
                user.key().as_ref(),
                ctx.accounts.claim_token_mint.key().as_ref(),
                &[user_unmint_details.bump],
            ]],
        ),
        claim_amount,
        ctx.accounts.abrafi_backed_token_mint.decimals,
    )?;

    // Convert abrafi token amount to collateral amount accounting for decimal differences
    let collateral_amount = scale_amount_to_new_decimals(
        claim_amount,
        ctx.accounts.abrafi_backed_token_mint.decimals,
        ctx.accounts.claim_token_mint.decimals,
        ErrorCode::CalculationOverflow,
    )?;

    // Check if withdrawal vault has enough tokens to transfer
    validate_sufficient_balance(
        collateral_amount,
        ctx.accounts.vault_token_account.amount,
        ErrorCode::InsufficientVaultBalance,
    )?;

    // Transfer claimed tokens from withdrawal vault to user using vault authority PDA signing
    token::transfer_checked(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.vault_token_account.to_account_info(),
                mint: ctx.accounts.claim_token_mint.to_account_info(),
                to: ctx.accounts.user_claim_token_account.to_account_info(),
                authority: ctx.accounts.vault_authority.to_account_info(),
            },
            &[&[
                VAULT_AUTHORITY_SEED,
                state.key().as_ref(),
                &[state.vault_authority_bump],
            ]],
        ),
        collateral_amount,
        ctx.accounts.claim_token_mint.decimals,
    )?;

    // Reload escrow account to get updated balance after burn
    ctx.accounts.escrow_token_account.reload()?;

    // Ensure escrow balance is either zero or >= minimum
    let escrow_balance_after = ctx.accounts.escrow_token_account.amount;
    validate_balance_zero_or_above_minimum(
        escrow_balance_after,
        state.minimum_unmint_amount,
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

    emit!(UnmintClaimed {
        version: 1,
        user: user.key(),
        claimed_amount: claim_amount,
        collateral_amount: collateral_amount,
        claim_token_mint: ctx.accounts.claim_token_mint.key(),
        user_claim_token_account: ctx.accounts.user_claim_token_account.key(),
    });

    Ok(())
}
