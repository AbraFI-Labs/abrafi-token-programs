use anchor_lang::prelude::*;

use crate::constants::*;
use crate::error::ErrorCode;
use crate::events::WithdrawalDelayUpdated;
use crate::state::*;

/// Update withdrawal delay instruction accounts
#[derive(Accounts)]
pub struct UpdateWithdrawalDelay<'info> {
    /// Liquid staking state account
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

/// Update the withdrawal delay
/// Only the authority can change the withdrawal delay
pub fn update_withdrawal_delay_handler(
    ctx: Context<UpdateWithdrawalDelay>,
    new_delay_seconds: i64,
) -> Result<()> {
    let state = &mut ctx.accounts.state;

    require!(
        new_delay_seconds >= 0
            && new_delay_seconds <= MAX_WITHDRAWAL_DELAY_SECONDS
            && new_delay_seconds != state.withdrawal_delay_seconds,
        ErrorCode::InvalidConfiguration
    );

    // Update the withdrawal delay
    state.withdrawal_delay_seconds = new_delay_seconds;

    emit!(WithdrawalDelayUpdated {
        version: 1,
        new_withdrawal_delay_seconds: new_delay_seconds,
    });

    Ok(())
}
