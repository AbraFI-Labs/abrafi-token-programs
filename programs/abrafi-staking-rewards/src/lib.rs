use anchor_lang::prelude::*;

mod constants;
mod error;
mod events;
mod instructions;
mod state;
mod utils;

pub use state::ProgramState;

#[allow(unused_imports)]
use instructions::*;

include!("declare_id.rs");

#[program]
pub mod abrafi_staking_rewards {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        initialize_handler(ctx)
    }

    pub fn stake(ctx: Context<Stake>, amount: u64) -> Result<()> {
        stake_handler(ctx, amount)
    }

    pub fn set_stake_enabled(ctx: Context<SetStakeEnabled>, enabled: bool) -> Result<()> {
        set_stake_enabled_handler(ctx, enabled)
    }

    pub fn request_unstake(ctx: Context<RequestUnstake>, amount: u64) -> Result<()> {
        request_unstake_handler(ctx, amount)
    }

    pub fn set_unstake_request_enabled(
        ctx: Context<SetUnstakeRequestEnabled>,
        enabled: bool,
    ) -> Result<()> {
        set_unstake_request_enabled_handler(ctx, enabled)
    }

    pub fn claim_unstake(ctx: Context<ClaimUnstake>) -> Result<()> {
        claim_unstake_handler(ctx)
    }

    pub fn set_unstake_claim_enabled(
        ctx: Context<SetUnstakeClaimEnabled>,
        enabled: bool,
    ) -> Result<()> {
        set_unstake_claim_enabled_handler(ctx, enabled)
    }

    pub fn post_yield(ctx: Context<PostYield>, amount: u64) -> Result<()> {
        post_yield_handler(ctx, amount)
    }

    pub fn claim_yield(ctx: Context<ClaimYield>) -> Result<()> {
        claim_yield_handler(ctx)
    }

    pub fn set_claim_yield_enabled(
        ctx: Context<SetClaimYieldEnabled>,
        enabled: bool,
    ) -> Result<()> {
        set_claim_yield_enabled_handler(ctx, enabled)
    }

    pub fn update_withdrawal_delay(
        ctx: Context<UpdateWithdrawalDelay>,
        new_delay: i64,
    ) -> Result<()> {
        update_withdrawal_delay_handler(ctx, new_delay)
    }

    pub fn update_authority(ctx: Context<UpdateAuthority>) -> Result<()> {
        update_authority_handler(ctx)
    }

    pub fn finalize_authority(ctx: Context<FinalizeAuthority>) -> Result<()> {
        finalize_authority_handler(ctx)
    }

    pub fn update_operations_authority(ctx: Context<UpdateOperationsAuthority>) -> Result<()> {
        update_operations_authority_handler(ctx)
    }

    #[cfg(feature = "test")]
    pub fn set_withdrawal_delay_for_testing(
        ctx: Context<SetWithdrawalDelayForTesting>,
        new_delay: i64,
    ) -> Result<()> {
        set_withdrawal_delay_for_testing_handler(ctx, new_delay)
    }

    #[cfg(feature = "test")]
    pub fn set_pending_authority_timestamp_for_testing(
        ctx: Context<SetPendingAuthorityTimestampForTesting>,
        new_timestamp: i64,
    ) -> Result<()> {
        set_pending_authority_timestamp_for_testing_handler(ctx, new_timestamp)
    }

    #[cfg(feature = "test")]
    pub fn set_unstake_claim_after_ts_for_testing(
        ctx: Context<SetUnstakeClaimAfterTsForTesting>,
        new_ts: i64,
    ) -> Result<()> {
        set_unstake_claim_after_ts_for_testing_handler(ctx, new_ts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use state::*;

    #[test]
    fn test_program_state_defaults() {
        let state = make_test_state(0, 0);
        assert_eq!(state.version, 1);
        assert_eq!(state.total_staked, 0);
        assert_eq!(state.total_yield_allocated, 0);
        assert_eq!(state.total_pending_unstake, 0);
        assert_eq!(state.global_reward_index, 0);
        assert_eq!(state.withdrawal_delay, constants::DEFAULT_WITHDRAWAL_DELAY);
    }

    #[test]
    fn test_user_stake_account_defaults() {
        let user_stake = make_test_user_stake(0, 0, 0);
        assert_eq!(user_stake.staked_amount, 0);
        assert_eq!(user_stake.reward_index_snapshot, 0);
        assert_eq!(user_stake.pending_rewards, 0);
    }

    #[test]
    fn test_unstake_request_fields() {
        let req = UnstakeRequest {
            version: 1,
            bump: 253,
            user: Pubkey::new_unique(),
            amount: 500_000_000,
            claim_after_ts: 1_000_000_000,
        };
        assert_eq!(req.amount, 500_000_000);
        assert_eq!(req.claim_after_ts, 1_000_000_000);
    }

    #[test]
    fn test_update_pending_rewards_happy_path() {
        // User has 1000 staked, index moved from 0 to 10^17.
        // Earned = 1000 * 10^17 / 10^18 = 100. Compounded directly into staked_amount.
        let global_index = 100_000_000_000_000_000u128;
        let mut state = make_test_state(1000, global_index);
        state.total_yield_allocated = 100;
        let mut user_stake = make_test_user_stake(1000, 0, 0);
        let compounded = utils::update_pending_rewards(&mut state, &mut user_stake).unwrap();
        assert_eq!(compounded, 100);
        assert_eq!(user_stake.staked_amount, 1100);
        assert_eq!(state.total_staked, 1100);
        assert_eq!(state.total_yield_allocated, 0);
        assert_eq!(user_stake.reward_index_snapshot, global_index);
    }

    #[test]
    fn test_update_pending_rewards_zero_staked_is_noop() {
        let global_index = 100_000_000_000_000_000u128;
        let mut state = make_test_state(0, global_index);
        let mut user_stake = make_test_user_stake(0, 0, 0);
        let compounded = utils::update_pending_rewards(&mut state, &mut user_stake).unwrap();
        assert_eq!(compounded, 0);
        assert_eq!(user_stake.staked_amount, 0);
        assert_eq!(state.total_staked, 0);
        assert_eq!(user_stake.reward_index_snapshot, global_index);
    }

    #[test]
    fn test_update_pending_rewards_no_index_change_is_noop() {
        // Index has not moved since user's last snapshot — nothing to compound.
        let mut state = make_test_state(1000, 0);
        let mut user_stake = make_test_user_stake(1000, 0, 0);
        let compounded = utils::update_pending_rewards(&mut state, &mut user_stake).unwrap();
        assert_eq!(compounded, 0);
        assert_eq!(user_stake.staked_amount, 1000);
        assert_eq!(state.total_staked, 1000);
    }

    #[test]
    fn test_update_pending_rewards_overflow_guard() {
        // staked_amount = u64::MAX, index_delta = u128::MAX → intermediate product overflows.
        let mut state = make_test_state(u64::MAX, u128::MAX);
        let mut user_stake = make_test_user_stake(u64::MAX, 0, 0);
        let result = utils::update_pending_rewards(&mut state, &mut user_stake);
        assert!(result.is_err());
    }

    #[test]
    fn test_update_pending_rewards_u64_truncation() {
        // staked = u64::MAX, index_delta = 2 * PRECISION_FACTOR → earned = u64::MAX * 2, overflows u64.
        let global_index = 2 * constants::PRECISION_FACTOR;
        let mut state = make_test_state(u64::MAX, global_index);
        let mut user_stake = make_test_user_stake(u64::MAX, 0, 0);
        let result = utils::update_pending_rewards(&mut state, &mut user_stake);
        assert!(result.is_err());
    }

    #[test]
    fn test_update_pending_rewards_staked_amount_overflow() {
        // earned is valid u64 but staked_amount + earned overflows u64.
        // delta = PRECISION_FACTOR → earned = staked_amount (100% yield), u64::MAX + u64::MAX overflows.
        let global_index = constants::PRECISION_FACTOR;
        let mut state = make_test_state(u64::MAX, global_index);
        let mut user_stake = make_test_user_stake(u64::MAX, 0, 0);
        let result = utils::update_pending_rewards(&mut state, &mut user_stake);
        assert!(result.is_err());
    }

    // ─── helpers ────────────────────────────────────────────────────────────────

    fn make_test_state(total_staked: u64, global_reward_index: u128) -> state::ProgramState {
        state::ProgramState {
            version: 1,
            bump: 255,
            authority: Pubkey::new_unique(),
            pending_authority: Pubkey::default(),
            pending_authority_expiration_timestamp: 0,
            operations_authority: Pubkey::new_unique(),
            stake_mint: Pubkey::new_unique(),
            staking_vault: Pubkey::new_unique(),
            total_staked,
            total_yield_allocated: 0,
            total_pending_unstake: 0,
            global_reward_index,
            withdrawal_delay: constants::DEFAULT_WITHDRAWAL_DELAY,
            stake_enabled: true,
            unstake_request_enabled: true,
            unstake_claim_enabled: true,
            claim_yield_enabled: true,
        }
    }

    fn make_test_user_stake(
        staked_amount: u64,
        reward_index_snapshot: u128,
        pending_rewards: u64,
    ) -> state::UserStakeAccount {
        state::UserStakeAccount {
            version: 1,
            bump: 254,
            user: Pubkey::new_unique(),
            staked_amount,
            reward_index_snapshot,
            pending_rewards,
        }
    }
}
