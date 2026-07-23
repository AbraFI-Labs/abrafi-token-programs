use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, Token, TokenAccount, TransferChecked},
    token_interface::{self, MintToChecked},
};

use crate::constants::*;
use crate::error::ErrorCode;
use crate::events::UnderlyingRestaked;
use crate::state::*;
use crate::utils::*;

/// Restake instruction accounts - convert unstake request back to staked liquid staking tokens
#[derive(Accounts)]
pub struct Restake<'info> {
    /// Liquid staking state account
    #[account(
        mut,
        seeds = [STATE_SEED],
        bump = state.state_bump,
        has_one = liquid_staking_token_mint,
        has_one = underlying_token_mint,
    )]
    pub state: Account<'info, ProgramState>,

    /// User who wants to restake their unstake request
    #[account(mut)]
    pub user: Signer<'info>,

    /// Liquid staking token mint
    #[account(mut)]
    pub liquid_staking_token_mint: Account<'info, Mint>,

    /// User's liquid staking token account (destination for restaked tokens)
    #[account(
        mut,
        associated_token::authority = user,
        associated_token::mint = state.liquid_staking_token_mint,
        constraint = !user_liquid_staking_token_account.is_frozen() @ ErrorCode::AccountFrozen,
    )]
    pub user_liquid_staking_token_account: Account<'info, TokenAccount>,

    /// Underlying token mint
    pub underlying_token_mint: Account<'info, Mint>,

    /// Vault account for storing underlying tokens
    #[account(
        mut,
        associated_token::authority = state,
        associated_token::mint = state.underlying_token_mint,
    )]
    pub vault_token_account: Account<'info, TokenAccount>,

    /// Escrow token account for holding underlying tokens during unstake process
    #[account(
        mut,
        associated_token::authority = user_unstake_request,
        associated_token::mint = state.underlying_token_mint,
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

    /// Token program for minting and transfers
    pub token_program: Program<'info, Token>,
}

/// Restake unstake request - convert escrowed underlying tokens back to liquid staking tokens
pub fn restake_handler(ctx: Context<Restake>, restake_underlying_amount: u64) -> Result<()> {
    let state = &ctx.accounts.state;
    let user_unstake_request = &mut ctx.accounts.user_unstake_request;

    // Check if staking is enabled
    require!(state.is_staking_enabled, ErrorCode::StakingDisabled);

    let escrow_balance = ctx.accounts.escrow_token_account.amount;

    // Check if escrow has any amount to restake
    require!(escrow_balance > 0, ErrorCode::NoUnstakeRequest);

    // Check for zero amount and escrow balance
    validate_sufficient_balance(restake_underlying_amount, escrow_balance, ErrorCode::InvalidAmount)?;

    // Check if amount is either the entire escrow balance or at least the minimum stake amount
    validate_amount_full_or_above_minimum(
        restake_underlying_amount,
        escrow_balance,
        state.minimum_stake_amount,
        ErrorCode::AmountBelowMinimum,
    )?;

    // Calculate how many liquid staking tokens to mint using current conversion rate
    let liquid_staking_amount = convert_to_shares(
        restake_underlying_amount,
        ctx.accounts.vault_token_account.amount,
        ctx.accounts.liquid_staking_token_mint.supply,
    )?;

    // Transfer underlying tokens from escrow to vault using UserUnstakeRequest PDA as authority
    token::transfer_checked(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.escrow_token_account.to_account_info(),
                mint: ctx.accounts.underlying_token_mint.to_account_info(),
                to: ctx.accounts.vault_token_account.to_account_info(),
                authority: user_unstake_request.to_account_info(),
            },
            &[&[
                USER_UNSTAKE_REQUEST_SEED,
                ctx.accounts.user.key().as_ref(),
                &[user_unstake_request.bump],
            ]],
        ),
        restake_underlying_amount,
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

    // Reload escrow account to get updated balance after transfer
    ctx.accounts.escrow_token_account.reload()?;

    // Close the accounts if the escrow is empty
    if ctx.accounts.escrow_token_account.amount == 0 {
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

    emit!(UnderlyingRestaked {
        version: 1,
        user: ctx.accounts.user.key(),
        underlying_amount: restake_underlying_amount,
        liquid_staking_minted: liquid_staking_amount,
    });

    Ok(())
}
