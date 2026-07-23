use anchor_lang::prelude::*;
use anchor_spl::token::{self, CloseAccount, TokenAccount};

use crate::constants::*;
use crate::error::ErrorCode;
use crate::state::*;

/// Re-export shared calculation functions
pub use shared::utils::calculations::{
    safe_add_delay,
    calculate_minimum_amount_from_decimals,
    scale_amount_to_new_decimals,
};

/// Re-export shared validation functions
pub use shared::utils::validations::{
    validate_amount_meets_minimum,
    validate_sufficient_balance,
    validate_balance_zero_or_above_minimum,
    validate_amount_full_or_above_minimum,
    validate_timestamp_has_passed,
    validate_timestamp_has_not_passed,
};

/// Find a collateral token configuration (regardless of disabled status)
/// Returns Some(config) if the token exists, None otherwise
pub fn find_token_config<'a>(
    state: &'a ProgramState,
    token_mint: &Pubkey,
) -> Option<&'a CollateralTokenConfig> {
    state
        .collateral_tokens
        .iter()
        .find(|config| config.token_mint == *token_mint)
}

/// Find a collateral token configuration mutably (regardless of disabled status)
/// Returns Some(config) if the token exists, None otherwise
pub fn find_token_config_mut<'a>(
    state: &'a mut ProgramState,
    token_mint: &Pubkey,
) -> Option<&'a mut CollateralTokenConfig> {
    state
        .collateral_tokens
        .iter_mut()
        .find(|config| config.token_mint == *token_mint)
}

/// Find an enabled collateral token configuration
/// Returns Some(config) if the token exists and is enabled, None otherwise
pub fn find_enabled_token_config<'a>(
    state: &'a ProgramState,
    token_mint: &Pubkey,
) -> Option<&'a CollateralTokenConfig> {
    find_token_config(state, token_mint)
        .filter(|config| !config.disabled)
}

/// Verify that a user is whitelisted for minting/unminting operations
/// This function checks if the whitelist is enabled and validates the mint_whitelist PDA account
pub fn verify_mint_whitelist<'info>(
    program_id: &Pubkey,
    user: &Pubkey,
    mint_whitelist: &AccountInfo<'info>,
    is_whitelist_enabled: bool,
) -> Result<()> {
    if is_whitelist_enabled {
        // Derive expected whitelist PDA
        let (expected_whitelist_pda, _) = Pubkey::find_program_address(
            &[MINT_WHITELIST_SEED, user.as_ref()],
            program_id,
        );

        // Verify the provided account matches the expected PDA
        require!(
            mint_whitelist.key() == expected_whitelist_pda,
            ErrorCode::AddressNotWhitelisted
        );

        // Verify the account exists (has lamports > 0) and is owned by the program
        require!(
            mint_whitelist.lamports() > 0 && mint_whitelist.owner == program_id,
            ErrorCode::AddressNotWhitelisted
        );

        // Verify the account data size is at least the expected struct size
        // Account size >= 8 bytes discriminator + struct size
        let expected_size = 8 + MintWhitelistEntry::INIT_SPACE;
        require!(
            mint_whitelist.data_len() >= expected_size,
            ErrorCode::AddressNotWhitelisted
        );
    }

    Ok(())
}

/// Resolve the effective unmint cooldown for a user.
///
/// Returns the wallet's custom cooldown if all of the following are true:
///   - The whitelist is enabled
///   - The whitelist entry has a custom cooldown configured
///   - The 24-hour activation delay has passed
///
/// Falls back to the global cooldown in all other cases, including when the
/// whitelist is disabled (no whitelist account is required in that case).
///
/// Must be called after `verify_mint_whitelist` so the account is already validated.
pub fn resolve_unmint_cooldown(
    mint_whitelist: &AccountInfo,
    is_whitelist_enabled: bool,
    global_cooldown_seconds: i64,
    now: i64,
) -> Result<i64> {
    if !is_whitelist_enabled {
        return Ok(global_cooldown_seconds);
    }

    let data = mint_whitelist.try_borrow_data()?;
    let entry = MintWhitelistEntry::try_deserialize(&mut data.as_ref())
        .map_err(|_| error!(ErrorCode::AddressNotWhitelisted))?;

    if entry.has_custom_cooldown && now >= entry.custom_cooldown_effective_timestamp {
        // Defense-in-depth: the setter already validates >= 0, but guard here in case
        // of future bugs or direct account manipulation.
        require!(
            entry.custom_cooldown_seconds >= 0,
            ErrorCode::InvalidConfiguration
        );
        Ok(entry.custom_cooldown_seconds)
    } else {
        Ok(global_cooldown_seconds)
    }
}

/// Close an escrow token account using the SPL Token program's close_account instruction
/// This is used when closing escrow accounts that are owned by the SPL Token program
/// but controlled by a PDA authority
pub fn close_escrow_token_account<'a>(
    token_program: AccountInfo<'a>,
    escrow_token_account: &Account<'a, TokenAccount>,
    destination: AccountInfo<'a>,
    authority: AccountInfo<'a>,
    user: &Pubkey,
    claim_token_mint: &Pubkey,
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
                UNMINT_DETAILS_SEED,
                user.as_ref(),
                claim_token_mint.as_ref(),
                &[authority_bump],
            ]],
        ),
    )
}
