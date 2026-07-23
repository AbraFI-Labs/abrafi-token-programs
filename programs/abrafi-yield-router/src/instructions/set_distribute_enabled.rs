use anchor_lang::prelude::*;

use crate::constants::STATE_SEED;
use crate::error::ErrorCode;
use crate::events::DistributeEnabledUpdated;
use crate::state::RouterState;

#[derive(Accounts)]
pub struct SetDistributeEnabled<'info> {
    #[account(mut, seeds = [STATE_SEED], bump = state.bump, has_one = operations_authority)]
    pub state: Account<'info, RouterState>,

    pub operations_authority: Signer<'info>,
}

pub fn set_distribute_enabled_handler(
    ctx: Context<SetDistributeEnabled>,
    enabled: bool,
) -> Result<()> {
    require!(
        enabled != ctx.accounts.state.distribute_enabled,
        ErrorCode::InvalidConfiguration
    );
    ctx.accounts.state.distribute_enabled = enabled;

    emit!(DistributeEnabledUpdated { version: 1, enabled });

    Ok(())
}
