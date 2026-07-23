use anchor_lang::prelude::*;

use crate::constants::STATE_SEED;
use crate::error::ErrorCode;
use crate::events::AuthorityUpdated;
use crate::state::RouterState;

#[derive(Accounts)]
pub struct FinalizeAuthority<'info> {
    #[account(mut, seeds = [STATE_SEED], bump = state.bump)]
    pub state: Account<'info, RouterState>,

    pub new_authority: Signer<'info>,
}

pub fn finalize_authority_handler(ctx: Context<FinalizeAuthority>) -> Result<()> {
    // Check no-pending first so clients see NoPendingAuthorityTransfer, not InvalidAuthority.
    require!(
        ctx.accounts.state.pending_authority != Pubkey::default(),
        ErrorCode::NoPendingAuthorityTransfer
    );
    require!(
        ctx.accounts.state.pending_authority == ctx.accounts.new_authority.key(),
        ErrorCode::InvalidAuthority
    );

    let now = Clock::get()?.unix_timestamp;
    require!(
        now < ctx.accounts.state.pending_authority_expiration_timestamp,
        ErrorCode::PendingAuthorityExpired
    );

    let old_authority = ctx.accounts.state.authority;
    let new_authority = ctx.accounts.new_authority.key();

    ctx.accounts.state.authority = new_authority;
    ctx.accounts.state.pending_authority = Pubkey::default();
    ctx.accounts.state.pending_authority_expiration_timestamp = 0;

    emit!(AuthorityUpdated {
        version: 1,
        old_authority,
        new_authority,
    });

    Ok(())
}
