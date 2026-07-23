use anchor_lang::prelude::*;

use crate::constants::*;
use crate::error::ErrorCode;
use crate::events::MaxRemovalPercentageUpdated;
use crate::state::*;

/// Update maximum removal percentage instruction accounts
#[derive(Accounts)]
pub struct UpdateMaxRemovalPercentage<'info> {
    /// Liquid staking state account
    #[account(
        mut,
        seeds = [STATE_SEED],
        bump = state.state_bump,
        has_one = authority,
    )]
    pub state: Account<'info, ProgramState>,

    /// Authority that can update maximum removal percentage
    pub authority: Signer<'info>,
}

/// Update the maximum percentage of vault balance that can be removed per remove_yield call
/// The percentage is specified in basis points (e.g., 0 = disabled, 500 = 5%, 10000 = 100%)
/// Setting to 0 disables yield removal entirely
/// Only callable by the main authority
pub fn update_max_removal_percentage_handler(
    ctx: Context<UpdateMaxRemovalPercentage>,
    new_max_removal_percentage: u16,
) -> Result<()> {
    let state = &mut ctx.accounts.state;

    // Validate that the new percentage is between 0 and 10000 (0 = disabled, 0.01% to 100%)
    // 0 disables yield removal, > 10000 would exceed 100%
    require!(
        new_max_removal_percentage <= 10000,
        ErrorCode::InvalidConfiguration
    );

    // Update the maximum removal percentage
    state.max_removal_percentage = new_max_removal_percentage;

    emit!(MaxRemovalPercentageUpdated {
        version: 1,
        new_max_removal_percentage,
    });

    Ok(())
}
