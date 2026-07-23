use anchor_lang::prelude::*;

use crate::constants::*;
use crate::error::ErrorCode;
use crate::events::MinimumUnstakeAmountUpdated;
use crate::state::*;

/// Update minimum unstake amount instruction accounts
#[derive(Accounts)]
pub struct UpdateMinimumUnstakeAmount<'info> {
    /// Liquid staking state account
    #[account(
        mut,
        seeds = [STATE_SEED],
        bump = state.state_bump,
        has_one = operations_authority,
    )]
    pub state: Account<'info, ProgramState>,

    /// Operations authority that can update minimum unstake amount
    pub operations_authority: Signer<'info>,
}

/// Update the minimum amount that can be unstaked
pub fn update_minimum_unstake_amount_handler(
    ctx: Context<UpdateMinimumUnstakeAmount>,
    new_minimum_amount: u64,
) -> Result<()> {
    let state = &mut ctx.accounts.state;

    require!(
        new_minimum_amount > 0 && new_minimum_amount != state.minimum_unstake_amount,
        ErrorCode::InvalidMinimumAmount
    );

    // Update the minimum unstake amount
    state.minimum_unstake_amount = new_minimum_amount;

    emit!(MinimumUnstakeAmountUpdated { version: 1, new_minimum_amount });

    Ok(())
}
