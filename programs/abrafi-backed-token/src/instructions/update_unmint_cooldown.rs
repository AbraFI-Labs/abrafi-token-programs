use anchor_lang::prelude::*;

use crate::constants::*;
use crate::error::ErrorCode;
use crate::events::*;
use crate::state::*;

/// Update unmint cooldown instruction accounts
#[derive(Accounts)]
pub struct UpdateUnmintCooldown<'info> {
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

/// Update the unmint cooldown period
/// Only the authority can change the cooldown period
/// Maximum cooldown period is 30 days (720 hours)
pub fn update_unmint_cooldown_handler(
    ctx: Context<UpdateUnmintCooldown>,
    new_cooldown_seconds: i64,
) -> Result<()> {
    let state = &mut ctx.accounts.state;

    require!(
        new_cooldown_seconds >= 0
            && new_cooldown_seconds <= MAX_COOLDOWN_SECONDS
            && new_cooldown_seconds != state.unmint_cooldown_seconds,
        ErrorCode::InvalidConfiguration
    );

    // Update the unmint cooldown period
    state.unmint_cooldown_seconds = new_cooldown_seconds;

    emit!(UnmintCooldownUpdated {
        version: 1,
        new_cooldown_seconds: new_cooldown_seconds,
    });

    Ok(())
}
