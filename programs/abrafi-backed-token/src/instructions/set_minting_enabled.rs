use anchor_lang::prelude::*;
use anchor_lang::solana_program::program_option::COption;
use anchor_spl::token::Mint;

use crate::constants::*;
use crate::error::ErrorCode;
use crate::events::*;
use crate::state::*;

/// Set minting enabled instruction accounts
#[derive(Accounts)]
pub struct SetMintingEnabled<'info> {
    /// Program state account
    #[account(
        mut,
        seeds = [STATE_SEED],
        bump = state.state_bump,
        has_one = operations_authority,
        has_one = abrafi_backed_token_mint,
    )]
    pub state: Account<'info, ProgramState>,

    /// Operations authority that can set minting enabled
    pub operations_authority: Signer<'info>,

    /// The abrafi token mint
    pub abrafi_backed_token_mint: Account<'info, Mint>,
}

/// Set minting enabled status
/// This function can only be called by the operations authority
pub fn set_minting_enabled_handler(ctx: Context<SetMintingEnabled>, enabled: bool) -> Result<()> {
    let state = &mut ctx.accounts.state;

    require!(
        enabled != state.is_minting_enabled,
        ErrorCode::InvalidConfiguration
    );

    // If enabling minting, verify that the state PDA is still the mint authority
    if enabled {
        require!(
            ctx.accounts.abrafi_backed_token_mint.mint_authority == COption::Some(state.key()),
            ErrorCode::InvalidMint
        );
    }

    // Update the minting enabled flag
    state.is_minting_enabled = enabled;

    emit!(MintingEnabled {
        version: 1,
        is_enabled: state.is_minting_enabled,
    });

    Ok(())
}
