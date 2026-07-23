use anchor_lang::prelude::*;

use crate::constants::STATE_SEED;
use crate::error::ErrorCode;
use crate::events::OperationsAuthorityUpdated;
use crate::state::RouterState;

#[derive(Accounts)]
pub struct UpdateOperationsAuthority<'info> {
    #[account(mut, seeds = [STATE_SEED], bump = state.bump, has_one = authority)]
    pub state: Account<'info, RouterState>,

    pub authority: Signer<'info>,

    #[account(
        constraint = new_operations_authority.key() != Pubkey::default()
            && new_operations_authority.key() != authority.key() @ ErrorCode::InvalidConfiguration,
    )]
    pub new_operations_authority: SystemAccount<'info>,
}

pub fn update_operations_authority_handler(
    ctx: Context<UpdateOperationsAuthority>,
) -> Result<()> {
    let old_operations_authority = ctx.accounts.state.operations_authority;
    let new_operations_authority = ctx.accounts.new_operations_authority.key();

    // Prevent assigning ops_authority to a pending_authority mid-transfer,
    // which would leave both tiers controlled by one key after finalize.
    let pending = ctx.accounts.state.pending_authority;
    require!(
        pending == Pubkey::default() || pending != new_operations_authority,
        ErrorCode::InvalidConfiguration
    );

    ctx.accounts.state.operations_authority = new_operations_authority;

    emit!(OperationsAuthorityUpdated {
        version: 1,
        old_operations_authority,
        new_operations_authority,
    });

    Ok(())
}
