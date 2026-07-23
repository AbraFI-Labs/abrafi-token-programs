use anchor_lang::prelude::*;

use crate::constants::STATE_SEED;
use crate::error::ErrorCode;
use crate::events::ClaimYieldEnabledChanged;
use crate::state::ProgramState;

#[derive(Accounts)]
pub struct SetClaimYieldEnabled<'info> {
    #[account(
        mut,
        seeds = [STATE_SEED],
        bump = state.bump,
        has_one = operations_authority,
    )]
    pub state: Account<'info, ProgramState>,

    pub operations_authority: Signer<'info>,
}

pub fn set_claim_yield_enabled_handler(
    ctx: Context<SetClaimYieldEnabled>,
    enabled: bool,
) -> Result<()> {
    require!(
        enabled != ctx.accounts.state.claim_yield_enabled,
        ErrorCode::NoChange
    );

    ctx.accounts.state.claim_yield_enabled = enabled;
    emit!(ClaimYieldEnabledChanged { version: 1, enabled });
    Ok(())
}
