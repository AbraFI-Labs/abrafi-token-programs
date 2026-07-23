use anchor_lang::prelude::*;

use crate::constants::*;
use crate::error::ErrorCode;
use crate::events::*;
use crate::state::*;

/// Update request expiration instruction accounts
#[derive(Accounts)]
pub struct UpdateRequestExpiration<'info> {
    /// Program state account
    #[account(
        mut,
        seeds = [STATE_SEED],
        bump = state.state_bump,
        has_one = authority,
    )]
    pub state: Account<'info, ProgramState>,

    /// Authority that can update program configuration
    pub authority: Signer<'info>,
}

/// Update the request expiration period
/// Only the authority can change the expiration period
/// Minimum expiration period is 1 hour (3600 seconds)
/// Maximum expiration period is 90 days (2160 hours)
pub fn update_request_expiration_handler(
    ctx: Context<UpdateRequestExpiration>,
    new_expiration_seconds: i64,
) -> Result<()> {
    let state = &mut ctx.accounts.state;

    require!(
        new_expiration_seconds >= MIN_EXPIRATION_SECONDS
            && new_expiration_seconds <= MAX_EXPIRATION_SECONDS
            && new_expiration_seconds != state.request_expiration_seconds,
        ErrorCode::InvalidConfiguration
    );

    // Update the request expiration period
    state.request_expiration_seconds = new_expiration_seconds;

    emit!(RequestExpirationUpdated {
        version: 1,
        new_expiration_seconds: new_expiration_seconds,
    });

    Ok(())
}

/// Test-only instruction to set request expiration period without bounds checks
/// This allows testing expiration scenarios by setting expiration to zero or other values
/// that would normally be rejected by the validation checks
/// Only available in test environment (when compiled with test features)
#[cfg(feature = "test")]
pub fn set_request_expiration_for_testing_handler(
    ctx: Context<UpdateRequestExpiration>,
    new_expiration_seconds: i64,
) -> Result<()> {
    let state = &mut ctx.accounts.state;

    // Bypass all validation checks for testing purposes
    // This allows setting expiration to zero to test expiration scenarios
    state.request_expiration_seconds = new_expiration_seconds;

    msg!(
        "Set request expiration to {} seconds for testing (bypassed validation)",
        new_expiration_seconds
    );

    Ok(())
}
