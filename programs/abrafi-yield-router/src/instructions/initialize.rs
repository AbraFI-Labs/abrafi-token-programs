use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};

use crate::constants::STATE_SEED;
use crate::error::ErrorCode;
use crate::events::RouterInitialized;
use crate::state::RouterState;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
        seeds = [STATE_SEED],
        space = 8 + RouterState::INIT_SPACE,
        bump
    )]
    pub state: Account<'info, RouterState>,

    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        constraint = operations_authority.key() != Pubkey::default()
            && operations_authority.key() != authority.key() @ ErrorCode::AuthoritiesMustBeDifferent
    )]
    pub operations_authority: SystemAccount<'info>,

    pub yield_token_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = authority,
        associated_token::mint = yield_token_mint,
        associated_token::authority = state,
    )]
    pub router_vault: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn initialize_handler(
    ctx: Context<Initialize>,
    min_distribution_amount: u64,
) -> Result<()> {
    require!(
        min_distribution_amount > 0,
        ErrorCode::InvalidConfiguration
    );

    let state = &mut ctx.accounts.state;

    state.version = 1;
    state.bump = ctx.bumps.state;
    state.authority = ctx.accounts.authority.key();
    state.pending_authority = Pubkey::default();
    state.pending_authority_expiration_timestamp = 0;
    state.operations_authority = ctx.accounts.operations_authority.key();
    state.yield_token_mint = ctx.accounts.yield_token_mint.key();
    state.router_vault = ctx.accounts.router_vault.key();
    state.recipients = Vec::new();
    state.total_distributed = 0;
    state.distribute_enabled = true;
    state.min_distribution_amount = min_distribution_amount;

    emit!(RouterInitialized {
        version: 1,
        authority: state.authority,
        operations_authority: state.operations_authority,
        yield_token_mint: state.yield_token_mint,
        router_vault: state.router_vault,
        min_distribution_amount: state.min_distribution_amount,
    });

    Ok(())
}
