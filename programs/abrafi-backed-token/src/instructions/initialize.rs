use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token};

use crate::constants::*;
use crate::error::ErrorCode;
use crate::events::*;
use crate::state::*;
use crate::utils::*;

#[derive(Accounts)]
pub struct Initialize<'info> {
    /// Program state account (PDA)
    #[account(
        init,
        payer = authority,
        seeds = [STATE_SEED],
        space = 8 + ProgramState::INIT_SPACE,
        bump
    )]
    pub state: Account<'info, ProgramState>,

    /// Authority that will control the program
    #[account(mut)]
    pub authority: Signer<'info>,

    /// Compliance authority for compliance-related operations
    #[account(
        constraint = compliance_authority.key() != Pubkey::default() && compliance_authority.key() != authority.key() @ ErrorCode::InvalidConfiguration,
    )]
    pub compliance_authority: SystemAccount<'info>,

    /// Operations authority for managing specific operations
    #[account(
        constraint = operations_authority.key() != Pubkey::default() && operations_authority.key() != authority.key() @ ErrorCode::InvalidConfiguration,
    )]
    pub operations_authority: SystemAccount<'info>,

    /// Vault authority PDA that controls all withdrawal vaults
    /// CHECK: Safe because we derive it with PDA and just store it
    #[account(
        seeds = [VAULT_AUTHORITY_SEED, state.key().as_ref()],
        bump
    )]
    pub vault_authority: UncheckedAccount<'info>,

    /// The abrafi token mint to be created
    #[account(
        init,
        payer = authority,
        mint::decimals = 6,
        mint::authority = state,
        mint::freeze_authority = state,
    )]
    pub abrafi_backed_token_mint: Account<'info, Mint>,

    /// Token program for mint creation
    pub token_program: Program<'info, Token>,

    /// System program for account creation
    pub system_program: Program<'info, System>,
}

/// Initialize the abrafi token program
pub fn initialize_handler(ctx: Context<Initialize>) -> Result<()> {
    let state = &mut ctx.accounts.state;
    let abrafi_backed_token_decimals = ctx.accounts.abrafi_backed_token_mint.decimals;

    // Set state fields
    state.version = 1;
    state.authority = ctx.accounts.authority.key();
    state.pending_authority = Pubkey::default(); // No pending authority initially
    state.pending_authority_expiration_timestamp = 0; // No pending authority expiration initially
    state.pending_mint_authority = Pubkey::default(); // No pending mint authority initially
    state.pending_mint_authority_expiration_timestamp = 0; // No pending mint authority expiration initially
    state.abrafi_backed_token_mint = ctx.accounts.abrafi_backed_token_mint.key();
    state.collateral_tokens = Vec::new();
    state.is_minting_enabled = true;
    state.is_unminting_request_enabled = true;
    state.is_unminting_claim_enabled = true;
    state.is_mint_whitelist_enabled = true;
    state.minimum_mint_amount = calculate_minimum_amount_from_decimals(abrafi_backed_token_decimals, ErrorCode::CalculationOverflow)?; // 1 abrafi token
    state.minimum_unmint_amount = calculate_minimum_amount_from_decimals(abrafi_backed_token_decimals, ErrorCode::CalculationOverflow)?; // 1 abrafi token
    state.request_expiration_seconds = DEFAULT_REQUEST_EXPIRATION_SECONDS;
    state.unmint_cooldown_seconds = DEFAULT_UNMINT_COOLDOWN_SECONDS;

    state.state_bump = ctx.bumps.state;
    state.vault_authority_bump = ctx.bumps.vault_authority;

    state.compliance_authority = ctx.accounts.compliance_authority.key();
    state.operations_authority = ctx.accounts.operations_authority.key();
    state._unused = Pubkey::default(); // Kept for backward compatibility

    emit!(ProgramInitialized {
        version: 1,
        authority: state.authority,
        compliance_authority: state.compliance_authority,
        operations_authority: state.operations_authority,
        abrafi_backed_token_mint: state.abrafi_backed_token_mint,
    });

    Ok(())
}
