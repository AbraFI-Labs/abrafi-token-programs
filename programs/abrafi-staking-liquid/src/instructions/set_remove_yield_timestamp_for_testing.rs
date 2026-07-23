use anchor_lang::prelude::*;

use crate::constants::*;
use crate::state::*;

/// Test-only instruction to set remove yield timestamp for testing
#[cfg(feature = "test")]
#[derive(Accounts)]
pub struct SetRemoveYieldTimestampForTesting<'info> {
    /// Liquid staking state account
    #[account(
        mut,
        seeds = [STATE_SEED],
        bump = state.state_bump,
        has_one = authority,
    )]
    pub state: Account<'info, ProgramState>,

    /// Authority that can update configuration
    pub authority: Signer<'info>,
}

/// Test-only instruction to set remove_yield_cooldown_end_timestamp
/// This allows testing remove_yield cooldown without waiting 24 hours
/// Only available in test environment (when compiled with test features)
#[cfg(feature = "test")]
pub fn set_remove_yield_timestamp_for_testing_handler(
    ctx: Context<SetRemoveYieldTimestampForTesting>,
    new_timestamp: i64,
) -> Result<()> {
    let state = &mut ctx.accounts.state;

    // Set the remove_yield_cooldown_end_timestamp to the specified value
    // For testing, we can set it directly to any timestamp to bypass cooldown
    state.remove_yield_cooldown_end_timestamp = new_timestamp;

    msg!(
        "Set remove_yield_cooldown_end_timestamp to {} for testing",
        new_timestamp
    );

    Ok(())
}
