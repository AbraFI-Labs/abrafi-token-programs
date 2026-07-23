use anchor_lang::prelude::*;
use anchor_spl::token::Mint;

use crate::constants::*;
use crate::error::ErrorCode;
use crate::events::*;
use crate::state::*;

/// Transfer mint authority instruction accounts (step 1: set pending mint authority)
#[derive(Accounts)]
pub struct TransferMintAuthority<'info> {
    /// Liquid staking state account
    #[account(
        mut,
        seeds = [STATE_SEED],
        bump = state.state_bump,
        has_one = liquid_staking_token_mint,
        has_one = underlying_token_mint,
        has_one = authority,
    )]
    pub state: Account<'info, ProgramState>,

    /// Current authority that can transfer mint authority
    pub authority: Signer<'info>,

    /// The liquid staking token mint
    pub liquid_staking_token_mint: Account<'info, Mint>,

    /// The underlying token mint
    pub underlying_token_mint: Account<'info, Mint>,

    /// New mint authority to transfer to (Some for setting, None for cancellation)
    /// CHECK: Safe because we're just storing the pubkey for the transfer
    pub new_mint_authority: Option<UncheckedAccount<'info>>,
}

/// Set pending mint authority (step 1: current authority initiates transfer)
/// This sets a pending mint authority that must be finalized by the new authority within 24 hours
/// Requires that staking and request unstaking are disabled
/// To cancel a pending mint authority transfer, pass None for new_mint_authority
pub fn transfer_mint_authority_handler(ctx: Context<TransferMintAuthority>) -> Result<()> {
    // Extract state key before creating mutable borrow to avoid borrow checker conflicts
    let state_key = ctx.accounts.state.key();
    let state = &mut ctx.accounts.state;
    let clock = Clock::get()?;

    // Validate that staking is disabled (required before transferring mint authority)
    require!(
        !state.is_staking_enabled,
        ErrorCode::InvalidConfiguration
    );

    // Validate that request unstaking is disabled (required before transferring mint authority)
    require!(
        !state.is_unstaking_request_enabled,
        ErrorCode::InvalidConfiguration
    );

    // Check if this is a cancellation request (None = cancel)
    if let None = ctx.accounts.new_mint_authority {
        // Only allow cancellation if there's a pending mint authority to cancel
        require!(
            state.pending_mint_authority != Pubkey::default(),
            ErrorCode::InvalidConfiguration
        );

        // Store the cancelled pending mint authority for the event
        let cancelled_pending_mint_authority = state.pending_mint_authority;

        // Clear the pending mint authority and expiration timestamp
        state.pending_mint_authority = Pubkey::default();
        state.pending_mint_authority_expiration_timestamp = 0;

        emit!(MintAuthorityTransferCancelled {
            version: 1,
            current_authority: state.authority,
            cancelled_pending_mint_authority,
            liquid_staking_token_mint: ctx.accounts.liquid_staking_token_mint.key(),
        });

        return Ok(());
    }

    // Normal validation for setting a new pending mint authority
    let new_mint_authority = ctx.accounts.new_mint_authority.as_ref().unwrap().key();

    // Validate that the new mint authority is not the default pubkey
    require!(
        new_mint_authority != Pubkey::default(),
        ErrorCode::InvalidConfiguration
    );

    // Validate that the new mint authority is not the state PDA (prevents circular authority)
    require!(
        new_mint_authority != state_key,
        ErrorCode::InvalidConfiguration
    );

    // Calculate and store expiration timestamp
    let expiration_timestamp = clock
        .unix_timestamp
        .checked_add(PENDING_AUTHORITY_EXPIRATION_SECONDS)
        .ok_or(ErrorCode::CalculationOverflow)?;

    // Set the pending mint authority and expiration timestamp
    state.pending_mint_authority = new_mint_authority;
    state.pending_mint_authority_expiration_timestamp = expiration_timestamp;

    emit!(MintAuthorityTransferPending {
        version: 1,
        current_authority: state.authority,
        pending_mint_authority: new_mint_authority,
        expiration_timestamp,
        liquid_staking_token_mint: ctx.accounts.liquid_staking_token_mint.key(),
    });

    Ok(())
}
