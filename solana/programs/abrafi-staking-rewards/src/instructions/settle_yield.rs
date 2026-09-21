// SPDX-License-Identifier: BUSL-1.1
// Copyright (C) 2026 AbraFi Ltd., Business Source License 1.1.
// Change Date: 2nd anniversary of this version's publication.
// Change License: Apache-2.0. See LICENSE at repo root.

use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount};

use crate::constants::{PRECISION_FACTOR, STATE_SEED};
use crate::error::ErrorCode;
use crate::events::YieldSettled;
use crate::state::ProgramState;

/// Permissionless accumulator settlement. No authority signature is required.
#[derive(Accounts)]
pub struct SettleYield<'info> {
    #[account(
        mut,
        seeds = [STATE_SEED],
        bump = state.bump,
        has_one = staking_vault,
        has_one = stake_mint,
    )]
    pub state: Account<'info, ProgramState>,

    #[account(
        mut,
        associated_token::mint = stake_mint,
        associated_token::authority = state,
    )]
    pub staking_vault: Account<'info, TokenAccount>,

    pub stake_mint: Account<'info, Mint>,
}

pub fn settle_yield_handler(ctx: Context<SettleYield>) -> Result<()> {
    let state = &mut ctx.accounts.state;

    require!(state.total_staked > 0, ErrorCode::NoStakersToReceiveYield);

    let vault_balance = ctx.accounts.staking_vault.amount;
    let already_accounted = state
        .total_staked
        .checked_add(state.total_yield_allocated)
        .ok_or(ErrorCode::CalculationOverflow)?
        .checked_add(state.total_pending_unstake)
        .ok_or(ErrorCode::CalculationOverflow)?;

    let effective_amount = vault_balance.saturating_sub(already_accounted);
    require!(effective_amount > 0, ErrorCode::InvalidAmount);

    let index_increase = (effective_amount as u128)
        .checked_mul(PRECISION_FACTOR)
        .ok_or(ErrorCode::CalculationOverflow)?
        .checked_div(state.total_staked as u128)
        .ok_or(ErrorCode::CalculationOverflow)?;

    state.global_reward_index = state
        .global_reward_index
        .checked_add(index_increase)
        .ok_or(ErrorCode::CalculationOverflow)?;

    // Back-compute claimable aggregate. Integer division in the index calculation means
    // effective_amount may exceed the sum of what all users can claim; the dust stays in
    // the vault and is swept into the next settle_yield call via the vault-balance delta.
    let distributable_amount = u64::try_from(
        index_increase
            .checked_mul(state.total_staked as u128)
            .ok_or(ErrorCode::CalculationOverflow)?
            .checked_div(PRECISION_FACTOR)
            .ok_or(ErrorCode::CalculationOverflow)?,
    )
    .map_err(|_| ErrorCode::CalculationOverflow)?;

    state.total_yield_allocated = state
        .total_yield_allocated
        .checked_add(distributable_amount)
        .ok_or(ErrorCode::CalculationOverflow)?;

    emit!(YieldSettled {
        version: 1,
        effective_amount: distributable_amount,
        new_global_reward_index: state.global_reward_index,
        reward_mint: ctx.accounts.stake_mint.key(),
    });

    Ok(())
}
