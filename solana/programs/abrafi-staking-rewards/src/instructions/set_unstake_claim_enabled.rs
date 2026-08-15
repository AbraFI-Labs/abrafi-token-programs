// SPDX-License-Identifier: BUSL-1.1
// Copyright (C) 2026 AbraFi Ltd., Business Source License 1.1.
// Change Date: 2nd anniversary of this version's publication.
// Change License: Apache-2.0. See LICENSE at repo root.

use anchor_lang::prelude::*;

use crate::constants::STATE_SEED;
use crate::error::ErrorCode;
use crate::events::UnstakeClaimEnabledChanged;
use crate::state::ProgramState;

#[derive(Accounts)]
pub struct SetUnstakeClaimEnabled<'info> {
    #[account(
        mut,
        seeds = [STATE_SEED],
        bump = state.bump,
        has_one = operations_authority,
    )]
    pub state: Account<'info, ProgramState>,

    pub operations_authority: Signer<'info>,
}

pub fn set_unstake_claim_enabled_handler(
    ctx: Context<SetUnstakeClaimEnabled>,
    enabled: bool,
) -> Result<()> {
    require!(
        enabled != ctx.accounts.state.unstake_claim_enabled,
        ErrorCode::NoChange
    );

    ctx.accounts.state.unstake_claim_enabled = enabled;

    emit!(UnstakeClaimEnabledChanged {
        version: 1,
        enabled,
    });

    Ok(())
}
