use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, TransferChecked};

use crate::constants::*;
use crate::error::ErrorCode;
use crate::events::Staked;
use crate::state::{ProgramState, UserStakeAccount};
use crate::utils::update_pending_rewards;

#[derive(Accounts)]
pub struct Stake<'info> {
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
        init_if_needed,
        payer = user,
        seeds = [USER_STAKE_SEED, user.key().as_ref()],
        space = 8 + UserStakeAccount::INIT_SPACE,
        bump
    )]
    pub user_stake: Account<'info, UserStakeAccount>,

    pub stake_mint: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = stake_mint,
        associated_token::authority = state,
    )]
    pub staking_vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = stake_mint,
        associated_token::authority = user,
        constraint = !user_token_account.is_frozen() @ ErrorCode::AccountFrozen,
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn stake_handler(ctx: Context<Stake>, amount: u64) -> Result<()> {
    require!(ctx.accounts.state.stake_enabled, ErrorCode::StakingDisabled);
    require!(amount > 0, ErrorCode::InvalidAmount);

    if ctx.accounts.user_stake.version == 0 {
        ctx.accounts.user_stake.version = 1;
        ctx.accounts.user_stake.bump = ctx.bumps.user_stake;
        ctx.accounts.user_stake.user = ctx.accounts.user.key();
    }

    // Settle and auto-compound pending rewards before modifying staked_amount.
    let compounded = update_pending_rewards(
        &mut ctx.accounts.state,
        &mut ctx.accounts.user_stake,
    )?;

    token::transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.user_token_account.to_account_info(),
                mint: ctx.accounts.stake_mint.to_account_info(),
                to: ctx.accounts.staking_vault.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        ),
        amount,
        ctx.accounts.stake_mint.decimals,
    )?;

    ctx.accounts.user_stake.staked_amount = ctx
        .accounts
        .user_stake
        .staked_amount
        .checked_add(amount)
        .ok_or(ErrorCode::CalculationOverflow)?;
    ctx.accounts.state.total_staked = ctx
        .accounts
        .state
        .total_staked
        .checked_add(amount)
        .ok_or(ErrorCode::CalculationOverflow)?;

    emit!(Staked {
        version: 1,
        user: ctx.accounts.user.key(),
        amount,
        compounded_rewards: compounded,
        new_staked_amount: ctx.accounts.user_stake.staked_amount,
        total_staked: ctx.accounts.state.total_staked,
    });

    Ok(())
}
