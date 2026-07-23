use anchor_lang::prelude::*;

pub use shared::utils::calculations::safe_add_delay as add_delay;

use crate::constants::PRECISION_FACTOR;
use crate::error::ErrorCode;
use crate::state::{ProgramState, UserStakeAccount};

/// Settle and auto-compound pending rewards in one step.
///
/// Computes rewards earned since the last snapshot and immediately compounds
/// them into `user_stake.staked_amount`, `state.total_staked`, and debits
/// `state.total_yield_allocated`. The snapshot is always advanced to the
/// current `state.global_reward_index`.
///
/// MUST be called BEFORE any `staked_amount` change in the caller.
///
/// Returns the amount compounded (0 if the index has not moved or
/// `staked_amount` is 0).
pub fn update_pending_rewards(
    state: &mut ProgramState,
    user_stake: &mut UserStakeAccount,
) -> Result<u64> {
    let index_delta = state
        .global_reward_index
        .checked_sub(user_stake.reward_index_snapshot)
        .ok_or(ErrorCode::RewardIndexInvariantViolated)?;

    let compounded = if index_delta > 0 && user_stake.staked_amount > 0 {
        let earned = (user_stake.staked_amount as u128)
            .checked_mul(index_delta)
            .ok_or(ErrorCode::CalculationOverflow)?
            .checked_div(PRECISION_FACTOR)
            .ok_or(ErrorCode::CalculationOverflow)?;
        let earned_u64 = u64::try_from(earned).map_err(|_| ErrorCode::CalculationOverflow)?;

        if earned_u64 > 0 {
            user_stake.staked_amount = user_stake
                .staked_amount
                .checked_add(earned_u64)
                .ok_or(ErrorCode::CalculationOverflow)?;
            state.total_staked = state
                .total_staked
                .checked_add(earned_u64)
                .ok_or(ErrorCode::CalculationOverflow)?;
            state.total_yield_allocated = state.total_yield_allocated.saturating_sub(earned_u64);
        }
        earned_u64
    } else {
        0
    };

    user_stake.reward_index_snapshot = state.global_reward_index;

    Ok(compounded)
}
