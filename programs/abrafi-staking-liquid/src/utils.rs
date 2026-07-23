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

/// Convert underlying amount to liquid staking token amount (shares) using conversion rate calculation with vault balance
pub fn convert_to_shares(
    underlying_amount: u64,
    vault_underlying_balance: u64,
    liquid_staking_token_supply: u64,
) -> Result<u64> {
    // Validate non-zero input
    require!(underlying_amount > 0, ErrorCode::InvalidAmount);

    // If this is the first staker (total supply is zero), return underlying_amount directly
    if liquid_staking_token_supply == 0 {
        // Prevent first stake/restake if vault already has a balance (yield must be removed first)
        // This ensures the first stake establishes a correct 1:1 exchange rate
        require!(
            vault_underlying_balance == 0,
            ErrorCode::InvalidVaultBalance
        );
        return Ok(underlying_amount);
    }

    // Reject if no vault balance to back the shares
    require!(
        vault_underlying_balance > 0,
        ErrorCode::InsufficientLiquidity
    );

    // Calculate shares: (underlying_amount * liquid_staking_token_supply) / vault_underlying_balance
    let shares = (underlying_amount as u128)
        .checked_mul(liquid_staking_token_supply as u128)
        .ok_or(Error::from(ErrorCode::CalculationOverflow))?
        .checked_div(vault_underlying_balance as u128)
        .ok_or(Error::from(ErrorCode::CalculationOverflow))?;

    // Reject deposits/restakes that would mint zero shares
    require!(shares > 0, ErrorCode::InvalidAmount);

    // Convert to u64, ensuring no truncation
    let shares_u64 = u64::try_from(shares).map_err(|_| ErrorCode::CalculationOverflow)?;
    Ok(shares_u64)
}

/// Convert liquid staking token amount to underlying amount (assets) using conversion rate calculation with vault balance
pub fn convert_to_assets(
    liquid_staking_amount: u64,
    vault_underlying_balance: u64,
    liquid_staking_token_supply: u64,
) -> Result<u64> {
    // Validate non-zero input
    require!(liquid_staking_amount > 0, ErrorCode::InvalidAmount);

    // Ensure there are underlying tokens to back the conversion (prevents division by zero)
    require!(
        liquid_staking_token_supply > 0,
        ErrorCode::InsufficientLiquidity
    );

    // Ensure the requested amount doesn't exceed the total supply
    require!(
        liquid_staking_amount <= liquid_staking_token_supply,
        ErrorCode::InvalidAmount
    );

    // Ensure the vault has assets to back the conversion
    require!(
        vault_underlying_balance > 0,
        ErrorCode::InsufficientLiquidity
    );

    // Calculate assets: (liquid_staking_amount * vault_underlying_balance) / liquid_staking_token_supply
    let assets = (liquid_staking_amount as u128)
        .checked_mul(vault_underlying_balance as u128)
        .ok_or(Error::from(ErrorCode::CalculationOverflow))?
        .checked_div(liquid_staking_token_supply as u128)
        .ok_or(Error::from(ErrorCode::CalculationOverflow))?;

    // Reject conversions that would yield zero assets
    require!(assets > 0, ErrorCode::InvalidAmount);

    // Convert to u64, ensuring no truncation
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
