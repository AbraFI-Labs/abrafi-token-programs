#[cfg(test)]
mod tests {
    use anchor_lang::prelude::*;
    use crate::utils::calculations::{
        calculate_minimum_amount_from_decimals,
        safe_add_delay,
        scale_amount_to_new_decimals,
    };
    use crate::utils::validations::{
        validate_amount_meets_minimum,
        validate_sufficient_balance,
        validate_balance_zero_or_above_minimum,
        validate_amount_full_or_above_minimum,
        validate_timestamp_has_passed,
        validate_timestamp_has_not_passed,
    };

    // Simple test error type for shared tests
    #[derive(Debug, Clone)]
    struct TestError;

    impl From<TestError> for Error {
        fn from(_: TestError) -> Self {
            Error::from(anchor_lang::error::ErrorCode::ConstraintOwner)
        }
    }

    #[test]
    fn test_calculate_minimum_amount_from_decimals() {
        // Test valid decimal values
        assert_eq!(calculate_minimum_amount_from_decimals(0, TestError).unwrap(), 1);
        assert_eq!(calculate_minimum_amount_from_decimals(1, TestError).unwrap(), 10);
        assert_eq!(calculate_minimum_amount_from_decimals(2, TestError).unwrap(), 100);
        assert_eq!(calculate_minimum_amount_from_decimals(6, TestError).unwrap(), 1_000_000);
        assert_eq!(calculate_minimum_amount_from_decimals(9, TestError).unwrap(), 1_000_000_000);

        // Test maximum valid decimals (10^19 is within u64::MAX which is ~1.8e19)
        // u64::MAX = 18446744073709551615
        // 10^19 = 10000000000000000000 (fits in u64)
        // 10^20 = 100000000000000000000 (overflows u64)
        assert_eq!(calculate_minimum_amount_from_decimals(18, TestError).unwrap(), 1_000_000_000_000_000_000);
    }

    #[test]
    fn test_calculate_minimum_amount_from_decimals_overflow() {
        // Test overflow case - 10^20 exceeds u64::MAX
        // u64::MAX = 18446744073709551615
        // 10^19 = 10000000000000000000 (fits)
        // 10^20 = 100000000000000000000 (overflows)
        let result = calculate_minimum_amount_from_decimals(20, TestError);
        assert!(result.is_err());

        // Test with even larger decimals
        let result = calculate_minimum_amount_from_decimals(255, TestError);
        assert!(result.is_err());
    }

    #[test]
    fn test_safe_add_delay() {
        let request_timestamp = 1640995200i64; // 2022-01-01 00:00:00 UTC
        let delay_seconds = 7 * 24 * 3600i64; // 7 days in seconds

        let result = safe_add_delay(request_timestamp, delay_seconds, TestError).unwrap();
        let expected_end = 1641600000i64; // 2022-01-08 00:00:00 UTC

        assert_eq!(result, expected_end);
        assert!(result > request_timestamp);

        // Test edge cases
        assert_eq!(safe_add_delay(0, 0, TestError).unwrap(), 0);
        assert_eq!(safe_add_delay(100, 3600, TestError).unwrap(), 3700); // 100 + 3600
    }

