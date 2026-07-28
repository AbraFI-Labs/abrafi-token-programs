// SPDX-License-Identifier: BUSL-1.1
// Copyright (C) 2026 AbraFi Ltd., Business Source License 1.1.
// Change Date: 2nd anniversary of this version's publication.
// Change License: Apache-2.0. See LICENSE at repo root.

use anchor_lang::prelude::*;

use crate::constants::*;
use crate::error::ErrorCode;
use crate::events::*;
use crate::state::*;

/// Set mint whitelist enabled instruction accounts
#[derive(Accounts)]
pub struct SetMintWhitelistEnabled<'info> {
    /// Program state account
    #[account(
        mut,
        seeds = [STATE_SEED],
        bump = state.state_bump,
        has_one = authority,
    )]
    pub state: Account<'info, ProgramState>,

    /// Program authority that can set mint whitelist enabled
    pub authority: Signer<'info>,
}

/// Set mint whitelist enabled status
/// This function can only be called by the program authority
pub fn set_mint_whitelist_enabled_handler(ctx: Context<SetMintWhitelistEnabled>, enabled: bool) -> Result<()> {
    let state = &mut ctx.accounts.state;

    require!(
        enabled != state.is_mint_whitelist_enabled,
        ErrorCode::InvalidConfiguration
    );

    // Update the mint whitelist enabled flag
    state.is_mint_whitelist_enabled = enabled;

    emit!(MintWhitelistEnabled {
        version: 1,
        is_enabled: state.is_mint_whitelist_enabled,
    });

    Ok(())
}

