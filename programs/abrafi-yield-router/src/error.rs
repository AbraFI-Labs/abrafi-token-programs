use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    // AUTHORITY ERRORS
    #[msg("Signer does not match expected authority")]
    InvalidAuthority,
    #[msg("No pending authority transfer to finalize")]
    NoPendingAuthorityTransfer,
    #[msg("Pending authority transfer has expired")]
    PendingAuthorityExpired,
    #[msg("All authority addresses must be non-default and mutually distinct")]
    AuthoritiesMustBeDifferent,

    // CONFIGURATION ERRORS
    #[msg("Invalid configuration value provided")]
    InvalidConfiguration,

    // RECIPIENT ERRORS
    #[msg("Maximum number of recipients already reached")]
    MaxRecipientsReached,
    #[msg("Recipient index is out of bounds")]
    InvalidRecipientIndex,
    #[msg("Destination token account mint does not match yield token mint")]
    InvalidDestinationMint,
    #[msg("Destination cannot be the router vault")]
    InvalidDestination,
    #[msg("Balance source account is invalid or cannot be read")]
    InvalidBalanceSource,

    // DISTRIBUTION ERRORS
    #[msg("Yield distribution is currently disabled")]
    DistributeDisabled,
    #[msg("All enabled recipients have zero balance — nothing to distribute")]
    ZeroTotalBalance,
    #[msg("Vault balance is below the minimum distribution amount")]
    AmountBelowMinimum,
    #[msg("Number of remaining accounts does not match enabled recipient count * 2")]
    RecipientAccountMismatch,
    #[msg("Remaining account does not match the expected recipient address")]
    InvalidRecipientAccount,

    // CALCULATION ERRORS
    #[msg("Arithmetic overflow occurred")]
    CalculationOverflow,
}
