use anchor_lang::prelude::*;

use crate::constants::*;
use crate::error::ErrorCode;
use crate::events::*;
use crate::state::*;

/// Update minimum unmint amount instruction accounts
#[derive(Accounts)]
pub struct UpdateMinimumUnmintAmount<'info> {
    /// Program state account
    #[account(
        mut,
        seeds = [STATE_SEED],
        bump = state.state_bump,
        has_one = operations_authority,
    )]
    pub state: Account<'info, ProgramState>,

    /// Operations authority that can update minimum unmint amount
    pub operations_authority: Signer<'info>,
}

/// Update the minimum amount that can be unminted
pub fn update_minimum_unmint_amount_handler(
    ctx: Context<UpdateMinimumUnmintAmount>,
    new_minimum_amount: u64,
) -> Result<()> {
    let state = &mut ctx.accounts.state;

    require!(
        new_minimum_amount > 0 && new_minimum_amount != state.minimum_unmint_amount,
        ErrorCode::InvalidMinimumAmount
    );

    // Update the minimum unmint amount
    state.minimum_unmint_amount = new_minimum_amount;

    emit!(MinimumUnmintAmountUpdated {
        version: 1,
        new_minimum_amount,
    });

    Ok(())
}
