use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;

use crate::constants::*;
use crate::error::ErrorCode;
use crate::events::ExternalTreasuryUpdated;
use crate::state::*;
use crate::utils::safe_add_delay;

/// Set or update external treasury address (authority only)
#[derive(Accounts)]
pub struct SetExternalTreasury<'info> {
    /// Liquid staking state account
    #[account(
        mut,
        seeds = [STATE_SEED],
        bump = state.state_bump,
        has_one = authority,
    )]
    pub state: Account<'info, ProgramState>,

    /// Authority that can update treasury address
    pub authority: Signer<'info>,

    /// External treasury account to be set as the treasury
    pub external_treasury_token_account: Option<Account<'info, TokenAccount>>,
}

/// Set or update the external treasury address for underlying tokens (authority only)
pub fn set_external_treasury_handler(ctx: Context<SetExternalTreasury>) -> Result<()> {
    let state = &mut ctx.accounts.state;

    match &ctx.accounts.external_treasury_token_account {
        // None case - clear treasury
        None => {
            state.external_treasury_token_account = Pubkey::default();
            state.treasury_cooldown_end_timestamp = 0;
        }
        // Some case - validate and set new treasury
        Some(token_acc) => {
            require!(
                token_acc.key() != state.external_treasury_token_account,
                ErrorCode::InvalidTreasuryAccount
            );

            require!(
                token_acc.mint == state.underlying_token_mint,
                ErrorCode::InvalidMint
            );

            state.external_treasury_token_account = token_acc.key();
            // Calculate and store the cooldown end timestamp
            let current_timestamp = Clock::get()?.unix_timestamp;
            state.treasury_cooldown_end_timestamp = safe_add_delay(
                current_timestamp,
                TREASURY_UPDATE_COOLDOWN_SECONDS,
                ErrorCode::CalculationOverflow,
            )?;
        }
    }

    emit!(ExternalTreasuryUpdated {
        version: 1,
        authority: ctx.accounts.authority.key(),
        new_treasury: state.external_treasury_token_account,
    });

    Ok(())
}
