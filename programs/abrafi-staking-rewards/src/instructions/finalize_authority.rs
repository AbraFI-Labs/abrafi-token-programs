use anchor_lang::prelude::*;
use crate::constants::STATE_SEED;
use crate::error::ErrorCode;
use crate::events::AuthorityUpdated;
use crate::state::ProgramState;

#[derive(Accounts)]
pub struct FinalizeAuthority<'info> {
    #[account(
        mut,
        seeds = [STATE_SEED],
        bump = state.bump,
        constraint = state.pending_authority == new_authority.key() @ ErrorCode::InvalidConfiguration,
    )]
    pub state: Account<'info, ProgramState>,
    pub new_authority: Signer<'info>,
}

pub fn finalize_authority_handler(ctx: Context<FinalizeAuthority>) -> Result<()> {
    require!(
        ctx.accounts.state.pending_authority != Pubkey::default(),
        ErrorCode::NoPendingAuthorityTransfer
    );
    let now = Clock::get()?.unix_timestamp;
    require!(
        now < ctx.accounts.state.pending_authority_expiration_timestamp,
        ErrorCode::PendingAuthorityExpired
    );
    ctx.accounts.state.authority = ctx.accounts.new_authority.key();
    ctx.accounts.state.pending_authority = Pubkey::default();
    ctx.accounts.state.pending_authority_expiration_timestamp = 0;
    emit!(AuthorityUpdated { version: 1, new_authority: ctx.accounts.new_authority.key() });
    Ok(())
}
