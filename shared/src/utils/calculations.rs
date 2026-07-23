use anchor_lang::prelude::*;

/// Calculate minimum amount based on token decimals
/// Returns 10^decimals with overflow detection
pub fn calculate_minimum_amount_from_decimals<E: Into<Error>>(decimals: u8, overflow_error: E) -> Result<u64> {
    // Calculate 10^decimals with overflow protection
    // Using checked_pow to detect overflow
    10u64
        .checked_pow(decimals as u32)
        .ok_or(overflow_error.into())
}

/// Safely add a delay to a timestamp with overflow checking
/// Generic function for calculating timestamps after a delay period
/// Used for withdrawal delays, request expirations, cooldowns, etc.
///
/// # Arguments
/// * `timestamp` - The base timestamp
/// * `delay_seconds` - The delay to add in seconds
/// * `overflow_error` - Error to return on overflow
///
/// # Returns
/// The timestamp after adding the delay, or an error if overflow occurs
pub fn safe_add_delay<E: Into<Error>>(timestamp: i64, delay_seconds: i64, overflow_error: E) -> Result<i64> {
    timestamp
        .checked_add(delay_seconds)
        .ok_or(overflow_error.into())
}

/// Scale amount from one token's decimal representation to another
/// This function rounds DOWN (truncates) when scaling to a token with fewer decimals.
///
/// # Arguments
/// * `amount` - The raw amount in `from_decimals` representation
/// * `from_decimals` - Decimal places of the source token
/// * `to_decimals` - Decimal places of the destination token
/// * `overflow_error` - Error to return on overflow
///
/// # Returns
/// The scaled amount in `to_decimals` representation (rounded down if division occurs)
///
/// # Examples
/// - Scaling 9 decimals to 6 decimals: 5000000000 / 1000 = 5000000 (truncates remainder)
/// - Scaling 2 decimals to 6 decimals: 1000 * 10000 = 10000000 (exact)
pub fn scale_amount_to_new_decimals<E: Into<Error> + Clone>(amount: u64, from_decimals: u8, to_decimals: u8, overflow_error: E) -> Result<u64> {
    if from_decimals == to_decimals {
        return Ok(amount);
    }

    if from_decimals > to_decimals {
        // Need to divide: amount / 10^(from_decimals - to_decimals)
        // Integer division in Rust truncates (rounds down), which is the desired behavior
        let divisor = 10u64
            .checked_pow((from_decimals - to_decimals) as u32)
            .ok_or_else(|| overflow_error.clone().into())?;
        amount
            .checked_div(divisor)
            .ok_or_else(|| overflow_error.into())
    } else {
        // Need to multiply: amount * 10^(to_decimals - from_decimals)
        // Multiplication is exact, no rounding needed
        let multiplier = 10u64
            .checked_pow((to_decimals - from_decimals) as u32)
            .ok_or_else(|| overflow_error.clone().into())?;
        amount
            .checked_mul(multiplier)
            .ok_or_else(|| overflow_error.into())
    }
}

