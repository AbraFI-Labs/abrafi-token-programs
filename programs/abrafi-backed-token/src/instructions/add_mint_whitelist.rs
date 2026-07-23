use anchor_lang::prelude::*;

use crate::constants::*;
use crate::error::ErrorCode;
use crate::events::*;
use crate::state::*;

/// Add mint whitelist address instruction accounts
#[derive(Accounts)]
pub struct AddMintWhitelist<'info> {
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

    /// Address to add to the mint whitelist
    /// CHECK: Any address can be added to the mint whitelist
    pub address: UncheckedAccount<'info>,

    /// Mint whitelist entry account (will be created)
    #[account(
        init,
        payer = compliance_authority,
        space = 8 + MintWhitelistEntry::INIT_SPACE,
        seeds = [MINT_WHITELIST_SEED, address.key().as_ref()],
        bump
    )]
    pub mint_whitelist: Account<'info, MintWhitelistEntry>,

    /// System program for account creation
    pub system_program: Program<'info, System>,
}

/// Add an address to the mint whitelist
///
/// An optional custom unmint cooldown can be configured at whitelisting time.
/// When provided, it activates 24 hours after this instruction is executed —
/// until then the global unmint_cooldown_seconds applies to this address.
///
/// This function can only be called by the compliance authority.
pub fn add_mint_whitelist_handler(
    ctx: Context<AddMintWhitelist>,
    custom_cooldown_seconds: Option<i64>,
) -> Result<()> {
    let clock = Clock::get()?;
    let mint_whitelist = &mut ctx.accounts.mint_whitelist;

    mint_whitelist.version = 1;
    mint_whitelist.added_timestamp = clock.unix_timestamp;

    if let Some(seconds) = custom_cooldown_seconds {
        require!(
            seconds >= 0 && seconds <= MAX_COOLDOWN_SECONDS,
            ErrorCode::InvalidConfiguration
        );
        mint_whitelist.has_custom_cooldown = true;
        mint_whitelist.custom_cooldown_seconds = seconds;
        mint_whitelist.custom_cooldown_effective_timestamp = clock
            .unix_timestamp
            .checked_add(CUSTOM_COOLDOWN_ACTIVATION_DELAY)
            .ok_or(ErrorCode::InvalidConfiguration)?;
    }

    emit!(MintWhitelistAdded {
        version: 1,
        user: ctx.accounts.address.key(),
        custom_cooldown_seconds: if mint_whitelist.has_custom_cooldown {
            Some(mint_whitelist.custom_cooldown_seconds)
        } else {
            None
        },
        custom_cooldown_effective_timestamp: if mint_whitelist.has_custom_cooldown {
            Some(mint_whitelist.custom_cooldown_effective_timestamp)
        } else {
            None
        },
    });

    Ok(())
}
