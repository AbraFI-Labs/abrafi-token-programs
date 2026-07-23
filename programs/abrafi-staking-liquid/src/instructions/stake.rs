use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, Token, TokenAccount, TransferChecked},
    token_interface::{self, MintToChecked},
};

use crate::constants::*;
use crate::error::ErrorCode;
use crate::events::UnderlyingStaked;
use crate::state::*;
use crate::utils::*;

/// Stake underlying tokens to receive liquid staking tokens
#[derive(Accounts)]
pub struct Stake<'info> {
    /// Liquid staking state account
    #[account(
        mut,
        seeds = [STATE_SEED],
        bump = state.state_bump,
        has_one = liquid_staking_token_mint,
        has_one = underlying_token_mint,
    )]
    pub state: Account<'info, ProgramState>,

    /// User who wants to stake underlying tokens
    #[account(mut)]
    pub user: Signer<'info>,

    /// The liquid staking token mint
    #[account(mut)]
    pub liquid_staking_token_mint: Account<'info, Mint>,

    /// The underlying token mint
    pub underlying_token_mint: Account<'info, Mint>,

    /// User's liquid staking token account (will be created if it doesn't exist)
    #[account(
        init_if_needed,
        payer = user,
        associated_token::authority = user,
        associated_token::mint = liquid_staking_token_mint,
        constraint = !user_liquid_staking_token_account.is_frozen() @ ErrorCode::AccountFrozen,
    )]
    pub user_liquid_staking_token_account: Account<'info, TokenAccount>,

    /// User's underlying token account
    #[account(
        mut,
        associated_token::authority = user,
        associated_token::mint = underlying_token_mint,
        constraint = !user_underlying_token_account.is_frozen() @ ErrorCode::AccountFrozen,
    )]
    pub user_underlying_token_account: Account<'info, TokenAccount>,

    /// Vault account for storing underlying tokens
    #[account(
        mut,
        associated_token::authority = state,
        associated_token::mint = underlying_token_mint,
    )]
    pub vault_token_account: Account<'info, TokenAccount>,

    /// Associated token program for account creation
    pub associated_token_program: Program<'info, AssociatedToken>,

    /// Token program for transfers and minting
    pub token_program: Program<'info, Token>,

    /// System program for account creation
    pub system_program: Program<'info, System>,
}

/// Stake underlying tokens to receive liquid staking tokens
/// The amount of liquid staking tokens received is calculated based on the current exchange rate
pub fn stake_handler(ctx: Context<Stake>, underlying_amount: u64) -> Result<()> {
    let state = &mut ctx.accounts.state;

    // Check if staking is enabled
    require!(state.is_staking_enabled, ErrorCode::StakingDisabled);

    // Check for zero amount and user's underlying token balance
    validate_sufficient_balance(
        underlying_amount,
        ctx.accounts.user_underlying_token_account.amount,
        ErrorCode::InvalidAmount,
    )?;

    // Check if amount meets minimum requirement
    validate_amount_meets_minimum(underlying_amount, state.minimum_stake_amount, ErrorCode::AmountBelowMinimum)?;

    // Calculate how many liquid staking tokens to mint using conversion rate calculation
    let liquid_staking_amount = convert_to_shares(
        underlying_amount,
        ctx.accounts.vault_token_account.amount,
        ctx.accounts.liquid_staking_token_mint.supply,
    )?;

    // Transfer underlying tokens from user to vault
    token::transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.user_underlying_token_account.to_account_info(),
                mint: ctx.accounts.underlying_token_mint.to_account_info(),
                to: ctx.accounts.vault_token_account.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        ),
        underlying_amount,
        ctx.accounts.underlying_token_mint.decimals,
    )?;

    // Mint liquid staking tokens to user
    token_interface::mint_to_checked(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintToChecked {
                mint: ctx.accounts.liquid_staking_token_mint.to_account_info(),
                to: ctx.accounts.user_liquid_staking_token_account.to_account_info(),
                authority: state.to_account_info(),
            },
            &[&[STATE_SEED, &[state.state_bump]]],
        ),
        liquid_staking_amount,
        ctx.accounts.liquid_staking_token_mint.decimals,
    )?;

    emit!(UnderlyingStaked {
        version: 1,
        user: ctx.accounts.user.key(),
        underlying_amount,
        liquid_staking_minted: liquid_staking_amount,
    });

    Ok(())
}
