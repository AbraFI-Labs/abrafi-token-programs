// programs/abrafi-backed-token/src/instructions/disable_collateral_token.rs
use anchor_lang::prelude::*;
use anchor_spl::token::Mint;

use crate::constants::*;
use crate::error::ErrorCode;
use crate::events::*;
use crate::state::*;
use crate::utils::*;

/// Disable collateral token instruction accounts
#[derive(Accounts)]
pub struct DisableCollateralToken<'info> {
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

    /// The token mint to disable
    /// Validated against the data structure to ensure it's configured
    pub token_mint_to_disable: Account<'info, Mint>,
}

/// Disable a collateral token type from the program
/// Disabled tokens cannot be used for minting or new unmint requests
/// They remain in the data structure to allow withdrawing funds from vaults
/// and completing existing unmint requests
pub fn disable_collateral_token_handler(
    ctx: Context<DisableCollateralToken>,
) -> Result<()> {
    let state = &mut ctx.accounts.state;
    let token_mint_key = ctx.accounts.token_mint_to_disable.key();

    // Find and disable the token configuration - validates it exists in the data structure
    let token_config = find_token_config_mut(state, &token_mint_key)
        .ok_or(ErrorCode::TokenNotConfigured)?;

    // Check if token is already disabled
    require!(!token_config.disabled, ErrorCode::InvalidConfiguration);

    // Disable the token
    token_config.disabled = true;

    emit!(CollateralTokenDisabled {
        version: 1,
        name: token_config.name.clone(),
        token_mint: token_mint_key,
    });

    Ok(())
}

