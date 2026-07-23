use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount};

use crate::constants::*;
use crate::error::ErrorCode;
use crate::events::*;
use crate::state::*;

/// Remove collateral token instruction accounts
#[derive(Accounts)]
pub struct RemoveCollateralToken<'info> {
    /// Program state account
    #[account(
        mut,
        seeds = [STATE_SEED],
        bump = state.state_bump,
        has_one = authority,
    )]
    pub state: Account<'info, ProgramState>,

    /// Authority that can update program configuration
    pub authority: Signer<'info>,

    /// Vault authority PDA that controls all withdrawal vaults
    /// CHECK: Safe because we derive it with PDA and just store it
    #[account(
        seeds = [VAULT_AUTHORITY_SEED, state.key().as_ref()],
        bump = state.vault_authority_bump,
    )]
    pub vault_authority: UncheckedAccount<'info>,

    /// The token mint to remove
    /// Validated against the data structure to ensure it's configured
    pub token_mint_to_remove: Account<'info, Mint>,

    /// Withdrawal vault account for the specific token
    /// Must be empty before the token can be removed
    #[account(
        associated_token::mint = token_mint_to_remove,
        associated_token::authority = vault_authority,
    )]
    pub vault_token_account: Account<'info, TokenAccount>,
}

/// Remove a collateral token type from the program
/// Requires that the withdrawal vault is empty to prevent fund stranding
pub fn remove_collateral_token_handler(
    ctx: Context<RemoveCollateralToken>,
) -> Result<()> {
    let state = &mut ctx.accounts.state;
    let token_mint_key = ctx.accounts.token_mint_to_remove.key();

    // Check that withdrawal vault is empty before removal
    require!(
        ctx.accounts.vault_token_account.amount == 0,
        ErrorCode::VaultNotEmpty
    );

    // Remove token configuration - validates it exists in the data structure
    let (removed, retained): (Vec<CollateralTokenConfig>, Vec<CollateralTokenConfig>) = state
        .collateral_tokens
        .drain(..)
        .partition(|config| config.token_mint == token_mint_key);

    // Check if token is configured as collateral
    require!(removed.len() == 1, ErrorCode::TokenNotConfigured);

    state.collateral_tokens = retained;

    emit!(CollateralTokenRemoved {
        version: 1,
        name: removed[0].name.clone(),
        num_collateral_tokens: state.collateral_tokens.len() as u8,
        token_mint: token_mint_key,
        treasury_token_account: removed[0].treasury_token_account,
    });

    Ok(())
}
