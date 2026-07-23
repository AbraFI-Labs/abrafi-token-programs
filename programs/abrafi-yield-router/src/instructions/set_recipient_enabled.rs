use anchor_lang::prelude::*;

use crate::constants::STATE_SEED;
use crate::error::ErrorCode;
use crate::events::RecipientEnabledUpdated;
use crate::state::RouterState;

#[derive(Accounts)]
pub struct SetRecipientEnabled<'info> {
    #[account(mut, seeds = [STATE_SEED], bump = state.bump, has_one = operations_authority)]
    pub state: Account<'info, RouterState>,

    pub operations_authority: Signer<'info>,
}

pub fn set_recipient_enabled_handler(
    ctx: Context<SetRecipientEnabled>,
    index: u8,
    enabled: bool,
) -> Result<()> {
    let idx = index as usize;
    require!(
        idx < ctx.accounts.state.recipients.len(),
        ErrorCode::InvalidRecipientIndex
    );
    require!(
        enabled != ctx.accounts.state.recipients[idx].enabled,
        ErrorCode::InvalidConfiguration
    );

    ctx.accounts.state.recipients[idx].enabled = enabled;

    emit!(RecipientEnabledUpdated {
        version: 1,
        index,
        enabled,
    });

    Ok(())
}
