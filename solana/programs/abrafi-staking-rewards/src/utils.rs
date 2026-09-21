// SPDX-License-Identifier: BUSL-1.1
// Copyright (C) 2026 AbraFi Ltd., Business Source License 1.1.
// Change Date: 2nd anniversary of this version's publication.
// Change License: Apache-2.0. See LICENSE at repo root.

use anchor_lang::prelude::*;

pub use shared::utils::calculations::safe_add_delay as add_delay;

use crate::constants::PRECISION_FACTOR;
use crate::error::ErrorCode;
use crate::state::{ProgramState, UserStakeAccount};

/// Settle and optionally compound pending rewards.
///
/// When `compound` is true, drains `pending_rewards` plus newly earned into
/// `staked_amount`. When false, accumulates earned into `pending_rewards` only.
///
/// MUST be called BEFORE any `staked_amount` change in the caller.
pub fn update_pending_rewards(
    state: &mut ProgramState,
    user_stake: &mut UserStakeAccount,
    compound: bool,
) -> Result<u64> {
    let index_delta = state
        .global_reward_index
        .checked_sub(user_stake.reward_index_snapshot)
        .ok_or(ErrorCode::RewardIndexInvariantViolated)?;

    let newly_earned = if index_delta > 0 && user_stake.staked_amount > 0 {
        let earned = (user_stake.staked_amount as u128)
            .checked_mul(index_delta)
            .ok_or(ErrorCode::CalculationOverflow)?
            .checked_div(PRECISION_FACTOR)
            .ok_or(ErrorCode::CalculationOverflow)?;
        u64::try_from(earned).map_err(|_| ErrorCode::CalculationOverflow)?
    } else {
        0
    };

    user_stake.reward_index_snapshot = state.global_reward_index;

    if compound {
        // Drain pending_rewards + newly earned into staked_amount.
        let total = newly_earned
            .checked_add(user_stake.pending_rewards)
            .ok_or(ErrorCode::CalculationOverflow)?;
        if total > 0 {
            user_stake.staked_amount = user_stake
                .staked_amount
                .checked_add(total)
                .ok_or(ErrorCode::CalculationOverflow)?;
            state.total_staked = state
                .total_staked
                .checked_add(total)
                .ok_or(ErrorCode::CalculationOverflow)?;
            state.total_yield_allocated = state.total_yield_allocated.saturating_sub(total);
            user_stake.pending_rewards = 0;
        }
        Ok(total)
    } else {
        // Accumulate in pending_rewards; tokens remain in total_yield_allocated.
        if newly_earned > 0 {
            user_stake.pending_rewards = user_stake
                .pending_rewards
                .checked_add(newly_earned)
                .ok_or(ErrorCode::CalculationOverflow)?;
        }
        Ok(0)
    }
}
