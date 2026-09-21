// SPDX-License-Identifier: BUSL-1.1
// Copyright (C) 2026 AbraFi Ltd., Business Source License 1.1.
// Change Date: 2nd anniversary of this version's publication.
// Change License: Apache-2.0. See LICENSE at repo root.

use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::token::{Mint, TokenAccount};

use crate::error::ErrorCode;
use crate::state::{ProgramState, UserUnmintDetails};
use crate::constants::*;

/// Expands a legacy 66-byte UserUnmintDetails account to the current layout, setting
/// version = 1 and requested_amount from the current escrow balance.
/// Permissionless: any payer can migrate any account — no user signature required.
/// Idempotent: returns Ok(()) if the account is already the current size.
#[derive(Accounts)]
pub struct MigrateUnmintDetails<'info> {
    /// Pays the rent top-up if the account needs to grow. Can be anyone.
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        seeds = [STATE_SEED],
        bump = state.state_bump,
        has_one = abrafi_backed_token_mint,
    )]
    pub state: Account<'info, ProgramState>,

    pub abrafi_backed_token_mint: Account<'info, Mint>,

    /// CHECK: Verified to be a UserUnmintDetails account (discriminator + size) in the handler.
    #[account(mut)]
    pub user_unmint_details: UncheckedAccount<'info>,

    /// Canonical escrow ATA: abrafi_backed_token_mint × user_unmint_details PDA.
    /// The current escrow balance is used as requested_amount.
    #[account(
        associated_token::mint = abrafi_backed_token_mint,
        associated_token::authority = user_unmint_details,
    )]
    pub escrow_token_account: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
}

pub fn migrate_unmint_details_handler(ctx: Context<MigrateUnmintDetails>) -> Result<()> {
    let account = ctx.accounts.user_unmint_details.to_account_info();

    let legacy_size: usize = 66;
    let current_size: usize = 8 + UserUnmintDetails::INIT_SPACE as usize;

    {
        let data = account.try_borrow_data()?;

        let disc: &[u8] = &UserUnmintDetails::DISCRIMINATOR;
        if data.len() < 8 || &data[..8] != disc {
            return Err(ProgramError::InvalidAccountData.into());
        }

        if data.len() == current_size {
            return Ok(());
        }

        // Only the known legacy 66-byte layout is an upgrade candidate.
        if data.len() != legacy_size {
            return Err(ProgramError::InvalidAccountData.into());
        }
    }

    let requested_amount = ctx.accounts.escrow_token_account.amount;
    require!(requested_amount > 0, ErrorCode::InvalidAmount);

    let rent = Rent::get()?;
    let minimum_balance = rent.minimum_balance(current_size);
    let current_lamports = account.lamports();
    if current_lamports < minimum_balance {
        system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.payer.to_account_info(),
                    to: account.clone(),
                },
            ),
            minimum_balance - current_lamports,
        )?;
    }

    account.realloc(current_size, true)?;

    let mut data = account.try_borrow_mut_data()?;
    let mut slice: &[u8] = &*data;
    let mut details = UserUnmintDetails::try_deserialize(&mut slice)
        .map_err(|_| ProgramError::InvalidAccountData)?;

    details.version = 1;
    details.requested_amount = requested_amount;

    let mut writer: &mut [u8] = &mut data;
    details
        .try_serialize(&mut writer)
        .map_err(|_| ProgramError::InvalidAccountData)?;

    Ok(())
}
