// SPDX-License-Identifier: BUSL-1.1
// Copyright (C) 2026 AbraFi Ltd., Business Source License 1.1.
// Change Date: 2nd anniversary of this version's publication.
// Change License: Apache-2.0. See LICENSE at repo root.

use anchor_lang::prelude::*;
use crate::constants::STATE_SEED;
use crate::state::ProgramState;

#[cfg(feature = "test")]
#[derive(Accounts)]
pub struct SetPendingAuthorityTimestampForTesting<'info> {
    #[account(mut, seeds = [STATE_SEED], bump = state.bump, has_one = authority)]
    pub state: Account<'info, ProgramState>,
    pub authority: Signer<'info>,
}

#[cfg(feature = "test")]
pub fn set_pending_authority_timestamp_for_testing_handler(
    ctx: Context<SetPendingAuthorityTimestampForTesting>,
    new_timestamp: i64,
) -> Result<()> {
    ctx.accounts.state.pending_authority_expiration_timestamp = new_timestamp;
    Ok(())
}
