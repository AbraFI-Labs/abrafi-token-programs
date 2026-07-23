use anchor_lang::prelude::*;

use crate::constants::STATE_SEED;
use crate::error::ErrorCode;
use crate::events::UnstakeRequestEnabledChanged;
use crate::state::ProgramState;

#[derive(Accounts)]
pub struct SetUnstakeRequestEnabled<'info> {
    #[account(
        mut,
        seeds = [STATE_SEED],
        bump = state.bump,
        has_one = operations_authority,
    )]
    pub state: Account<'info, ProgramState>,

    pub operations_authority: Signer<'info>,
}

pub fn set_unstake_request_enabled_handler(
    ctx: Context<SetUnstakeRequestEnabled>,
    enabled: bool,
) -> Result<()> {
    require!(
        enabled != ctx.accounts.state.unstake_request_enabled,
        ErrorCode::NoChange
    );

    ctx.accounts.state.unstake_request_enabled = enabled;

    emit!(UnstakeRequestEnabledChanged {
        version: 1,
        enabled,
    });

    Ok(())
}
