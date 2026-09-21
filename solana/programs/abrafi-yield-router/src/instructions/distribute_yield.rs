// SPDX-License-Identifier: BUSL-1.1
// Copyright (C) 2026 AbraFi Ltd., Business Source License 1.1.
// Change Date: 2nd anniversary of this version's publication.
// Change License: Apache-2.0. See LICENSE at repo root.

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, TransferChecked};

use crate::constants::STATE_SEED;
use crate::error::ErrorCode;
use crate::events::YieldDistributed;
use crate::state::{enabled_recipient_count, RecipientType, RouterState};
use abrafi_staking_rewards::cpi as staking_cpi;
use abrafi_staking_rewards::cpi::accounts::SettleYield as SettleYieldAccounts;

#[derive(Accounts)]
pub struct DistributeYield<'info> {
    #[account(mut, seeds = [STATE_SEED], bump = state.bump, has_one = operations_authority)]
    pub state: Account<'info, RouterState>,

    pub operations_authority: Signer<'info>,

    #[account(
        mut,
        constraint = router_vault.key() == state.router_vault @ ErrorCode::InvalidRecipientAccount
    )]
    pub router_vault: Account<'info, TokenAccount>,

    #[account(
        constraint = yield_token_mint.key() == state.yield_token_mint @ ErrorCode::InvalidRecipientAccount
    )]
    pub yield_token_mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,
    // remaining_accounts: per enabled recipient (in Vec order), slot count varies by type:
    //   StakingRewards:    [staking_state_pda (writable), staking_vault (writable), staking_program]
    //   LiquidStaking:     [vault (writable, == dest), dest (writable)]
    //   External:          [vault (writable, == dest), dest (writable)]
    // StakingRewards state PDA must be writable because settle_yield CPI updates global_reward_index.
    // Fixed 10-slot approach hits BPF stack limit — remaining_accounts is the correct Solana
    // pattern for variable-length account lists.
}

pub fn distribute_yield_handler<'a, 'b, 'c, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, DistributeYield<'info>>,
) -> Result<()> {
    require!(ctx.accounts.state.distribute_enabled, ErrorCode::DistributeDisabled);

    // Use full vault balance — caller funds vault then triggers distribution.
    let amount = ctx.accounts.router_vault.amount;
    require!(
        amount >= ctx.accounts.state.min_distribution_amount,
        ErrorCode::AmountBelowMinimum
    );

    // StakingRewards recipients need 3 accounts each; others need 2.
    let expected_accounts: usize = ctx.accounts.state.recipients.iter()
        .filter(|r| r.enabled)
        .map(|r| if r.recipient_type == RecipientType::StakingRewards { 3 } else { 2 })
        .sum();
    require!(
        ctx.remaining_accounts.len() == expected_accounts,
        ErrorCode::RecipientAccountMismatch
    );

    // Collect into owned Vec so remaining_accounts borrow is released before CPI borrows begin.
    let remaining: Vec<AccountInfo<'info>> = ctx.remaining_accounts.to_vec();
    let recipients = ctx.accounts.state.recipients.clone();
    let decimals = ctx.accounts.yield_token_mint.decimals;
    let yield_token_mint_key = ctx.accounts.yield_token_mint.key();
    let state_bump = ctx.accounts.state.bump;
    let seeds = &[STATE_SEED, &[state_bump]];
    let signer_seeds = &[&seeds[..]];

    // ── Phase 1: read balances and validate account addresses ─────────────────
    let mut balances: Vec<u64> = Vec::with_capacity(recipients.len());
    let mut account_offset: usize = 0;

    for recipient in recipients.iter() {
        if !recipient.enabled {
            balances.push(0);
            continue;
        }

        let slot_count = if recipient.recipient_type == RecipientType::StakingRewards { 3 } else { 2 };
        let balance_src = &remaining[account_offset];
        let dest_acct   = &remaining[account_offset + 1];
        account_offset += slot_count;

        require!(
            balance_src.key() == recipient.balance_source,
            ErrorCode::InvalidRecipientAccount
        );
        require!(
            dest_acct.key() == recipient.destination,
            ErrorCode::InvalidRecipientAccount
        );
        // StakingRewards balance_src (staking state PDA) must be writable because
        // settle_yield CPI updates global_reward_index on it.
        // LiquidStaking/External balance_src == destination, so is_writable is already ensured
        // by the dest check below. Non-StakingRewards types where balance_src != dest (none
        // currently) would be read-only — enforce that here.
        if balance_src.key() != dest_acct.key() && recipient.recipient_type != RecipientType::StakingRewards {
            require!(!balance_src.is_writable, ErrorCode::InvalidRecipientAccount);
        }
        require!(dest_acct.is_writable, ErrorCode::InvalidRecipientAccount);

        let balance = read_balance(balance_src, &dest_acct.key(), &recipient.recipient_type, &yield_token_mint_key, recipient.staking_program_id)?;
        balances.push(balance);
    }

    let total_balance: u128 = balances.iter().map(|&b| b as u128).sum();
    require!(total_balance > 0, ErrorCode::ZeroTotalBalance);

    // ── Phase 2: calculate proportional amounts and transfer ──────────────────
    let mut amounts_per_recipient: Vec<u64> = Vec::with_capacity(recipients.len());
    let mut account_offset2: usize = 0;
    let mut total_transferred: u64 = 0;

    for (i, recipient) in recipients.iter().enumerate() {
        if !recipient.enabled {
            amounts_per_recipient.push(0);
            continue;
        }

        let slot_count = if recipient.recipient_type == RecipientType::StakingRewards { 3 } else { 2 };
        let balance_src = remaining[account_offset2].clone();
        let dest_acct   = remaining[account_offset2 + 1].clone();
        account_offset2 += slot_count;

        if balances[i] == 0 {
            amounts_per_recipient.push(0);
            continue;
        }

        // Floor division — dust (amount - sum(recipient_amounts)) stays in vault for next round.
        let recipient_amount = (amount as u128)
            .checked_mul(balances[i] as u128)
            .ok_or(ErrorCode::CalculationOverflow)?
            .checked_div(total_balance)
            .ok_or(ErrorCode::CalculationOverflow)?;
        let recipient_amount_u64 =
            u64::try_from(recipient_amount).map_err(|_| ErrorCode::CalculationOverflow)?;

        amounts_per_recipient.push(recipient_amount_u64);

        if recipient_amount_u64 == 0 {
            continue;
        }

        let cpi_accounts = TransferChecked {
            from: ctx.accounts.router_vault.to_account_info(),
            mint: ctx.accounts.yield_token_mint.to_account_info(),
            to: dest_acct.clone(),
            authority: ctx.accounts.state.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            cpi_accounts,
            signer_seeds,
        );
        token::transfer_checked(cpi_ctx, recipient_amount_u64, decimals)?;

        // For StakingRewards: CPI into settle_yield to update the accumulator now that
        // tokens have landed in the staking vault. This makes yield distribution atomic.
        if recipient.recipient_type == RecipientType::StakingRewards {
            let staking_program = remaining[account_offset2 - 1].clone();
            require_keys_eq!(
                staking_program.key(),
                recipient.staking_program_id,
                ErrorCode::InvalidRecipientAccount
            );
            let cpi_accounts = SettleYieldAccounts {
                state: balance_src,
                staking_vault: dest_acct,
                stake_mint: ctx.accounts.yield_token_mint.to_account_info(),
            };
            let cpi_ctx = CpiContext::new(staking_program, cpi_accounts);
            staking_cpi::settle_yield(cpi_ctx)?;
        }

        total_transferred = total_transferred
            .checked_add(recipient_amount_u64)
            .ok_or(ErrorCode::CalculationOverflow)?;
    }

    ctx.accounts.state.total_distributed = ctx
        .accounts
        .state
        .total_distributed
        .checked_add(total_transferred)
        .ok_or(ErrorCode::CalculationOverflow)?;

    emit!(YieldDistributed {
        version: 1,
        vault_balance: amount,
        transferred: total_transferred,
        total_distributed: ctx.accounts.state.total_distributed,
        amounts_per_recipient,
    });

    Ok(())
}

