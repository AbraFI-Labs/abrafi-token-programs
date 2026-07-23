pub const STATE_SEED: &[u8] = b"abrafi_staking_rewards_state";
pub const USER_STAKE_SEED: &[u8] = b"user_stake";
pub const UNSTAKE_REQUEST_SEED: &[u8] = b"unstake_request";

/// Accumulator precision: 10^18. Must be u128.
/// 10^12 truncates to 0 for small reward injections into large pools — do not change.
pub const PRECISION_FACTOR: u128 = 1_000_000_000_000_000_000;

pub const DEFAULT_WITHDRAWAL_DELAY: i64 = 7 * 24 * 3600;        // 7 days
pub const MAX_WITHDRAWAL_DELAY: i64 = 90 * 24 * 3600;           // 90 days
pub const PENDING_AUTHORITY_EXPIRATION_SECONDS: i64 = 24 * 3600; // 24 hours
