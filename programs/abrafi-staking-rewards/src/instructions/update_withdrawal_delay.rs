use anchor_lang::prelude::*;

use crate::constants::{STATE_SEED, MAX_WITHDRAWAL_DELAY};
use crate::error::ErrorCode;
use crate::events::WithdrawalDelayUpdated;
use crate::state::ProgramState;

#[derive(Accounts)]
pub struct UpdateWithdrawalDelay<'info> {
    #[account(
        mut,
        seeds = [STATE_SEED],
        bump = state.bump,
        has_one = operations_authority,
    )]
    pub state: Account<'info, ProgramState>,

    pub operations_authority: Signer<'info>,
}

pub fn update_withdrawal_delay_handler(
    ctx: Context<UpdateWithdrawalDelay>,
    new_delay: i64,
) -> Result<()> {
    require!(
        new_delay >= 0 && new_delay <= MAX_WITHDRAWAL_DELAY,
        ErrorCode::InvalidConfiguration
    );
    require!(
        new_delay != ctx.accounts.state.withdrawal_delay,
        ErrorCode::NoChange
    );

    ctx.accounts.state.withdrawal_delay = new_delay;

    emit!(WithdrawalDelayUpdated {
        version: 1,
        new_delay,
    });

    Ok(())
}
