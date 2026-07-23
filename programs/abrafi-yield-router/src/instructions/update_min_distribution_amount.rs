use anchor_lang::prelude::*;

use crate::constants::STATE_SEED;
use crate::error::ErrorCode;
use crate::events::MinDistributionAmountUpdated;
use crate::state::RouterState;

#[derive(Accounts)]
pub struct UpdateMinDistributionAmount<'info> {
    #[account(mut, seeds = [STATE_SEED], bump = state.bump, has_one = operations_authority)]
    pub state: Account<'info, RouterState>,

    pub operations_authority: Signer<'info>,
}

pub fn update_min_distribution_amount_handler(
    ctx: Context<UpdateMinDistributionAmount>,
    new_amount: u64,
) -> Result<()> {
    require!(new_amount > 0, ErrorCode::InvalidConfiguration);

    let old_amount = ctx.accounts.state.min_distribution_amount;
    require!(new_amount != old_amount, ErrorCode::InvalidConfiguration);

    ctx.accounts.state.min_distribution_amount = new_amount;

    emit!(MinDistributionAmountUpdated {
        version: 1,
        old_amount,
        new_amount,
    });

    Ok(())
}
