use anchor_lang::prelude::*;
use anchor_spl::token::Mint;

use crate::constants::*;
use crate::error::ErrorCode;
use crate::events::YieldCompounded;
use crate::state::{ProgramState, UserStakeAccount};
use crate::utils::update_pending_rewards;

#[derive(Accounts)]
pub struct ClaimYield<'info> {
    #[account(
        mut,
        seeds = [STATE_SEED],
        bump = state.bump,
        has_one = stake_mint,
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

    pub stake_mint: Account<'info, Mint>,
}

pub fn claim_yield_handler(ctx: Context<ClaimYield>) -> Result<()> {
    require!(
        ctx.accounts.state.claim_yield_enabled,
        ErrorCode::ClaimYieldDisabled
    );

    let compounded = update_pending_rewards(
        &mut ctx.accounts.state,
        &mut ctx.accounts.user_stake,
    )?;

    require!(compounded > 0, ErrorCode::NoPendingRewards);

    emit!(YieldCompounded {
        version: 1,
        user: ctx.accounts.user.key(),
        compounded_amount: compounded,
        new_staked_amount: ctx.accounts.user_stake.staked_amount,
    });

    Ok(())
}
