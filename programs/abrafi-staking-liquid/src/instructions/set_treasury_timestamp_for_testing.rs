use anchor_lang::prelude::*;

use crate::constants::*;
use crate::state::*;

/// Test-only instruction to set treasury timestamp for testing
#[cfg(feature = "test")]
#[derive(Accounts)]
pub struct SetTreasuryTimestampForTesting<'info> {
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

/// Test-only instruction to set treasury cooldown end timestamp
/// This allows testing yield removal without waiting 24 hours
/// Only available in test environment (when compiled with test features)
#[cfg(feature = "test")]
pub fn set_treasury_timestamp_for_testing_handler(
    ctx: Context<SetTreasuryTimestampForTesting>,
    new_timestamp: i64,
) -> Result<()> {
    let state = &mut ctx.accounts.state;

    // Set the treasury cooldown end timestamp to the specified value
    // For testing, we can set it directly to any timestamp to bypass cooldown
    state.treasury_cooldown_end_timestamp = new_timestamp;

    msg!(
        "Set treasury cooldown end timestamp to {} for testing",
        new_timestamp
    );

    Ok(())
}
