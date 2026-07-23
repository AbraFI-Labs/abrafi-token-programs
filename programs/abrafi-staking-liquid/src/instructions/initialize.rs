use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::constants::*;
use crate::error::ErrorCode;
use crate::events::ProgramInitialized;
use crate::state::*;
use crate::utils::*;

/// Initialize the liquid staking program
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
        constraint = compliance_authority.key() != Pubkey::default()
            && compliance_authority.key() != authority.key()
            @ ErrorCode::InvalidConfiguration,
    )]
    pub compliance_authority: SystemAccount<'info>,

    /// Operations authority for managing specific operations
    #[account(
        constraint = operations_authority.key() != Pubkey::default()
            && operations_authority.key() != authority.key()
            @ ErrorCode::InvalidConfiguration,
    )]
    pub operations_authority: SystemAccount<'info>,

    /// The liquid staking token mint to be created
    #[account(
        init,
        payer = authority,
        mint::decimals = 6,
        mint::authority = state,
        mint::freeze_authority = state,
    )]
    pub liquid_staking_token_mint: Account<'info, Mint>,

    /// The underlying token mint (passed as parameter)
    pub underlying_token_mint: Account<'info, Mint>,

    /// Vault account for storing underlying tokens (will be created if it doesn't exist)
    #[account(
        init_if_needed,
        payer = authority,
        associated_token::authority = state,
        associated_token::mint = underlying_token_mint,
    )]
    pub vault_token_account: Account<'info, TokenAccount>,

    /// Associated token program for account creation
    pub associated_token_program: Program<'info, AssociatedToken>,

    /// Token program for mint creation
    pub token_program: Program<'info, Token>,

    /// System program for account creation
    pub system_program: Program<'info, System>,
}

/// Initialize the liquid staking program
/// Creates the liquid staking token mint and sets up the vault
pub fn initialize_handler(ctx: Context<Initialize>) -> Result<()> {
    let state = &mut ctx.accounts.state;
    let underlying_decimals = ctx.accounts.underlying_token_mint.decimals;

    state.version = 1;
    state.authority = ctx.accounts.authority.key();
    state.pending_authority = Pubkey::default();
    state.pending_authority_expiration_timestamp = 0;
    state.pending_mint_authority = Pubkey::default();
    state.pending_mint_authority_expiration_timestamp = 0;
    state.operations_authority = ctx.accounts.operations_authority.key();
    state.compliance_authority = ctx.accounts.compliance_authority.key();

    state.underlying_token_mint = ctx.accounts.underlying_token_mint.key();
    state.liquid_staking_token_mint = ctx.accounts.liquid_staking_token_mint.key();

    state.state_bump = ctx.bumps.state;

    state.is_staking_enabled = true;
    state.is_unstaking_request_enabled = true;
    state.is_unstaking_claim_enabled = true;
    state.minimum_stake_amount = calculate_minimum_amount_from_decimals(underlying_decimals, ErrorCode::CalculationOverflow)?;
    state.minimum_unstake_amount = calculate_minimum_amount_from_decimals(underlying_decimals, ErrorCode::CalculationOverflow)?;
    state.max_removal_percentage = DEFAULT_MAX_REMOVAL_PERCENTAGE;

    state.external_treasury_token_account = Pubkey::default();
    state.treasury_cooldown_end_timestamp = 0;
    state.remove_yield_cooldown_end_timestamp = 0;

    state.withdrawal_delay_seconds = DEFAULT_WITHDRAWAL_DELAY_SECONDS;

    emit!(ProgramInitialized {
        version: 1,
        authority: state.authority,
        compliance_authority: state.compliance_authority,
        operations_authority: state.operations_authority,
        liquid_staking_token_mint: state.liquid_staking_token_mint,
        underlying_token_mint: state.underlying_token_mint,
    });

    Ok(())
}