/// Read the proportional balance from a balance_source account.
/// StakingRewards: deserializes abrafi-staking-rewards ProgramState and returns total_staked.
/// LiquidStaking / External: deserializes an SPL token account and returns amount.
fn read_balance(
    info: &AccountInfo,
    dest_key: &Pubkey,
    recipient_type: &RecipientType,
    yield_token_mint: &Pubkey,
    staking_program_id: Pubkey,
) -> Result<u64> {
    match recipient_type {
        RecipientType::StakingRewards => {
            require!(
                info.owner == &staking_program_id,
                ErrorCode::InvalidBalanceSource
            );
            // Verify the account is the canonical state PDA for the staking program —
            // not just any account owned by it.
            let (expected_pda, _) = Pubkey::find_program_address(
                &[b"abrafi_staking_rewards_state"],
                &staking_program_id,
            );
            require!(
                info.key() == expected_pda,
                ErrorCode::InvalidBalanceSource
            );
            let data = info
                .try_borrow_data()
                .map_err(|_| error!(ErrorCode::InvalidBalanceSource))?;
            let mut slice: &[u8] = &*data;
            let state =
                abrafi_staking_rewards::ProgramState::try_deserialize(&mut slice)
                    .map_err(|_| error!(ErrorCode::InvalidBalanceSource))?;
            // Confirm the staking program's stake_mint matches the yield token this router
            // distributes. Enforced at add_recipient time; re-checked here as defense in depth.
            require!(
                state.stake_mint == *yield_token_mint,
                ErrorCode::InvalidBalanceSource
            );
            require!(
                state.staking_vault == *dest_key,
                ErrorCode::InvalidRecipientAccount
            );
            Ok(state.total_staked)
        }
        RecipientType::LiquidStaking | RecipientType::External => {
            require!(
                info.owner == &anchor_spl::token::ID,
                ErrorCode::InvalidBalanceSource
            );
            let data = info
                .try_borrow_data()
                .map_err(|_| error!(ErrorCode::InvalidBalanceSource))?;
            let mut slice: &[u8] = &*data;
            let token_acct = anchor_spl::token::TokenAccount::try_deserialize(&mut slice)
                .map_err(|_| error!(ErrorCode::InvalidBalanceSource))?;
            // Defense-in-depth: reject a balance_source whose mint differs from the yield token.
            // add_recipient enforces balance_source == destination for these types, and destination
            // must hold yield_token_mint, so in normal operation this never fires. The destination
            // account's mint is separately enforced by transfer_checked at the SPL layer.
            require!(
                token_acct.mint == *yield_token_mint,
                ErrorCode::InvalidBalanceSource
            );
            Ok(token_acct.amount)
        }
    }
}
