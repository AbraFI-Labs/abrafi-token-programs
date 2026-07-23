use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount};

use crate::constants::*;
use crate::error::ErrorCode;
use crate::events::AccountThawedEvent;
use crate::state::*;

/// Thaw account instruction accounts
#[derive(Accounts)]
pub struct ThawAccount<'info> {
    /// Liquid staking state account
    #[account(
        seeds = [STATE_SEED],
        bump = state.state_bump,
        has_one = compliance_authority,
        has_one = liquid_staking_token_mint,
    )]
    pub state: Account<'info, ProgramState>,

    /// Compliance authority that can thaw accounts
    pub compliance_authority: Signer<'info>,

    /// User's liquid staking token account to thaw
    /// CHECK: Any user can have their account thawed by the compliance authority
    pub user: UncheckedAccount<'info>,

    /// Token mint (required for thaw operations)
    pub liquid_staking_token_mint: Account<'info, Mint>,

    /// Token account to thaw
    #[account(
        mut,
        associated_token::authority = user,
        associated_token::mint = liquid_staking_token_mint,
        constraint = user_liquid_staking_token_account.is_frozen() @ ErrorCode::AccountNotFrozen,
    )]
    pub user_liquid_staking_token_account: Account<'info, TokenAccount>,

    /// Token program for thaw operations
    pub token_program: Program<'info, Token>,
}

/// Thaw a token account
/// This function can only be called by the compliance authority
pub fn thaw_account_handler(ctx: Context<ThawAccount>) -> Result<()> {
    // Thaw user token account
    token::thaw_account(CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        token::ThawAccount {
            account: ctx.accounts.user_liquid_staking_token_account.to_account_info(),
            mint: ctx.accounts.liquid_staking_token_mint.to_account_info(),
            authority: ctx.accounts.state.to_account_info(),
        },
        &[&[STATE_SEED, &[ctx.accounts.state.state_bump]]],
    ))?;

    emit!(AccountThawedEvent {
        version: 1,
        token_account: ctx.accounts.user_liquid_staking_token_account.key(),
        user: ctx.accounts.user_liquid_staking_token_account.owner,
    });

    Ok(())
}
