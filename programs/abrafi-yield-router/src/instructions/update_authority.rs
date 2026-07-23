use anchor_lang::prelude::*;

use crate::constants::{PENDING_AUTHORITY_EXPIRATION_SECONDS, STATE_SEED};
use crate::error::ErrorCode;
use crate::events::AuthorityUpdateProposed;
use crate::state::RouterState;

#[derive(Accounts)]
pub struct UpdateAuthority<'info> {
    #[account(mut, seeds = [STATE_SEED], bump = state.bump, has_one = authority)]
    pub state: Account<'info, RouterState>,

    pub authority: Signer<'info>,

    #[account(
        constraint = new_authority.key() != Pubkey::default() @ ErrorCode::AuthoritiesMustBeDifferent
    )]
    pub new_authority: SystemAccount<'info>,
}

pub fn update_authority_handler(ctx: Context<UpdateAuthority>) -> Result<()> {
    let new_key = ctx.accounts.new_authority.key();
    let state = &ctx.accounts.state;
    require!(
        new_key != state.authority && new_key != state.operations_authority,
        ErrorCode::AuthoritiesMustBeDifferent
    );

    let now = Clock::get()?.unix_timestamp;

    let expiration = now
        .checked_add(PENDING_AUTHORITY_EXPIRATION_SECONDS)
        .ok_or(ErrorCode::CalculationOverflow)?;

    ctx.accounts.state.pending_authority = new_key;
    ctx.accounts.state.pending_authority_expiration_timestamp = expiration;

    emit!(AuthorityUpdateProposed {
        version: 1,
        pending_authority: new_key,
        expiration_timestamp: expiration,
    });

    Ok(())
}
