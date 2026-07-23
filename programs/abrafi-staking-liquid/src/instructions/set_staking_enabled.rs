use anchor_lang::prelude::*;
use anchor_lang::solana_program::program_option::COption;
use anchor_spl::token::Mint;

use crate::constants::*;
use crate::error::ErrorCode;
use crate::events::StakingEnabled;
use crate::state::*;

/// Set staking enabled instruction accounts
#[derive(Accounts)]
pub struct SetStakingEnabled<'info> {
    /// Liquid staking state account
    #[account(
        mut,
        seeds = [STATE_SEED],
        bump = state.state_bump,
        has_one = operations_authority,
        has_one = liquid_staking_token_mint,
    )]
    pub state: Account<'info, ProgramState>,

    /// Operations authority that can set staking enabled
    pub operations_authority: Signer<'info>,

    /// The liquid staking token mint
    pub liquid_staking_token_mint: Account<'info, Mint>,
}

/// Set staking enabled status
/// This function can only be called by the operations authority
pub fn set_staking_enabled_handler(ctx: Context<SetStakingEnabled>, enabled: bool) -> Result<()> {
    let state = &mut ctx.accounts.state;

    require!(
        enabled != state.is_staking_enabled,
        ErrorCode::InvalidConfiguration
    );

    // If enabling staking, verify that the state PDA is still the mint authority
    if enabled {
        require!(
            ctx.accounts.liquid_staking_token_mint.mint_authority == COption::Some(state.key()),
            ErrorCode::InvalidMint
        );
    }

    // Update the staking enabled flag
    state.is_staking_enabled = enabled;

    emit!(StakingEnabled {
        version: 1,
        is_enabled: state.is_staking_enabled,
    });

    Ok(())
}
