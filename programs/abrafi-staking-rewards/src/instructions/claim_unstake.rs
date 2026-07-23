use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, Token, TokenAccount, TransferChecked},
};

use crate::constants::*;
use crate::error::ErrorCode;
use crate::events::UnstakeClaimed;
use crate::state::{ProgramState, UnstakeRequest};

#[derive(Accounts)]
pub struct ClaimUnstake<'info> {
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
        seeds = [UNSTAKE_REQUEST_SEED, user.key().as_ref()],
        bump = unstake_request.bump,
        has_one = user,
        close = user,
    )]
    pub unstake_request: Account<'info, UnstakeRequest>,

    pub stake_mint: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = stake_mint,
        associated_token::authority = state,
    )]
    pub staking_vault: Account<'info, TokenAccount>,

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

pub fn claim_unstake_handler(ctx: Context<ClaimUnstake>) -> Result<()> {
    require!(
        ctx.accounts.state.unstake_claim_enabled,
        ErrorCode::UnstakeClaimDisabled
    );

    let now = Clock::get()?.unix_timestamp;
    require!(
        now >= ctx.accounts.unstake_request.claim_after_ts,
        ErrorCode::WithdrawalDelayNotElapsed
    );

    let amount = ctx.accounts.unstake_request.amount;
    let state_bump = ctx.accounts.state.bump;

    ctx.accounts.state.total_pending_unstake = ctx
        .accounts
        .state
        .total_pending_unstake
        .checked_sub(amount)
        .ok_or(ErrorCode::CalculationOverflow)?;

    token::transfer_checked(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.staking_vault.to_account_info(),
                mint: ctx.accounts.stake_mint.to_account_info(),
                to: ctx.accounts.user_token_account.to_account_info(),
                authority: ctx.accounts.state.to_account_info(),
            },
            &[&[STATE_SEED, &[state_bump]]],
        ),
        amount,
        ctx.accounts.stake_mint.decimals,
    )?;

    emit!(UnstakeClaimed {
        version: 1,
        user: ctx.accounts.user.key(),
        amount,
    });

    Ok(())
}
