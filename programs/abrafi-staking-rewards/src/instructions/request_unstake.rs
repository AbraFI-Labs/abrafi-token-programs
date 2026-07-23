use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};

use crate::constants::*;
use crate::error::ErrorCode;
use crate::events::UnstakeRequested;
use crate::state::{ProgramState, UnstakeRequest, UserStakeAccount};
use crate::utils::{add_delay, update_pending_rewards};

#[derive(Accounts)]
pub struct RequestUnstake<'info> {
    #[account(
        mut,
        seeds = [STATE_SEED],
        bump = state.bump,
        has_one = stake_mint,
        has_one = staking_vault,
    )]
    pub state: Account<'info, ProgramState>,

    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [USER_STAKE_SEED, user.key().as_ref()],
        bump = user_stake.bump,
        has_one = user,
    )]
    pub user_stake: Account<'info, UserStakeAccount>,

    #[account(
        init,
        payer = user,
        seeds = [UNSTAKE_REQUEST_SEED, user.key().as_ref()],
        space = 8 + UnstakeRequest::INIT_SPACE,
        bump
    )]
    pub unstake_request: Account<'info, UnstakeRequest>,

    pub stake_mint: Account<'info, Mint>,

    #[account(
        associated_token::mint = stake_mint,
        associated_token::authority = state,
    )]
    pub staking_vault: Account<'info, TokenAccount>,

    /// User ATA is created here if it does not yet exist so that claim_unstake
    /// is guaranteed to have a destination. Frozen accounts are rejected to
    /// prevent initiating a withdrawal to an inaccessible address.
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = stake_mint,
        associated_token::authority = user,
        constraint = !user_token_account.is_frozen() @ ErrorCode::AccountFrozen,
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn request_unstake_handler(ctx: Context<RequestUnstake>, amount: u64) -> Result<()> {
    require!(
        ctx.accounts.state.unstake_request_enabled,
        ErrorCode::UnstakeRequestDisabled
    );
    require!(amount > 0, ErrorCode::InvalidAmount);

    let compounded = update_pending_rewards(
        &mut ctx.accounts.state,
        &mut ctx.accounts.user_stake,
    )?;

    require!(
        ctx.accounts.user_stake.staked_amount >= amount,
        ErrorCode::InsufficientStakedBalance
    );

    ctx.accounts.user_stake.staked_amount = ctx
        .accounts
        .user_stake
        .staked_amount
        .checked_sub(amount)
        .ok_or(ErrorCode::CalculationOverflow)?;
    ctx.accounts.state.total_staked = ctx
        .accounts
        .state
        .total_staked
        .checked_sub(amount)
        .ok_or(ErrorCode::CalculationOverflow)?;
    ctx.accounts.state.total_pending_unstake = ctx
        .accounts
        .state
        .total_pending_unstake
        .checked_add(amount)
        .ok_or(ErrorCode::CalculationOverflow)?;

    let now = Clock::get()?.unix_timestamp;
    let claim_after_ts =
        add_delay(now, ctx.accounts.state.withdrawal_delay, ErrorCode::CalculationOverflow)?;

    let request = &mut ctx.accounts.unstake_request;
    request.version = 1;
    request.bump = ctx.bumps.unstake_request;
    request.user = ctx.accounts.user.key();
    request.amount = amount;
    request.claim_after_ts = claim_after_ts;

    emit!(UnstakeRequested {
        version: 1,
        user: ctx.accounts.user.key(),
        amount,
        claim_after_ts,
        compounded_rewards: compounded,
    });

    Ok(())
}