    #[test]
    fn test_safe_add_delay_overflow() {
        // Test overflow when adding would exceed i64::MAX
        let max_timestamp = i64::MAX;
        let positive_delay = 1i64;

        let result = safe_add_delay(max_timestamp, positive_delay, TestError);
        assert!(result.is_err());

        // Test with large positive delay
        let large_delay = i64::MAX;
        let result = safe_add_delay(1i64, large_delay, TestError);
        assert!(result.is_err());

        // Test that normal operations near but not at the limit work
        let near_max = i64::MAX - 1000;
        let small_delay = 500i64;
        let result = safe_add_delay(near_max, small_delay, TestError);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), near_max + small_delay);
    }

    #[test]
    fn test_scale_amount_to_new_decimals_same_decimals() {
        // Test scaling with same decimals (no scaling needed)
        assert_eq!(scale_amount_to_new_decimals(1000000, 6, 6, TestError).unwrap(), 1000000);
        assert_eq!(scale_amount_to_new_decimals(5000000000, 9, 9, TestError).unwrap(), 5000000000);
        assert_eq!(scale_amount_to_new_decimals(1000, 2, 2, TestError).unwrap(), 1000);
    }

    #[test]
    fn test_scale_amount_to_new_decimals_round_down_when_dividing() {
        // Test rounding down when scaling from more decimals to fewer decimals
        // 9 decimals to 6 decimals: divide by 1000
        // 5000000001 / 1000 = 5000000 (truncates remainder 1)
        assert_eq!(scale_amount_to_new_decimals(5000000001, 9, 6, TestError).unwrap(), 5000000);

        // 5000000999 / 1000 = 5000000 (truncates remainder 999)
        assert_eq!(scale_amount_to_new_decimals(5000000999, 9, 6, TestError).unwrap(), 5000000);

        // 2 decimals to 6 decimals: multiply by 10000 (exact, no rounding)
        assert_eq!(scale_amount_to_new_decimals(1000, 2, 6, TestError).unwrap(), 10000000);

        // 6 decimals to 2 decimals: divide by 10000 (rounds down)
        // 10000001 / 10000 = 1000 (truncates remainder 1)
        assert_eq!(scale_amount_to_new_decimals(10000001, 6, 2, TestError).unwrap(), 1000);

        // 10000999 / 10000 = 1000 (truncates remainder 999)
        assert_eq!(scale_amount_to_new_decimals(10000999, 6, 2, TestError).unwrap(), 1000);
    }

    #[test]
    fn test_scale_amount_to_new_decimals_multiply_exact() {
        // Test multiplication (exact, no rounding)
        // 2 decimals to 6 decimals: multiply by 10000
        assert_eq!(scale_amount_to_new_decimals(1000, 2, 6, TestError).unwrap(), 10000000);
        assert_eq!(scale_amount_to_new_decimals(500, 2, 6, TestError).unwrap(), 5000000);

        // 6 decimals to 9 decimals: multiply by 1000
        assert_eq!(scale_amount_to_new_decimals(5000000, 6, 9, TestError).unwrap(), 5000000000);
        assert_eq!(scale_amount_to_new_decimals(1000000, 6, 9, TestError).unwrap(), 1000000000);
    }

    #[test]
    fn test_scale_amount_to_new_decimals_mint_scenario() {
        // Test minting scenario: collateral (9 decimals) -> USDAF (6 decimals)
        // 5.5 collateral tokens = 5500000000 raw (9 decimals)
        // Should scale to 5500000 USDAF (6 decimals) = 5.5 tokens
        // But with rounding down: 5500000000 / 1000 = 5500000 (exact in this case)
        assert_eq!(scale_amount_to_new_decimals(5500000000, 9, 6, TestError).unwrap(), 5500000);

        // 5.0001 collateral tokens = 5000100000 raw (9 decimals)
        // Should scale to 5000100 USDAF (6 decimals) = 5.0001 tokens
        // With rounding down: 5000100000 / 1000 = 5000100 (exact)
        assert_eq!(scale_amount_to_new_decimals(5000100000, 9, 6, TestError).unwrap(), 5000100);

        // 5.0000001 collateral tokens = 5000000100 raw (9 decimals)
        // Should scale to 5000000 USDAF (6 decimals) = 5.0 tokens (truncates 100)
        assert_eq!(scale_amount_to_new_decimals(5000000100, 9, 6, TestError).unwrap(), 5000000);
    }

    #[test]
    fn test_scale_amount_to_new_decimals_unmint_scenario() {
        // Test unminting scenario: USDAF (6 decimals) -> collateral (2 decimals)
        // 10.0001 USDAF tokens = 10000100 raw (6 decimals)
        // Should scale to 1000 collateral (2 decimals) = 10.0 tokens (truncates 100)
        assert_eq!(scale_amount_to_new_decimals(10000100, 6, 2, TestError).unwrap(), 1000);

        // 10.9999 USDAF tokens = 10999900 raw (6 decimals)
        // Should scale to 1099 collateral (2 decimals) = 10.99 tokens (truncates 9900)
        assert_eq!(scale_amount_to_new_decimals(10999900, 6, 2, TestError).unwrap(), 1099);

        // Test unminting scenario: USDAF (6 decimals) -> collateral (9 decimals)
        // 5 USDAF tokens = 5000000 raw (6 decimals)
        // Should scale to 5000000000 collateral (9 decimals) = 5.0 tokens (exact)
        assert_eq!(scale_amount_to_new_decimals(5000000, 6, 9, TestError).unwrap(), 5000000000);
    }

    #[test]
    fn test_scale_amount_to_new_decimals_zero_result() {
        // Test that very small amounts round down to zero
        // 999 raw (9 decimals) / 1000 = 0 (truncates)
        assert_eq!(scale_amount_to_new_decimals(999, 9, 6, TestError).unwrap(), 0);

        // 99 raw (6 decimals) / 10000 = 0 (truncates)
        assert_eq!(scale_amount_to_new_decimals(99, 6, 2, TestError).unwrap(), 0);
    }

    #[test]
    fn test_scale_amount_to_new_decimals_overflow() {
        // Test overflow cases
        let max_u64 = u64::MAX;

        // Multiplying by a large factor should overflow
        let result = scale_amount_to_new_decimals(max_u64, 6, 18, TestError);
        assert!(result.is_err());

        // Very large divisor calculation should overflow
        let result = scale_amount_to_new_decimals(1000, 255, 0, TestError);
        assert!(result.is_err());
    }

    // VALIDATION FUNCTION TESTS

    #[test]
    fn test_validate_amount_meets_minimum() {
        // Test valid cases
        assert!(validate_amount_meets_minimum(100, 50, TestError).is_ok());
        assert!(validate_amount_meets_minimum(100, 100, TestError).is_ok());
        assert!(validate_amount_meets_minimum(1000, 100, TestError).is_ok());

        // Test invalid cases
        assert!(validate_amount_meets_minimum(50, 100, TestError).is_err());
        assert!(validate_amount_meets_minimum(0, 100, TestError).is_err());
        assert!(validate_amount_meets_minimum(99, 100, TestError).is_err());
    }

    #[test]
    fn test_validate_sufficient_balance() {
        // Test valid cases
        assert!(validate_sufficient_balance(100, 200, TestError).is_ok());
        assert!(validate_sufficient_balance(100, 100, TestError).is_ok());
        assert!(validate_sufficient_balance(1, 1000, TestError).is_ok());

        // Test invalid cases - zero amount
        assert!(validate_sufficient_balance(0, 100, TestError).is_err());
        assert!(validate_sufficient_balance(0, 0, TestError).is_err());

        // Test invalid cases - insufficient balance
        assert!(validate_sufficient_balance(100, 50, TestError).is_err());
        assert!(validate_sufficient_balance(100, 99, TestError).is_err());
        assert!(validate_sufficient_balance(1, 0, TestError).is_err());
    }

    #[test]
    fn test_validate_balance_zero_or_above_minimum() {
        // Test valid cases - zero balance
        assert!(validate_balance_zero_or_above_minimum(0, 100, TestError).is_ok());
        assert!(validate_balance_zero_or_above_minimum(0, 1000, TestError).is_ok());

        // Test valid cases - balance meets minimum
        assert!(validate_balance_zero_or_above_minimum(100, 100, TestError).is_ok());
        assert!(validate_balance_zero_or_above_minimum(200, 100, TestError).is_ok());
        assert!(validate_balance_zero_or_above_minimum(1000, 100, TestError).is_ok());

        // Test invalid cases - balance below minimum (and not zero)
        assert!(validate_balance_zero_or_above_minimum(50, 100, TestError).is_err());
        assert!(validate_balance_zero_or_above_minimum(99, 100, TestError).is_err());
        assert!(validate_balance_zero_or_above_minimum(1, 100, TestError).is_err());
    }

    #[test]
    fn test_validate_amount_full_or_above_minimum() {
        let full_balance = 1000u64;
        let minimum = 100u64;

        // Test valid cases - full balance
        assert!(validate_amount_full_or_above_minimum(full_balance, full_balance, minimum, TestError).is_ok());

        // Test valid cases - amount meets minimum
        assert!(validate_amount_full_or_above_minimum(100, full_balance, minimum, TestError).is_ok());
        assert!(validate_amount_full_or_above_minimum(200, full_balance, minimum, TestError).is_ok());
        assert!(validate_amount_full_or_above_minimum(500, full_balance, minimum, TestError).is_ok());

        // Test invalid cases - amount below minimum (and not full balance)
        assert!(validate_amount_full_or_above_minimum(50, full_balance, minimum, TestError).is_err());
        assert!(validate_amount_full_or_above_minimum(99, full_balance, minimum, TestError).is_err());
        assert!(validate_amount_full_or_above_minimum(0, full_balance, minimum, TestError).is_err());
    }

    #[test]
    fn test_validate_timestamp_has_passed() {
        let target_timestamp = 1641600000i64; // 2022-01-08 00:00:00 UTC

        // Test valid cases - timestamp has passed
        assert!(validate_timestamp_has_passed(1641600000, target_timestamp, TestError).is_ok()); // exactly at target
        assert!(validate_timestamp_has_passed(1641600001, target_timestamp, TestError).is_ok()); // after target
        assert!(validate_timestamp_has_passed(1641696000, target_timestamp, TestError).is_ok()); // well after target

        // Test invalid cases - timestamp has not passed
        assert!(validate_timestamp_has_passed(1641599999, target_timestamp, TestError).is_err()); // 1 second before
        assert!(validate_timestamp_has_passed(1641513600, target_timestamp, TestError).is_err()); // 1 day before
        assert!(validate_timestamp_has_passed(0, target_timestamp, TestError).is_err()); // epoch start
    }

    #[test]
    fn test_validate_timestamp_has_passed_edge_cases() {
        // Test with zero timestamps
        assert!(validate_timestamp_has_passed(0, 0, TestError).is_ok()); // exactly at zero
        assert!(validate_timestamp_has_passed(1, 0, TestError).is_ok()); // after zero
        assert!(validate_timestamp_has_passed(0, 1, TestError).is_err()); // before target

        // Test with negative timestamps (edge case)
        assert!(validate_timestamp_has_passed(-1, -2, TestError).is_ok()); // -1 >= -2
        assert!(validate_timestamp_has_passed(-2, -1, TestError).is_err()); // -2 < -1

        // Test with large timestamps
        let large_timestamp = i64::MAX - 1000;
        assert!(validate_timestamp_has_passed(large_timestamp, large_timestamp, TestError).is_ok());
        assert!(validate_timestamp_has_passed(large_timestamp + 1, large_timestamp, TestError).is_ok());
        assert!(validate_timestamp_has_passed(large_timestamp - 1, large_timestamp, TestError).is_err());
    }

    #[test]
    fn test_validate_timestamp_has_not_passed() {
        let target_timestamp = 1641600000i64; // 2022-01-08 00:00:00 UTC

        // Test valid cases - timestamp has not passed
        assert!(validate_timestamp_has_not_passed(1641599999, target_timestamp, TestError).is_ok()); // 1 second before
        assert!(validate_timestamp_has_not_passed(1641513600, target_timestamp, TestError).is_ok()); // 1 day before
        assert!(validate_timestamp_has_not_passed(0, target_timestamp, TestError).is_ok()); // epoch start

        // Test invalid cases - timestamp has passed
        assert!(validate_timestamp_has_not_passed(1641600000, target_timestamp, TestError).is_err()); // exactly at target
        assert!(validate_timestamp_has_not_passed(1641600001, target_timestamp, TestError).is_err()); // after target
        assert!(validate_timestamp_has_not_passed(1641696000, target_timestamp, TestError).is_err()); // well after target
    }

    #[test]
    fn test_validate_timestamp_has_not_passed_edge_cases() {
        // Test with zero timestamps
        assert!(validate_timestamp_has_not_passed(0, 1, TestError).is_ok()); // before target
        assert!(validate_timestamp_has_not_passed(0, 0, TestError).is_err()); // exactly at zero (has passed)
        assert!(validate_timestamp_has_not_passed(1, 0, TestError).is_err()); // after zero (has passed)

        // Test with negative timestamps (edge case)
        assert!(validate_timestamp_has_not_passed(-2, -1, TestError).is_ok()); // -2 < -1
        assert!(validate_timestamp_has_not_passed(-1, -2, TestError).is_err()); // -1 >= -2 (has passed)

        // Test with large timestamps
        let large_timestamp = i64::MAX - 1000;
        assert!(validate_timestamp_has_not_passed(large_timestamp - 1, large_timestamp, TestError).is_ok());
        assert!(validate_timestamp_has_not_passed(large_timestamp, large_timestamp, TestError).is_err());
        assert!(validate_timestamp_has_not_passed(large_timestamp + 1, large_timestamp, TestError).is_err());
    }
}

