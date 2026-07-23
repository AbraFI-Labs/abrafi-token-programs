use anchor_lang::prelude::*;

use crate::constants::STATE_SEED;
use crate::error::ErrorCode;
use crate::events::StakeEnabledChanged;
use crate::state::ProgramState;

#[derive(Accounts)]
pub struct SetStakeEnabled<'info> {
    #[account(
        mut,
        seeds = [STATE_SEED],
        bump = state.bump,
        has_one = operations_authority,
    )]
    pub state: Account<'info, ProgramState>,

    pub operations_authority: Signer<'info>,
}

pub fn set_stake_enabled_handler(ctx: Context<SetStakeEnabled>, enabled: bool) -> Result<()> {
    require!(
        enabled != ctx.accounts.state.stake_enabled,
        ErrorCode::NoChange
    );

    ctx.accounts.state.stake_enabled = enabled;

    emit!(StakeEnabledChanged {
        version: 1,
        enabled,
    });

    Ok(())
}
