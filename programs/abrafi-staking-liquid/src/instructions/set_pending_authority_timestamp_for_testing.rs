// SPDX-License-Identifier: BUSL-1.1
// Copyright (C) 2026 AbraFi Ltd., Business Source License 1.1.
// Change Date: 2nd anniversary of this version's publication.
// Change License: Apache-2.0. See LICENSE at repo root.

use anchor_lang::prelude::*;

use crate::constants::*;
use crate::state::*;

/// Test-only instruction to set pending authority expiration timestamp for testing
#[cfg(feature = "test")]
#[derive(Accounts)]
pub struct SetPendingAuthorityTimestampForTesting<'info> {
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

/// Test-only instruction to set pending authority expiration timestamp
/// This allows testing authority expiration without waiting 24 hours
/// Only available in test environment (when compiled with test features)
#[cfg(feature = "test")]
pub fn set_pending_authority_timestamp_for_testing_handler(
    ctx: Context<SetPendingAuthorityTimestampForTesting>,
    new_timestamp: i64,
) -> Result<()> {
    let state = &mut ctx.accounts.state;

    // Set the pending authority expiration timestamp to the specified value
    // For testing, we can set it directly to any timestamp to test expiration
    state.pending_authority_expiration_timestamp = new_timestamp;

    msg!(
        "Set pending authority expiration timestamp to {} for testing",
        new_timestamp
    );

    Ok(())
}
