use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, TransferChecked};

use crate::constants::*;
use crate::error::ErrorCode;
use crate::events::*;
use crate::state::*;
use crate::utils::*;

/// Withdraw from vault instruction accounts
#[derive(Accounts)]
pub struct WithdrawFromVault<'info> {
    /// Program state account
    #[account(
        seeds = [STATE_SEED],
        bump = state.state_bump,
        has_one = operations_authority,
    )]
    pub state: Account<'info, ProgramState>,

    /// Operations authority that can withdraw from withdrawal vault
    pub operations_authority: Signer<'info>,

    /// Vault authority PDA that controls all withdrawal vaults
    /// CHECK: Safe because we derive it with PDA and just store it
    #[account(
        seeds = [VAULT_AUTHORITY_SEED, state.key().as_ref()],
        bump = state.vault_authority_bump,
    )]
    pub vault_authority: UncheckedAccount<'info>,

    /// The token mint to withdraw from the vault
    pub withdraw_token_mint: Account<'info, Mint>,

    /// Withdrawal vault account for the specific token
    #[account(
        mut,
        associated_token::mint = withdraw_token_mint,
        associated_token::authority = vault_authority,
    )]
    pub vault_token_account: Account<'info, TokenAccount>,

    /// Treasury token account for the specific token
    /// Must match the configured treasury address for the token mint
    #[account(
        mut,
        constraint = treasury_token_account.mint == withdraw_token_mint.key() @ ErrorCode::InvalidMintAccount,
    )]
    pub treasury_token_account: Account<'info, TokenAccount>,

    /// Token program for transfers
    pub token_program: Program<'info, Token>,
}

/// Remove tokens from withdrawal vault back to treasury (operations authority only)
/// Allows the operations authority to withdraw excess funds from the withdrawal vault
/// Continues to work for disabled collateral tokens
pub fn withdraw_from_vault_handler(ctx: Context<WithdrawFromVault>, amount: u64) -> Result<()> {
    let state = &ctx.accounts.state;

    // Check for zero amount and if vault has enough tokens
    validate_sufficient_balance(
        amount,
        ctx.accounts.vault_token_account.amount,
        ErrorCode::InsufficientVaultBalance,
    )?;

    // The token configuration must exist (ok if disabled)
    let token_config = find_token_config(state, &ctx.accounts.withdraw_token_mint.key())
        .ok_or(ErrorCode::TokenNotConfigured)?;

    // Validate treasury account matches
    require!(
        ctx.accounts.treasury_token_account.key() == token_config.treasury_token_account,
        ErrorCode::InvalidTreasuryAccount
    );

    // Transfer tokens from withdrawal vault to treasury using vault authority PDA signing
    token::transfer_checked(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.vault_token_account.to_account_info(),
                mint: ctx.accounts.withdraw_token_mint.to_account_info(),
                to: ctx.accounts.treasury_token_account.to_account_info(),
                authority: ctx.accounts.vault_authority.to_account_info(),
            },
            &[&[
                VAULT_AUTHORITY_SEED,
                state.key().as_ref(),
                &[state.vault_authority_bump],
            ]],
        ),
        amount,
        ctx.accounts.withdraw_token_mint.decimals,
    )?;

    emit!(VaultWithdrawal {
        version: 1,
        name: token_config.name.clone(),
        amount: amount,
        withdraw_token_mint: ctx.accounts.withdraw_token_mint.key(),
    });

    Ok(())
}
