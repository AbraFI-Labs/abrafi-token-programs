use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, SetAuthority, Token, TokenAccount};

use crate::constants::*;
use crate::error::ErrorCode;
use crate::events::MintAuthorityTransferred;
use crate::state::*;

/// Finalize mint authority instruction accounts (step 2: new mint authority accepts)
#[derive(Accounts)]
pub struct FinalizeMintAuthority<'info> {
    /// Liquid staking state account
    #[account(
        mut,
        seeds = [STATE_SEED],
        bump = state.state_bump,
        has_one = liquid_staking_token_mint,
        has_one = underlying_token_mint,
        has_one = pending_mint_authority,
    )]
    pub state: Account<'info, ProgramState>,

    /// The liquid staking token mint
    #[account(mut)]
    pub liquid_staking_token_mint: Account<'info, Mint>,

    /// The underlying token mint
    pub underlying_token_mint: Account<'info, Mint>,

    /// Vault account for storing underlying tokens (ownership will be transferred)
    #[account(
        mut,
        associated_token::authority = state,
        associated_token::mint = underlying_token_mint,
    )]
    pub vault_token_account: Account<'info, TokenAccount>,

    /// Pending mint authority that must sign to accept the transfer
    pub pending_mint_authority: Signer<'info>,

    /// Token program for set_authority instruction
    pub token_program: Program<'info, Token>,
}

/// Finalize the mint authority transfer (step 2: new mint authority accepts)
/// This completes the mint authority transfer by actually transferring the mint authority
/// Also transfers ownership of the vault token account to the new mint authority
/// Must be called before it expires
pub fn finalize_mint_authority_handler(ctx: Context<FinalizeMintAuthority>) -> Result<()> {
    let pending_mint_authority = ctx.accounts.pending_mint_authority.key();
    let clock = Clock::get()?;
    let vault_balance = ctx.accounts.vault_token_account.amount;

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
    let seeds = &[STATE_SEED, &[state_bump]];
    let signer = &[&seeds[..]];

    // Transfer ownership of the vault token account from state PDA to new mint authority
    token::set_authority(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            SetAuthority {
                current_authority: ctx.accounts.state.to_account_info(),
                account_or_mint: ctx.accounts.vault_token_account.to_account_info(),
            },
            signer,
        ),
        anchor_spl::token::spl_token::instruction::AuthorityType::AccountOwner,
        Some(pending_mint_authority),
    )?;

    // Transfer mint authority from state PDA to new authority
    token::set_authority(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            SetAuthority {
                current_authority: ctx.accounts.state.to_account_info(),
                account_or_mint: ctx.accounts.liquid_staking_token_mint.to_account_info(),
            },
            signer,
        ),
        anchor_spl::token::spl_token::instruction::AuthorityType::MintTokens,
        Some(pending_mint_authority),
    )?;

    // Clear the pending mint authority and expiration timestamp
    // Now we can safely create a mutable borrow after the CPI calls
    let state = &mut ctx.accounts.state;
    state.pending_mint_authority = Pubkey::default();
    state.pending_mint_authority_expiration_timestamp = 0;

    emit!(MintAuthorityTransferred {
        version: 1,
        new_mint_authority: pending_mint_authority,
        liquid_staking_token_mint: ctx.accounts.liquid_staking_token_mint.key(),
        vault_balance,
    });

    Ok(())
}
