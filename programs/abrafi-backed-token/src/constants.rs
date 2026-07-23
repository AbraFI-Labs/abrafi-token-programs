/// Seed for the main program state account.
/// When built with `--features usdaf-compat`, uses the legacy "usdaf_state" seed so that
/// an in-place upgrade of the existing USDAF deployment can locate the existing state account.
/// New deployments (SOLAF, etc.) omit this feature and use the default seed.
#[cfg(not(feature = "usdaf-compat"))]
pub const STATE_SEED: &[u8] = b"abrafi_backed_token_state";

#[cfg(feature = "usdaf-compat")]
pub const STATE_SEED: &[u8] = b"usdaf_state";

/// Seed for vault authority PDAs
pub const VAULT_AUTHORITY_SEED: &[u8] = b"vault_authority";

/// Seed for unmint details PDAs
pub const UNMINT_DETAILS_SEED: &[u8] = b"unmint_details";

/// Seed for mint whitelist PDAs
pub const MINT_WHITELIST_SEED: &[u8] = b"mint_whitelist";

/// Maximum number of collateral tokens supported
pub const MAX_COLLATERAL_TOKENS: usize = 10;

/// Maximum unmint cooldown period
pub const MAX_COOLDOWN_SECONDS: i64 = 30 * 24 * 3600; // 30 days

/// Minimum request expiration period (1 hour)
pub const MIN_EXPIRATION_SECONDS: i64 = 3600; // 1 hour

/// Maximum request expiration period
pub const MAX_EXPIRATION_SECONDS: i64 = 90 * 24 * 3600; // 90 days

/// Default request expiration period (14 days)
pub const DEFAULT_REQUEST_EXPIRATION_SECONDS: i64 = 14 * 24 * 3600; // 14 days

/// Default unmint cooldown period (48 hours)
pub const DEFAULT_UNMINT_COOLDOWN_SECONDS: i64 = 48 * 3600; // 48 hours

/// Maximum length for collateral token name
pub const MAX_TOKEN_NAME_LEN: usize = 32;

/// Pending authority expiration period (24 hours)
pub const PENDING_AUTHORITY_EXPIRATION_SECONDS: i64 = 24 * 3600; // 24 hours

/// Delay before a custom whitelist cooldown becomes effective (24 hours)
pub const CUSTOM_COOLDOWN_ACTIVATION_DELAY: i64 = 24 * 3600; // 24 hours
