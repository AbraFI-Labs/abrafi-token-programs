use anchor_lang::prelude::*;
use anchor_lang::solana_program::program_option::COption;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::constants::*;
use crate::error::ErrorCode;
use crate::events::*;
use crate::state::*;
use crate::utils::*;

/// Add collateral token instruction accounts
#[derive(Accounts)]
pub struct AddCollateralToken<'info> {
    /// Program state account
    #[account(
        mut,
        seeds = [STATE_SEED],
        bump = state.state_bump,
        has_one = authority,
    )]
    pub state: Account<'info, ProgramState>,

    /// Authority that can update program configuration
    #[account(mut)]
    pub authority: Signer<'info>,

    /// Vault authority PDA that controls all withdrawal vaults
    /// CHECK: Safe because we derive it with PDA and just store it
    #[account(
        seeds = [VAULT_AUTHORITY_SEED, state.key().as_ref()],
        bump = state.vault_authority_bump,
    )]
    pub vault_authority: UncheckedAccount<'info>,

    /// Token mint for the new collateral token
    #[account(
        constraint = new_token_mint.mint_authority != COption::Some(state.key()) @ ErrorCode::InvalidMint,
    )]
    pub new_token_mint: Account<'info, Mint>,

    /// Treasury account for the new token
    #[account(
        constraint = treasury_token_account.mint == new_token_mint.key() @ ErrorCode::InvalidMintAccount,
    )]
    pub treasury_token_account: Account<'info, TokenAccount>,

    /// Vault account to initialize for the new token
    #[account(
        init_if_needed,
        payer = authority,
        associated_token::authority = vault_authority,
        associated_token::mint = new_token_mint,
    )]
    pub vault_token_account: Account<'info, TokenAccount>,

    /// Associated token program for account creation
    pub associated_token_program: Program<'info, AssociatedToken>,

    /// Token program for account creation
    pub token_program: Program<'info, Token>,

    /// System program for account creation
    pub system_program: Program<'info, System>,
}

/// Add a new collateral token type to the program
/// Can also be used to re-enable a disabled token and update its treasury/name
pub fn add_collateral_token_handler(ctx: Context<AddCollateralToken>, name: String) -> Result<()> {
    let state = &mut ctx.accounts.state;

    // Validate the name length
    let name = name.trim();
    require!(
        !name.is_empty() && name.len() <= MAX_TOKEN_NAME_LEN,
        ErrorCode::InvalidTokenName
    );

    // Check if token already exists
    let existing_token = find_token_config_mut(state, &ctx.accounts.new_token_mint.key());

    match existing_token {
        Some(token_config) => {
            // Token already exists
            if token_config.disabled {
                // Re-enable the token and update treasury/name
                token_config.disabled = false;
                token_config.treasury_token_account = ctx.accounts.treasury_token_account.key();
                token_config.name = name.to_string();
            } else {
                // Token is already enabled - cannot add again
                return Err(ErrorCode::TokenAlreadyConfigured.into());
            }
        }
        None => {
            // Token doesn't exist - add new token configuration
            // Check if we have reached the maximum number of collateral tokens
            require!(
                state.collateral_tokens.len() < MAX_COLLATERAL_TOKENS,
                ErrorCode::CapacityExceeded
            );

            // Add new token configuration (enabled by default)
            state.collateral_tokens.push(CollateralTokenConfig {
                version: 1,
                name: name.to_string(),
                token_mint: ctx.accounts.new_token_mint.key(),
                treasury_token_account: ctx.accounts.treasury_token_account.key(),
                disabled: false,
            });
        }
    }

    // Emit event after the token has been added or re-enabled
    emit!(CollateralTokenAdded {
        version: 1,
        name: name.to_string(),
        num_collateral_tokens: state.collateral_tokens.len() as u8,
        token_mint: ctx.accounts.new_token_mint.key(),
        treasury_token_account: ctx.accounts.treasury_token_account.key(),
        vault_token_account: ctx.accounts.vault_token_account.key(),
    });

    Ok(())
}
