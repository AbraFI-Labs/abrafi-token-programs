#[cfg(not(feature = "susdaf-compat"))]
pub const STATE_SEED: &[u8] = b"abrafi_liquid_staking_state";

#[cfg(feature = "susdaf-compat")]
pub const STATE_SEED: &[u8] = b"susdaf_state";

/// Seed for user unstake request PDAs
pub const USER_UNSTAKE_REQUEST_SEED: &[u8] = b"user_unstake_request";

/// Pending authority expiration period (24 hours)
pub const PENDING_AUTHORITY_EXPIRATION_SECONDS: i64 = 24 * 3600;

/// Treasury update cooldown period in seconds (24 hours)
pub const TREASURY_UPDATE_COOLDOWN_SECONDS: i64 = 24 * 3600;

/// Remove yield cooldown period in seconds (24 hours)
pub const REMOVE_YIELD_COOLDOWN_SECONDS: i64 = 24 * 3600;

/// Default withdrawal delay period (7 days)
pub const DEFAULT_WITHDRAWAL_DELAY_SECONDS: i64 = 7 * 24 * 3600;

/// Maximum withdrawal delay (30 days)
pub const MAX_WITHDRAWAL_DELAY_SECONDS: i64 = 30 * 24 * 3600;

/// Default max removal percentage in basis points (500 = 5%)
pub const DEFAULT_MAX_REMOVAL_PERCENTAGE: u16 = 500;
