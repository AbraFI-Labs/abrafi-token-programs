use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, SetAuthority, Token};

use crate::constants::*;
use crate::error::ErrorCode;
use crate::events::MintAuthorityTransferred;
use crate::state::*;

/// Finalize mint authority instruction accounts (step 2: new mint authority accepts)
#[derive(Accounts)]
pub struct FinalizeMintAuthority<'info> {
    /// Program state account
    #[account(
        mut,
        seeds = [STATE_SEED],
        bump = state.state_bump,
        has_one = abrafi_backed_token_mint,
        has_one = pending_mint_authority,
    )]
    pub state: Account<'info, ProgramState>,

    /// The abrafi token mint
    #[account(mut)]
    pub abrafi_backed_token_mint: Account<'info, Mint>,

    /// Pending mint authority that must sign to accept the transfer
    pub pending_mint_authority: Signer<'info>,

    /// Token program for set_authority instruction
    pub token_program: Program<'info, Token>,
}

/// Finalize the mint authority transfer (step 2: new mint authority accepts)
/// This completes the mint authority transfer by actually transferring the mint authority
/// Must be called within 24 hours of the pending mint authority being set
pub fn finalize_mint_authority_handler(ctx: Context<FinalizeMintAuthority>) -> Result<()> {
    let pending_mint_authority = ctx.accounts.pending_mint_authority.key();
    let clock = Clock::get()?;

    // Verify that pending mint authority is set (has_one constraint already verifies it matches)
    require!(
        ctx.accounts.state.pending_mint_authority != Pubkey::default(),
        ErrorCode::InvalidConfiguration
    );

    // Check that the pending mint authority hasn't expired
    require!(
        clock.unix_timestamp < ctx.accounts.state.pending_mint_authority_expiration_timestamp,
        ErrorCode::PendingAuthorityExpired
    );

    // Extract state_bump before creating mutable borrow
    let state_bump = ctx.accounts.state.state_bump;
    // Transfer mint authority from state PDA to new authority
    let seeds = &[STATE_SEED, &[state_bump]];
    let signer = &[&seeds[..]];

    token::set_authority(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            SetAuthority {
                current_authority: ctx.accounts.state.to_account_info(),
                account_or_mint: ctx.accounts.abrafi_backed_token_mint.to_account_info(),
            },
            signer,
        ),
        anchor_spl::token::spl_token::instruction::AuthorityType::MintTokens,
        Some(pending_mint_authority),
    )?;

    // Clear the pending mint authority and expiration timestamp
    // Now we can safely create a mutable borrow after the CPI call
    let state = &mut ctx.accounts.state;
    state.pending_mint_authority = Pubkey::default();
    state.pending_mint_authority_expiration_timestamp = 0;

    emit!(MintAuthorityTransferred {
        version: 1,
        new_mint_authority: pending_mint_authority,
        abrafi_backed_token_mint: ctx.accounts.abrafi_backed_token_mint.key(),
    });

    Ok(())
}
