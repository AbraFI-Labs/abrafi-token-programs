use anchor_lang::prelude::*;

use crate::constants::*;
use crate::error::ErrorCode;
use crate::events::MinimumStakeAmountUpdated;
use crate::state::*;

/// Update the minimum stake amount
#[derive(Accounts)]
pub struct UpdateMinimumStakeAmount<'info> {
    /// Liquid staking state account
    #[account(
        mut,
        seeds = [STATE_SEED],
        bump = state.state_bump,
        has_one = operations_authority,
    )]
    pub state: Account<'info, ProgramState>,

    /// Operations authority that can update minimum stake amount
    pub operations_authority: Signer<'info>,
}

/// Update the minimum amount that can be staked
pub fn update_minimum_stake_amount_handler(
    ctx: Context<UpdateMinimumStakeAmount>,
    new_minimum_amount: u64,
) -> Result<()> {
    let state = &mut ctx.accounts.state;

    require!(
        new_minimum_amount > 0 && new_minimum_amount != state.minimum_stake_amount,
        ErrorCode::InvalidMinimumAmount
    );

    // Update minimum stake amount
    state.minimum_stake_amount = new_minimum_amount;

    emit!(MinimumStakeAmountUpdated { version: 1, new_minimum_amount });

    Ok(())
}
