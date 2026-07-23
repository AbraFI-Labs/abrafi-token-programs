use anchor_lang::prelude::*;

use crate::constants::*;
use crate::error::ErrorCode;
use crate::events::*;
use crate::state::*;

/// Update minimum mint amount instruction accounts
#[derive(Accounts)]
pub struct UpdateMinimumMintAmount<'info> {
    /// Program state account
    #[account(
        mut,
        seeds = [STATE_SEED],
        bump = state.state_bump,
        has_one = operations_authority,
    )]
    pub state: Account<'info, ProgramState>,

    /// Operations authority that can update minimum mint amount
    pub operations_authority: Signer<'info>,
}

/// Update the minimum amount that can be minted
pub fn update_minimum_mint_amount_handler(
    ctx: Context<UpdateMinimumMintAmount>,
    new_minimum_amount: u64,
) -> Result<()> {
    let state = &mut ctx.accounts.state;

    require!(
        new_minimum_amount > 0 && new_minimum_amount != state.minimum_mint_amount,
        ErrorCode::InvalidMinimumAmount
    );

    // Update the minimum mint amount
    state.minimum_mint_amount = new_minimum_amount;

    emit!(MinimumMintAmountUpdated {
        version: 1,
        new_minimum_amount,
    });

    Ok(())
}
