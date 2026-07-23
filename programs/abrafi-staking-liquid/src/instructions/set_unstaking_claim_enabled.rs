use anchor_lang::prelude::*;

use crate::constants::*;
use crate::error::ErrorCode;
use crate::events::UnstakingClaimEnabled;
use crate::state::*;

/// Set unstaking claim enabled instruction accounts
#[derive(Accounts)]
pub struct SetUnstakingClaimEnabled<'info> {
    /// Liquid staking state account
    #[account(
        mut,
        seeds = [STATE_SEED],
        bump = state.state_bump,
        has_one = operations_authority,
    )]
    pub state: Account<'info, ProgramState>,

    /// Operations authority that can set unstaking claim enabled
    pub operations_authority: Signer<'info>,
}

/// Set unstaking claim enabled status
/// This function can only be called by the operations authority
pub fn set_unstaking_claim_enabled_handler(ctx: Context<SetUnstakingClaimEnabled>, enabled: bool) -> Result<()> {
    let state = &mut ctx.accounts.state;

    require!(
        enabled != state.is_unstaking_claim_enabled,
        ErrorCode::InvalidConfiguration
    );

    // Update the unstaking claim enabled flag
    state.is_unstaking_claim_enabled = enabled;

    emit!(UnstakingClaimEnabled {
        version: 1,
        is_enabled: state.is_unstaking_claim_enabled,
    });

    Ok(())
}
