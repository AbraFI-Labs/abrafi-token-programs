// SPDX-License-Identifier: BUSL-1.1
// Copyright (C) 2026 AbraFi Ltd., Business Source License 1.1.
// Change Date: 2nd anniversary of this version's publication.
// Change License: Apache-2.0. See LICENSE at repo root.

use anchor_lang::prelude::*;
use anchor_spl::token::{self, CloseAccount, TokenAccount};

use crate::constants::*;
use crate::error::ErrorCode;

/// Re-export shared calculation functions
pub use shared::utils::calculations::{
    safe_add_delay,
    calculate_minimum_amount_from_decimals,
};

/// Re-export shared validation functions
pub use shared::utils::validations::{
    validate_amount_meets_minimum,
    validate_sufficient_balance,
    validate_balance_zero_or_above_minimum,
    validate_amount_full_or_above_minimum,
    validate_timestamp_has_passed,
};

/// Returns 10^decimals as the virtual offset for share/asset conversions.
/// This permanently locks a small virtual deposit in the pool, making donation attacks
/// economically infeasible while having negligible impact on the exchange rate at scale.
fn virtual_offset_for(decimals: u8) -> Result<u64> {
    10u64
        .checked_pow(decimals as u32)
        .ok_or(Error::from(ErrorCode::CalculationOverflow))
}

/// Convert underlying amount to liquid staking tokens: underlying * (supply + V) / (vault + V)
pub fn convert_to_shares(
    underlying_amount: u64,
    vault_underlying_balance: u64,
    liquid_staking_token_supply: u64,
    underlying_decimals: u8,
) -> Result<u64> {
    require!(underlying_amount > 0, ErrorCode::InvalidAmount);

    let virtual_offset = virtual_offset_for(underlying_decimals)?;

    let effective_supply = (liquid_staking_token_supply as u128)
        .checked_add(virtual_offset as u128)
        .ok_or(Error::from(ErrorCode::CalculationOverflow))?;
    let effective_vault = (vault_underlying_balance as u128)
        .checked_add(virtual_offset as u128)
        .ok_or(Error::from(ErrorCode::CalculationOverflow))?;

    let shares = (underlying_amount as u128)
        .checked_mul(effective_supply)
        .ok_or(Error::from(ErrorCode::CalculationOverflow))?
        .checked_div(effective_vault)
        .ok_or(Error::from(ErrorCode::CalculationOverflow))?;

    require!(shares > 0, ErrorCode::InvalidAmount);

    let shares_u64 = u64::try_from(shares).map_err(|_| ErrorCode::CalculationOverflow)?;
    Ok(shares_u64)
}

/// Convert liquid staking tokens to underlying amount: liquid_staking_token_amount * (vault + V) / (supply + V)
pub fn convert_to_assets(
    liquid_staking_amount: u64,
    vault_underlying_balance: u64,
    liquid_staking_token_supply: u64,
    underlying_decimals: u8,
) -> Result<u64> {
    require!(liquid_staking_amount > 0, ErrorCode::InvalidAmount);

    let virtual_offset = virtual_offset_for(underlying_decimals)?;

    require!(
        liquid_staking_amount <= liquid_staking_token_supply,
        ErrorCode::InvalidAmount
    );

    let effective_vault = (vault_underlying_balance as u128)
        .checked_add(virtual_offset as u128)
        .ok_or(Error::from(ErrorCode::CalculationOverflow))?;
    let effective_supply = (liquid_staking_token_supply as u128)
        .checked_add(virtual_offset as u128)
        .ok_or(Error::from(ErrorCode::CalculationOverflow))?;

    let assets = (liquid_staking_amount as u128)
        .checked_mul(effective_vault)
        .ok_or(Error::from(ErrorCode::CalculationOverflow))?
        .checked_div(effective_supply)
        .ok_or(Error::from(ErrorCode::CalculationOverflow))?;

    require!(assets > 0, ErrorCode::InvalidAmount);

    let assets_u64 = u64::try_from(assets).map_err(|_| ErrorCode::CalculationOverflow)?;
    Ok(assets_u64)
}

/// Close an escrow token account using the SPL Token program's close_account instruction
pub fn close_escrow_token_account<'a>(
    token_program: AccountInfo<'a>,
    escrow_token_account: &Account<'a, TokenAccount>,
    destination: AccountInfo<'a>,
    authority: AccountInfo<'a>,
    user: &Pubkey,
    authority_bump: u8,
) -> Result<()> {
    token::close_account(
        CpiContext::new_with_signer(
            token_program,
            CloseAccount {
                account: escrow_token_account.to_account_info(),
                destination,
                authority,
            },
            &[&[
                USER_UNSTAKE_REQUEST_SEED,
                user.as_ref(),
                &[authority_bump],
            ]],
        ),
    )
}
