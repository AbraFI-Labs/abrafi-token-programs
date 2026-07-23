use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, TransferChecked};

use crate::constants::STATE_SEED;
use crate::error::ErrorCode;
use crate::events::YieldDistributed;
use crate::state::{enabled_recipient_count, RecipientType, RouterState};

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
    // remaining_accounts: interleaved pairs per enabled recipient, in Vec order:
    //   [balance_src_0 (readonly), dest_0 (writable), balance_src_1 (readonly), dest_1 (writable), ...]
    // Count must equal enabled_recipient_count * 2.
    // StakingRewards balance_src: abrafi-staking-rewards ProgramState PDA (total_staked).
    // LiquidStaking / External balance_src: same as dest — token account amount.
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

    let enabled_count = enabled_recipient_count(&ctx.accounts.state.recipients);
    let expected_accounts = enabled_count
        .checked_mul(2)
        .ok_or(ErrorCode::CalculationOverflow)?;
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
    let mut pair_idx: usize = 0;

    for recipient in recipients.iter() {
        if !recipient.enabled {
            balances.push(0);
            continue;
        }

        let balance_src = &remaining[pair_idx * 2];
        let dest_acct   = &remaining[pair_idx * 2 + 1];
        pair_idx += 1;

        require!(
            balance_src.key() == recipient.balance_source,
            ErrorCode::InvalidRecipientAccount
        );
        require!(
            dest_acct.key() == recipient.destination,
            ErrorCode::InvalidRecipientAccount
        );
        // balance_source is read-only; destination must be writable for the CPI transfer.
        require!(!balance_src.is_writable, ErrorCode::InvalidRecipientAccount);
        require!(dest_acct.is_writable,    ErrorCode::InvalidRecipientAccount);

        let balance = read_balance(balance_src, &recipient.recipient_type, &yield_token_mint_key, recipient.staking_program_id)?;
        balances.push(balance);
    }

    let total_balance: u128 = balances.iter().map(|&b| b as u128).sum();
    require!(total_balance > 0, ErrorCode::ZeroTotalBalance);

    // ── Phase 2: calculate proportional amounts and transfer ──────────────────
    let mut amounts_per_recipient: Vec<u64> = Vec::with_capacity(recipients.len());
    let mut pair_idx: usize = 0;
    let mut total_transferred: u64 = 0;

    for (i, recipient) in recipients.iter().enumerate() {
        if !recipient.enabled {
            amounts_per_recipient.push(0);
            continue;
        }

        let dest_acct = remaining[pair_idx * 2 + 1].clone();
        pair_idx += 1;

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

        let cpi_accounts = TransferChecked {
            from: ctx.accounts.router_vault.to_account_info(),
            mint: ctx.accounts.yield_token_mint.to_account_info(),
            to: dest_acct,
            authority: ctx.accounts.state.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            cpi_accounts,
            signer_seeds,
        );
        token::transfer_checked(cpi_ctx, recipient_amount_u64, decimals)?;

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
