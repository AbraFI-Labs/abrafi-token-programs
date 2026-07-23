use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;

use crate::constants::{MAX_RECIPIENTS, STATE_SEED};
use crate::error::ErrorCode;
use crate::events::RecipientAdded;
use crate::state::{RecipientConfig, RecipientType, RouterState};

#[derive(Accounts)]
pub struct AddRecipient<'info> {
    // Space pre-allocated at init for MAX_RECIPIENTS via #[max_len(10)] — no realloc needed.
    #[account(mut, seeds = [STATE_SEED], bump = state.bump, has_one = authority)]
    pub state: Account<'info, RouterState>,

    pub authority: Signer<'info>,

    /// Destination token account — receives yield. Must hold yield_token_mint, not the router vault.
    #[account(
        constraint = destination.mint == state.yield_token_mint @ ErrorCode::InvalidDestinationMint,
        constraint = destination.key() != state.router_vault @ ErrorCode::InvalidDestination,
    )]
    pub destination: Account<'info, TokenAccount>,

    /// Account read for proportional balance at distribution time.
    /// StakingRewards: abrafi-staking-rewards ProgramState PDA (total_staked).
    /// LiquidStaking / External: pass the same address as destination (token account amount).
    /// CHECK: Validated in handler — non-default for all types;
    ///        StakingRewards: must differ from destination, owned by the provided staking_program_id,
    ///        and deserializable as ProgramState;
    ///        LiquidStaking / External: must equal destination.
    pub balance_source: UncheckedAccount<'info>,
}

pub fn add_recipient_handler(
    ctx: Context<AddRecipient>,
    recipient_type: RecipientType,
    staking_program_id: Pubkey,
) -> Result<()> {
    require!(
        ctx.accounts.state.recipients.len() < MAX_RECIPIENTS,
        ErrorCode::MaxRecipientsReached
    );

    let destination = ctx.accounts.destination.key();
    let balance_source = ctx.accounts.balance_source.key();

    require!(
        ctx.accounts.state.recipients.iter().all(|r| r.destination != destination),
        ErrorCode::InvalidConfiguration
    );

    require!(
        balance_source != Pubkey::default(),
        ErrorCode::InvalidBalanceSource
    );

    match recipient_type {
        RecipientType::StakingRewards => {
            // balance_source is the staking program state PDA — must differ from the destination vault
            require!(
                balance_source != destination,
                ErrorCode::InvalidBalanceSource
            );
            require!(
                staking_program_id != Pubkey::default(),
                ErrorCode::InvalidBalanceSource
            );
            // Validate owner + discriminator before storing. Without this, any pubkey that passes
            // the != destination check could consume a slot and cause every distribute_yield to
            // fail with InvalidBalanceSource until an operator manually disables the recipient.
            let bs_info = ctx.accounts.balance_source.to_account_info();
            require!(
                bs_info.owner == &staking_program_id,
                ErrorCode::InvalidBalanceSource
            );
            let data = bs_info
                .try_borrow_data()
                .map_err(|_| error!(ErrorCode::InvalidBalanceSource))?;
            let mut slice: &[u8] = &*data;
            let staking_state =
                abrafi_staking_rewards::ProgramState::try_deserialize(&mut slice)
                    .map_err(|_| error!(ErrorCode::InvalidBalanceSource))?;
            // Ensure the staking program stakes the same token this router distributes.
            // Without this, total_balance in distribute_yield would mix incomparable units.
            require!(
                staking_state.stake_mint == ctx.accounts.state.yield_token_mint,
                ErrorCode::InvalidBalanceSource
            );
        }
        RecipientType::LiquidStaking | RecipientType::External => {
            // balance_source is the same token account as destination
            require!(
                balance_source == destination,
                ErrorCode::InvalidBalanceSource
            );
        }
    }

    let index = ctx.accounts.state.recipients.len() as u8;

    ctx.accounts.state.recipients.push(RecipientConfig {
        destination,
        recipient_type: recipient_type.clone(),
        balance_source,
        staking_program_id,
        enabled: true,
    });

    emit!(RecipientAdded {
        version: 1,
        index,
        destination,
        balance_source,
        staking_program_id,
        recipient_type,
    });

    Ok(())
}
