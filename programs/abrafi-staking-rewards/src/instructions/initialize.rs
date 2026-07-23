use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};

use crate::constants::*;
use crate::error::ErrorCode;
use crate::events::ProgramInitialized;
use crate::state::ProgramState;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
        seeds = [STATE_SEED],
        space = 8 + ProgramState::INIT_SPACE,
        bump
    )]
    pub state: Account<'info, ProgramState>,

    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        constraint = operations_authority.key() != Pubkey::default()
            && operations_authority.key() != authority.key() @ ErrorCode::InvalidConfiguration,
    )]
    pub operations_authority: SystemAccount<'info>,

    pub stake_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = authority,
        associated_token::mint = stake_mint,
        associated_token::authority = state,
    )]
    pub staking_vault: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn initialize_handler(ctx: Context<Initialize>) -> Result<()> {
    let state = &mut ctx.accounts.state;

    state.version = 1;
    state.bump = ctx.bumps.state;
    state.authority = ctx.accounts.authority.key();
    state.pending_authority = Pubkey::default();
    state.pending_authority_expiration_timestamp = 0;
    state.operations_authority = ctx.accounts.operations_authority.key();
    state.stake_mint = ctx.accounts.stake_mint.key();
    state.staking_vault = ctx.accounts.staking_vault.key();
    state.total_staked = 0;
    state.total_yield_allocated = 0;
    state.total_pending_unstake = 0;
    state.global_reward_index = 0;
    state.withdrawal_delay = DEFAULT_WITHDRAWAL_DELAY;
    state.stake_enabled = true;
    state.unstake_request_enabled = true;
    state.unstake_claim_enabled = true;
    state.claim_yield_enabled = true;

    emit!(ProgramInitialized {
        version: 1,
        authority: state.authority,
        operations_authority: state.operations_authority,
        stake_mint: state.stake_mint,
        staking_vault: state.staking_vault,
    });

    Ok(())
}
