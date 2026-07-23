use anchor_lang::prelude::*;

/// Validate that an amount meets or exceeds a minimum requirement
///
/// # Arguments
/// * `amount` - The amount to validate
/// * `minimum` - The minimum required amount
/// * `error` - Error code to return if validation fails
///
/// # Returns
/// `Ok(())` if `amount >= minimum`, otherwise returns the provided error
pub fn validate_amount_meets_minimum<E: Into<Error>>(amount: u64, minimum: u64, error: E) -> Result<()> {
    if amount < minimum {
        return Err(error.into());
    }
    Ok(())
}

/// Validate that an amount is greater than zero and the balance is sufficient
///
/// # Arguments
/// * `amount` - The amount to validate (must be > 0)
/// * `balance` - The available balance (must be >= amount)
/// * `error` - Error code to return if validation fails
///
/// # Returns
/// `Ok(())` if `amount > 0 && balance >= amount`, otherwise returns the provided error
pub fn validate_sufficient_balance<E: Into<Error>>(amount: u64, balance: u64, error: E) -> Result<()> {
    if amount == 0 || balance < amount {
        return Err(error.into());
    }
    Ok(())
}

/// Validate that a balance is either zero or meets the minimum requirement
///
/// This is commonly used to ensure that after an operation, a remaining balance
/// is either completely depleted (zero) or still meets the minimum threshold.
///
/// # Arguments
/// * `balance` - The balance to validate
/// * `minimum` - The minimum required amount (if balance is not zero)
/// * `error` - Error code to return if validation fails
///
/// # Returns
/// `Ok(())` if `balance == 0 || balance >= minimum`, otherwise returns the provided error
pub fn validate_balance_zero_or_above_minimum<E: Into<Error>>(balance: u64, minimum: u64, error: E) -> Result<()> {
    if balance > 0 && balance < minimum {
        return Err(error.into());
    }
    Ok(())
}

/// Validate that an amount is either the full balance or meets the minimum requirement
///
/// This is commonly used for partial claim/cancel operations where the user can
/// either claim/cancel the entire amount or a partial amount that meets the minimum.
///
/// # Arguments
/// * `amount` - The amount to validate
/// * `full_balance` - The full available balance
/// * `minimum` - The minimum required amount (if amount is not the full balance)
/// * `error` - Error code to return if validation fails
///
/// # Returns
/// `Ok(())` if `amount == full_balance || amount >= minimum`, otherwise returns the provided error
pub fn validate_amount_full_or_above_minimum<E: Into<Error>>(amount: u64, full_balance: u64, minimum: u64, error: E) -> Result<()> {
    if amount != full_balance && amount < minimum {
        return Err(error.into());
    }
    Ok(())
}

/// Validate that a timestamp has passed (current timestamp is >= target timestamp)
///
/// This is commonly used for checking if cooldowns, withdrawal delays, or other time-based
/// restrictions have expired.
///
/// # Arguments
/// * `current_timestamp` - The current timestamp
/// * `target_timestamp` - The target timestamp that must have passed
/// * `error` - Error code to return if validation fails
///
/// # Returns
/// `Ok(())` if `current_timestamp >= target_timestamp`, otherwise returns the provided error
pub fn validate_timestamp_has_passed<E: Into<Error>>(current_timestamp: i64, target_timestamp: i64, error: E) -> Result<()> {
    if current_timestamp < target_timestamp {
        return Err(error.into());
    }
    Ok(())
}

/// Validate that a timestamp has not passed (current timestamp is < target timestamp)
///
/// This is commonly used for checking if requests, offers, or other time-limited operations
/// are still valid and haven't expired.
///
/// # Arguments
/// * `current_timestamp` - The current timestamp
/// * `target_timestamp` - The target timestamp that must not have passed
/// * `error` - Error code to return if validation fails
///
/// # Returns
/// `Ok(())` if `current_timestamp < target_timestamp`, otherwise returns the provided error
pub fn validate_timestamp_has_not_passed<E: Into<Error>>(current_timestamp: i64, target_timestamp: i64, error: E) -> Result<()> {
    if current_timestamp >= target_timestamp {
        return Err(error.into());
    }
    Ok(())
}

