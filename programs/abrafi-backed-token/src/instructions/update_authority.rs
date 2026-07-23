use anchor_lang::prelude::*;

use crate::constants::*;
use crate::error::ErrorCode;
use crate::events::*;
use crate::state::*;

/// Update authority instruction accounts
#[derive(Accounts)]
pub struct UpdateAuthority<'info> {
    /// Program state account
    #[account(
        mut,
        seeds = [STATE_SEED],
        bump = state.state_bump,
        has_one = authority,
    )]
    pub state: Account<'info, ProgramState>,

    /// Current authority that can update program configuration
    pub authority: Signer<'info>,

    /// New authority to set (Some for setting, None for cancellation)
    pub new_authority: Option<SystemAccount<'info>>,
}

/// Update the program authority (step 1: current authority sets pending authority)
/// This sets a pending authority that must be finalized by the new authority within 24 hours
/// To cancel a pending authority update, pass None for new_authority
pub fn update_authority_handler(ctx: Context<UpdateAuthority>) -> Result<()> {
    let state = &mut ctx.accounts.state;
    let clock = Clock::get()?;

    // Check if this is a cancellation request (None = cancel)
    if let None = ctx.accounts.new_authority {
        // Only allow cancellation if there's a pending authority to cancel
        require!(
            state.pending_authority != Pubkey::default(),
            ErrorCode::InvalidConfiguration
        );

        // Store the cancelled pending authority for the event
        let cancelled_pending_authority = state.pending_authority;

        // Clear the pending authority and expiration timestamp
        state.pending_authority = Pubkey::default();
        state.pending_authority_expiration_timestamp = 0;

        emit!(AuthorityUpdateCancelled {
            version: 1,
            authority_type: AuthorityType::Program,
            current_authority: state.authority,
            cancelled_pending_authority,
        });

        return Ok(());
    }

    // Normal validation for setting a new pending authority
    let new_authority = ctx.accounts.new_authority.as_ref().unwrap().key();
    require!(
        new_authority != state.authority
            && new_authority != state.compliance_authority
            && new_authority != state.operations_authority,
        ErrorCode::InvalidConfiguration
    );

    // Calculate and store expiration timestamp (24 hours from now)
    let expiration_timestamp = clock
        .unix_timestamp
        .checked_add(PENDING_AUTHORITY_EXPIRATION_SECONDS)
        .ok_or(ErrorCode::CalculationOverflow)?;

    // Set the pending authority and expiration timestamp
    state.pending_authority = new_authority;
    state.pending_authority_expiration_timestamp = expiration_timestamp;

    emit!(AuthorityUpdatePending {
        version: 1,
        authority_type: AuthorityType::Program,
        current_authority: state.authority,
        pending_authority: new_authority,
        expiration_timestamp,
    });

    Ok(())
}

/// Update the compliance authority
pub fn update_compliance_authority_handler(ctx: Context<UpdateAuthority>) -> Result<()> {
    let state = &mut ctx.accounts.state;
    let new_compliance_authority = ctx.accounts.new_authority
        .as_ref()
        .ok_or(ErrorCode::InvalidConfiguration)?
        .key();

    require!(
        new_compliance_authority != Pubkey::default()
            && new_compliance_authority != state.compliance_authority
            && new_compliance_authority != state.authority,
        ErrorCode::InvalidConfiguration
    );

    state.compliance_authority = new_compliance_authority;

    emit!(AuthorityUpdated {
        version: 1,
        authority_type: AuthorityType::Compliance,
        new_authority: new_compliance_authority,
    });

    Ok(())
}

/// Update the operations authority
pub fn update_operations_authority_handler(ctx: Context<UpdateAuthority>) -> Result<()> {
    let state = &mut ctx.accounts.state;
    let new_operations_authority = ctx.accounts.new_authority
        .as_ref()
        .ok_or(ErrorCode::InvalidConfiguration)?
        .key();

    require!(
        new_operations_authority != Pubkey::default()
            && new_operations_authority != state.operations_authority
            && new_operations_authority != state.authority,
        ErrorCode::InvalidConfiguration
    );

    state.operations_authority = new_operations_authority;

    emit!(AuthorityUpdated {
        version: 1,
        authority_type: AuthorityType::Operations,
        new_authority: new_operations_authority,
    });

    Ok(())
}

/// Finalize authority instruction accounts (step 2: new authority accepts)
#[derive(Accounts)]
pub struct FinalizeAuthority<'info> {
    /// Program state account
    #[account(
        mut,
        seeds = [STATE_SEED],
        bump = state.state_bump,
        has_one = pending_authority,
    )]
    pub state: Account<'info, ProgramState>,

    /// Pending authority that must sign to accept the transfer
    pub pending_authority: Signer<'info>,
}

/// Finalize the program authority transfer (step 2: new authority accepts)
/// This completes the authority transfer by moving pending_authority to authority
/// Must be called within 24 hours of the pending authority being set
pub fn finalize_authority_handler(ctx: Context<FinalizeAuthority>) -> Result<()> {
    let state = &mut ctx.accounts.state;
    let pending_authority = ctx.accounts.pending_authority.key();
    let clock = Clock::get()?;

    // Verify that pending authority is set (has_one constraint already verifies it matches)
    require!(
        state.pending_authority != Pubkey::default(),
        ErrorCode::InvalidConfiguration
    );

    // Check that the pending authority hasn't expired
    require!(
        clock.unix_timestamp < state.pending_authority_expiration_timestamp,
        ErrorCode::PendingAuthorityExpired
    );

    // Complete the transfer by moving pending to active
    state.authority = state.pending_authority;
    state.pending_authority = Pubkey::default();
    state.pending_authority_expiration_timestamp = 0;

    emit!(AuthorityUpdated {
        version: 1,
        authority_type: AuthorityType::Program,
        new_authority: pending_authority,
    });

    Ok(())
}
