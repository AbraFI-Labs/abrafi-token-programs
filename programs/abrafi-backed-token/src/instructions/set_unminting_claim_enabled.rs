use anchor_lang::prelude::*;

use crate::constants::*;
use crate::error::ErrorCode;
use crate::events::UnmintingClaimEnabled;
use crate::state::*;

/// Set unminting claim enabled instruction accounts
#[derive(Accounts)]
pub struct SetUnmintingClaimEnabled<'info> {
    /// Program state account
    #[account(
        mut,
        seeds = [STATE_SEED],
        bump = state.state_bump,
        has_one = operations_authority,
    )]
    pub state: Account<'info, ProgramState>,

    /// Operations authority that can set unminting claim enabled
    pub operations_authority: Signer<'info>,
}

/// Set unminting claim enabled status
/// This function can only be called by the operations authority
pub fn set_unminting_claim_enabled_handler(ctx: Context<SetUnmintingClaimEnabled>, enabled: bool) -> Result<()> {
    let state = &mut ctx.accounts.state;

    require!(
        enabled != state.is_unminting_claim_enabled,
        ErrorCode::InvalidConfiguration
    );

    // Update the unminting claim enabled flag
    state.is_unminting_claim_enabled = enabled;

    emit!(UnmintingClaimEnabled {
        version: 1,
        is_enabled: state.is_unminting_claim_enabled,
    });

    Ok(())
}

