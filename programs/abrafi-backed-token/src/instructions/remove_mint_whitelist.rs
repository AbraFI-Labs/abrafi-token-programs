use anchor_lang::prelude::*;

use crate::constants::*;
use crate::events::*;
use crate::state::*;

/// Remove mint whitelist address instruction accounts
#[derive(Accounts)]
pub struct RemoveMintWhitelist<'info> {
    /// Program state account
    #[account(
        seeds = [STATE_SEED],
        bump = state.state_bump,
        has_one = compliance_authority,
    )]
    pub state: Account<'info, ProgramState>,

    /// Compliance authority that can manage the mint whitelist
    #[account(mut)]
    pub compliance_authority: Signer<'info>,

    /// Address to remove from the mint whitelist
    /// CHECK: Any address can be removed from the mint whitelist
    pub address: UncheckedAccount<'info>,

    /// Mint whitelist entry account (will be closed)
    #[account(
        mut,
        close = compliance_authority,
        seeds = [MINT_WHITELIST_SEED, address.key().as_ref()],
        bump
    )]
    pub mint_whitelist: Account<'info, MintWhitelistEntry>,
}

/// Remove an address from the mint whitelist
/// This function can only be called by the compliance authority
/// The whitelist controls which addresses can use the mint and unmint operations
pub fn remove_mint_whitelist_handler(ctx: Context<RemoveMintWhitelist>) -> Result<()> {
    emit!(MintWhitelistRemoved {
        version: 1,
        user: ctx.accounts.address.key(),
    });

    Ok(())
}
