use anchor_lang::prelude::*;

use crate::constants::STATE_SEED;
use crate::error::ErrorCode;
use crate::events::OperationsAuthorityUpdated;
use crate::state::ProgramState;

#[derive(Accounts)]
pub struct UpdateOperationsAuthority<'info> {
    #[account(mut, seeds = [STATE_SEED], bump = state.bump, has_one = authority)]
    pub state: Account<'info, ProgramState>,
    pub authority: Signer<'info>,
    #[account(
        constraint = new_operations_authority.key() != Pubkey::default()
            && new_operations_authority.key() != authority.key() @ ErrorCode::InvalidConfiguration,
    )]
    pub new_operations_authority: SystemAccount<'info>,
}

pub fn update_operations_authority_handler(ctx: Context<UpdateOperationsAuthority>) -> Result<()> {
    ctx.accounts.state.operations_authority = ctx.accounts.new_operations_authority.key();
    emit!(OperationsAuthorityUpdated {
        version: 1,
        new_authority: ctx.accounts.new_operations_authority.key(),
    });
    Ok(())
}
