use anchor_lang::prelude::*;

use crate::constants::*;
use crate::state::*;

/// Test-only instruction to set pending mint authority expiration timestamp for testing
#[cfg(feature = "test")]
#[derive(Accounts)]
pub struct SetPendingMintAuthorityTimestampForTesting<'info> {
    /// Program state account
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

/// Test-only instruction to set pending mint authority expiration timestamp to any value
/// This allows testing mint authority expiration scenarios without waiting
/// Can be set to past values to test expiration or future values to test non-expiration
/// Only available in test environment (when compiled with test features)
#[cfg(feature = "test")]
pub fn set_pending_mint_authority_timestamp_for_testing_handler(
    ctx: Context<SetPendingMintAuthorityTimestampForTesting>,
    new_timestamp: i64,
) -> Result<()> {
    let state = &mut ctx.accounts.state;

    // Set the pending mint authority expiration timestamp to the specified value
    // For testing, we can set it to any timestamp to test expiration or non-expiration scenarios
    state.pending_mint_authority_expiration_timestamp = new_timestamp;

    msg!(
        "Set pending mint authority expiration timestamp to {} for testing",
        new_timestamp
    );

    Ok(())
}
