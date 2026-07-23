use anchor_lang::prelude::*;
use anchor_lang::solana_program::program_option::COption;
use anchor_spl::token::Mint;

use crate::constants::*;
use crate::error::ErrorCode;
use crate::events::UnmintingRequestEnabled;
use crate::state::*;

/// Set unminting request enabled instruction accounts
#[derive(Accounts)]
pub struct SetUnmintingRequestEnabled<'info> {
    /// Program state account
    #[account(
        mut,
        seeds = [STATE_SEED],
        bump = state.state_bump,
        has_one = operations_authority,
        has_one = abrafi_backed_token_mint,
    )]
    pub state: Account<'info, ProgramState>,

    /// Operations authority that can set unminting request enabled
    pub operations_authority: Signer<'info>,

    /// The abrafi token mint
    pub abrafi_backed_token_mint: Account<'info, Mint>,
}

/// Set unminting request enabled status
/// This function can only be called by the operations authority
pub fn set_unminting_request_enabled_handler(ctx: Context<SetUnmintingRequestEnabled>, enabled: bool) -> Result<()> {
    let state = &mut ctx.accounts.state;

    require!(
        enabled != state.is_unminting_request_enabled,
        ErrorCode::InvalidConfiguration
    );

    // If enabling request unminting, verify that the state PDA is still the mint authority
    if enabled {
        require!(
            ctx.accounts.abrafi_backed_token_mint.mint_authority == COption::Some(state.key()),
            ErrorCode::InvalidMint
        );
    }

    // Update the unminting request enabled flag
    state.is_unminting_request_enabled = enabled;

    emit!(UnmintingRequestEnabled {
        version: 1,
        is_enabled: state.is_unminting_request_enabled,
    });

    Ok(())
}

