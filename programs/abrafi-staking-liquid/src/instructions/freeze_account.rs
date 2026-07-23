use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount};

use crate::constants::*;
use crate::error::ErrorCode;
use crate::events::AccountFrozenEvent;
use crate::state::*;

/// Freeze account instruction accounts
#[derive(Accounts)]
pub struct FreezeAccount<'info> {
    /// Liquid staking state account
    #[account(
        seeds = [STATE_SEED],
        bump = state.state_bump,
        has_one = compliance_authority,
        has_one = liquid_staking_token_mint,
    )]
    pub state: Account<'info, ProgramState>,

    /// Compliance authority that can freeze accounts
    pub compliance_authority: Signer<'info>,

    /// User's liquid staking token account to freeze
    /// CHECK: Any user can have their account frozen by the compliance authority
    pub user: UncheckedAccount<'info>,

    /// Token mint (required for freeze operations)
    pub liquid_staking_token_mint: Account<'info, Mint>,

    /// Token account to freeze
    #[account(
        mut,
        associated_token::authority = user,
        associated_token::mint = liquid_staking_token_mint,
        constraint = !user_liquid_staking_token_account.is_frozen() @ ErrorCode::AccountFrozen,
    )]
    pub user_liquid_staking_token_account: Account<'info, TokenAccount>,

    /// Token program for freeze operations
    pub token_program: Program<'info, Token>,
}

/// Freeze a token account
/// This function can only be called by the compliance authority
pub fn freeze_account_handler(ctx: Context<FreezeAccount>, reason_code: u32) -> Result<()> {
    // Freeze user token account
    token::freeze_account(CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        token::FreezeAccount {
            account: ctx.accounts.user_liquid_staking_token_account.to_account_info(),
            mint: ctx.accounts.liquid_staking_token_mint.to_account_info(),
            authority: ctx.accounts.state.to_account_info(),
        },
        &[&[STATE_SEED, &[ctx.accounts.state.state_bump]]],
    ))?;

    emit!(AccountFrozenEvent {
        version: 1,
        reason_code,
        token_account: ctx.accounts.user_liquid_staking_token_account.key(),
        user: ctx.accounts.user_liquid_staking_token_account.owner,
    });

    Ok(())
}
