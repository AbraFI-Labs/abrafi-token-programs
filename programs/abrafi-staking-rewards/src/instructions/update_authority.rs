use anchor_lang::prelude::*;
use crate::constants::{STATE_SEED, PENDING_AUTHORITY_EXPIRATION_SECONDS};
use crate::error::ErrorCode;
use crate::events::{AuthorityUpdateCancelled, AuthorityUpdatePending};
use crate::state::ProgramState;
use crate::utils::add_delay;

#[derive(Accounts)]
pub struct UpdateAuthority<'info> {
    #[account(mut, seeds = [STATE_SEED], bump = state.bump, has_one = authority)]
    pub state: Account<'info, ProgramState>,
    pub authority: Signer<'info>,
    pub new_authority: Option<SystemAccount<'info>>,
}

pub fn update_authority_handler(ctx: Context<UpdateAuthority>) -> Result<()> {
    let state = &mut ctx.accounts.state;

    if ctx.accounts.new_authority.is_none() {
        require!(
            state.pending_authority != Pubkey::default(),
            ErrorCode::NoPendingAuthorityTransfer
        );
        let cancelled = state.pending_authority;
        state.pending_authority = Pubkey::default();
        state.pending_authority_expiration_timestamp = 0;
        emit!(AuthorityUpdateCancelled {
            version: 1,
            current_authority: ctx.accounts.authority.key(),
            cancelled_pending_authority: cancelled,
        });
        return Ok(());
    }

    let new_key = ctx.accounts.new_authority.as_ref().unwrap().key();
    require!(
        new_key != Pubkey::default()
            && new_key != state.authority
            && new_key != state.operations_authority,
        ErrorCode::InvalidConfiguration
    );
    let now = Clock::get()?.unix_timestamp;
    let expiration = add_delay(now, PENDING_AUTHORITY_EXPIRATION_SECONDS, ErrorCode::CalculationOverflow)?;
    // Overwrites any in-flight pending transfer; the previous pending_authority can no longer finalize.
    state.pending_authority = new_key;
    state.pending_authority_expiration_timestamp = expiration;
    emit!(AuthorityUpdatePending {
        version: 1,
        current_authority: ctx.accounts.authority.key(),
        pending_authority: new_key,
        expiration_timestamp: expiration,
    });
    Ok(())
}
