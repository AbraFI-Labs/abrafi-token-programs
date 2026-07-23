use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint as TokenMint, Token, TokenAccount, TransferChecked},
    token_interface::{self, MintToChecked},
};

use crate::constants::*;
use crate::error::ErrorCode;
use crate::events::*;
use crate::state::*;
use crate::utils::*;

/// Mint abrafi token instruction accounts
#[derive(Accounts)]
pub struct Mint<'info> {
    /// Program state account
    #[account(
        seeds = [STATE_SEED],
        bump = state.state_bump,
        has_one = abrafi_backed_token_mint,
    )]
    pub state: Account<'info, ProgramState>,

    /// User who wants to mint abrafi tokens
    #[account(mut)]
    pub user: Signer<'info>,

    /// The abrafi token mint
    #[account(mut)]
    pub abrafi_backed_token_mint: Account<'info, TokenMint>,

    /// The collateral token mint
    #[account(mut)]
    pub collateral_token_mint: Account<'info, TokenMint>,

    /// User's abrafi token account (will be created if it doesn't exist)
    #[account(
        init_if_needed,
        payer = user,
        associated_token::authority = user,
        associated_token::mint = abrafi_backed_token_mint,
        constraint = !user_abrafi_backed_token_account.is_frozen() @ ErrorCode::AccountFrozen,
    )]
    pub user_abrafi_backed_token_account: Account<'info, TokenAccount>,

    /// User's collateral token account
    #[account(
        mut,
        associated_token::authority = user,
        associated_token::mint = collateral_token_mint,
        constraint = !user_collateral_token_account.is_frozen() @ ErrorCode::AccountFrozen,
    )]
    pub user_collateral_token_account: Account<'info, TokenAccount>,

    /// Treasury token account for the specific collateral token
    #[account(
        mut,
        constraint = treasury_token_account.mint == collateral_token_mint.key() @ ErrorCode::InvalidMintAccount,
    )]
    pub treasury_token_account: Account<'info, TokenAccount>,

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

/// Mint abrafi tokens using collateral tokens
pub fn mint_handler(ctx: Context<Mint>, amount: u64) -> Result<()> {
    let state = &mut ctx.accounts.state;

    // Ensure minting is enabled
    require!(state.is_minting_enabled, ErrorCode::MintingDisabled);

    // Check whitelist if enabled
    verify_mint_whitelist(
        ctx.program_id,
        &ctx.accounts.user.key(),
        &ctx.accounts.mint_whitelist.to_account_info(),
        state.is_mint_whitelist_enabled,
    )?;

    // Check for zero amount and user's collateral token balance
    validate_sufficient_balance(
        amount,
        ctx.accounts.user_collateral_token_account.amount,
        ErrorCode::InsufficientCollateralBalance,
    )?;

    // Check if the user's collateral token account matches one of the enabled collateral tokens
    let token_config = find_enabled_token_config(state, &ctx.accounts.collateral_token_mint.key())
        .ok_or(ErrorCode::TokenNotConfigured)?;

    let treasury_token_address = token_config.treasury_token_account;

    // Validate treasury account
    require!(
        treasury_token_address == ctx.accounts.treasury_token_account.key(),
        ErrorCode::InvalidTreasuryAccount
    );

    // Convert collateral amount to abrafi token amount accounting for decimal differences
    // This must be done before checking minimum since minimum is in abrafi token terms
    let abrafi_backed_token_amount = scale_amount_to_new_decimals(
        amount,
        ctx.accounts.collateral_token_mint.decimals,
        ctx.accounts.abrafi_backed_token_mint.decimals,
        ErrorCode::CalculationOverflow,
    )?;

    // Validate converted abrafi token amount against minimum mint amount
    // The minimum is stored in abrafi token terms, so we compare the converted amount
    validate_amount_meets_minimum(abrafi_backed_token_amount, state.minimum_mint_amount, ErrorCode::AmountBelowMinimum)?;

    // Transfer collateral tokens from user to treasury
    token::transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.user_collateral_token_account.to_account_info(),
                mint: ctx.accounts.collateral_token_mint.to_account_info(),
                to: ctx.accounts.treasury_token_account.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        ),
        amount,
        ctx.accounts.collateral_token_mint.decimals,
    )?;

    // Mint abrafi tokens to user
    token_interface::mint_to_checked(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintToChecked {
                mint: ctx.accounts.abrafi_backed_token_mint.to_account_info(),
                to: ctx.accounts.user_abrafi_backed_token_account.to_account_info(),
                authority: state.to_account_info(),
            },
            &[&[STATE_SEED, &[state.state_bump]]],
        ),
        abrafi_backed_token_amount,
        ctx.accounts.abrafi_backed_token_mint.decimals,
    )?;

    emit!(TokensMinted {
        version: 1,
        collateral_mint: ctx.accounts.collateral_token_mint.key(),
        user_abrafi_backed_token_account: ctx.accounts.user_abrafi_backed_token_account.key(),
        abrafi_backed_token_amount: abrafi_backed_token_amount,
        collateral_amount: amount,
        user: ctx.accounts.user.key(),
    });

    Ok(())
}
