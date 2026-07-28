// SPDX-License-Identifier: BUSL-1.1
// Copyright (C) 2026 AbraFi Ltd., Business Source License 1.1.
// Change Date: 2nd anniversary of this version's publication.
// Change License: Apache-2.0. See LICENSE at repo root.

use anchor_lang::prelude::*;

mod constants;
mod error;
mod events;
mod instructions;
mod state;

#[allow(unused_imports)]
use instructions::*;

include!("declare_id.rs");

#[cfg(not(feature = "no-entrypoint"))]
solana_security_txt::security_txt! {
    name: "AbraFi Yield Router",
    project_url: "https://www.abrafi.org/",
    contacts: "email:security@abrafi.io",
    policy: "https://www.abrafi.org/security-policy",
    preferred_languages: "en",
    source_code: "https://github.com/AbraFI-Labs/abrafi-token-programs"
}

#[program]
pub mod abrafi_yield_router {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        min_distribution_amount: u64,
    ) -> Result<()> {
        initialize_handler(ctx, min_distribution_amount)
    }

    pub fn add_recipient(
        ctx: Context<AddRecipient>,
        recipient_type: state::RecipientType,
        staking_program_id: Pubkey,
    ) -> Result<()> {
        add_recipient_handler(ctx, recipient_type, staking_program_id)
    }

    pub fn set_recipient_enabled(ctx: Context<SetRecipientEnabled>, index: u8, enabled: bool) -> Result<()> {
        set_recipient_enabled_handler(ctx, index, enabled)
    }

    pub fn distribute_yield<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, DistributeYield<'info>>,
    ) -> Result<()> {
        distribute_yield_handler(ctx)
    }

    pub fn set_distribute_enabled(
        ctx: Context<SetDistributeEnabled>,
        enabled: bool,
    ) -> Result<()> {
        set_distribute_enabled_handler(ctx, enabled)
    }

    pub fn update_authority(ctx: Context<UpdateAuthority>) -> Result<()> {
        update_authority_handler(ctx)
    }

    pub fn finalize_authority(ctx: Context<FinalizeAuthority>) -> Result<()> {
        finalize_authority_handler(ctx)
    }

    pub fn update_operations_authority(ctx: Context<UpdateOperationsAuthority>) -> Result<()> {
        update_operations_authority_handler(ctx)
    }

    pub fn update_min_distribution_amount(
        ctx: Context<UpdateMinDistributionAmount>,
        new_amount: u64,
    ) -> Result<()> {
        update_min_distribution_amount_handler(ctx, new_amount)
    }
}
