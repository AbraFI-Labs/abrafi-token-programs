use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, TransferChecked};

use crate::constants::*;
use crate::error::ErrorCode;
use crate::events::YieldPosted;
use crate::state::ProgramState;

#[derive(Accounts)]
pub struct PostYield<'info> {
    #[account(
        mut,
        seeds = [STATE_SEED],
        bump = state.bump,
        has_one = operations_authority,
        has_one = staking_vault,
        has_one = stake_mint,
    )]
    pub state: Account<'info, ProgramState>,

    pub operations_authority: Signer<'info>,

    #[account(
        mut,
        associated_token::mint = stake_mint,
        associated_token::authority = state,
    )]
    pub staking_vault: Account<'info, TokenAccount>,

    pub stake_mint: Account<'info, Mint>,

    #[account(
        mut,
        token::mint = stake_mint,
        token::authority = operations_authority,
    )]
    pub authority_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

pub fn post_yield_handler(ctx: Context<PostYield>, amount: u64) -> Result<()> {
    require!(amount > 0, ErrorCode::InvalidAmount);
    require!(ctx.accounts.state.total_staked > 0, ErrorCode::NoStakersToReceiveYield);

    token::transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.authority_token_account.to_account_info(),
                mint: ctx.accounts.stake_mint.to_account_info(),
                to: ctx.accounts.staking_vault.to_account_info(),
                authority: ctx.accounts.operations_authority.to_account_info(),
            },
        ),
        amount,
        ctx.accounts.stake_mint.decimals,
    )?;
    ctx.accounts.staking_vault.reload()?;

    let vault_balance = ctx.accounts.staking_vault.amount;
    let already_accounted = ctx
        .accounts
        .state
        .total_staked
        .checked_add(ctx.accounts.state.total_yield_allocated)
        .ok_or(ErrorCode::CalculationOverflow)?
        .checked_add(ctx.accounts.state.total_pending_unstake)
        .ok_or(ErrorCode::CalculationOverflow)?;

    let effective_amount = vault_balance.saturating_sub(already_accounted);
    require!(effective_amount > 0, ErrorCode::InvalidAmount);

    let index_increase = (effective_amount as u128)
        .checked_mul(PRECISION_FACTOR)
        .ok_or(ErrorCode::CalculationOverflow)?
        .checked_div(ctx.accounts.state.total_staked as u128)
        .ok_or(ErrorCode::CalculationOverflow)?;

    ctx.accounts.state.global_reward_index = ctx
        .accounts
        .state
        .global_reward_index
        .checked_add(index_increase)
        .ok_or(ErrorCode::CalculationOverflow)?;

    // Back-compute the amount that is actually claimable in aggregate. Integer
    // division in the index calculation means effective_amount may exceed the
    // sum of what all users can claim; the dust stays in the vault and is swept
    // into the next post_yield call via the vault-balance delta.
    let distributable_amount = u64::try_from(
        index_increase
            .checked_mul(ctx.accounts.state.total_staked as u128)
            .ok_or(ErrorCode::CalculationOverflow)?
            .checked_div(PRECISION_FACTOR)
            .ok_or(ErrorCode::CalculationOverflow)?,
    )
    .map_err(|_| ErrorCode::CalculationOverflow)?;

    ctx.accounts.state.total_yield_allocated = ctx
        .accounts
        .state
        .total_yield_allocated
        .checked_add(distributable_amount)
        .ok_or(ErrorCode::CalculationOverflow)?;

    emit!(YieldPosted {
        version: 1,
        reward_mint: ctx.accounts.stake_mint.key(),
        effective_amount: distributable_amount,
        new_global_reward_index: ctx.accounts.state.global_reward_index,
    });

    Ok(())
}
