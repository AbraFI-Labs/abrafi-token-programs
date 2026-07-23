use anchor_lang::prelude::*;

mod constants;
mod error;
mod events;
mod instructions;
mod state;
mod utils;

use instructions::*;

// Include the generated declare_id! statement
include!("declare_id.rs");

#[program]
pub mod abrafi_backed_token {
    use super::*;

    /// Initialize the abrafi token program with multiple collateral token mints and their treasuries
    /// This sets up the program state and creates the abrafi token mint
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        initialize_handler(ctx)
    }

    /// Add a new collateral token and its treasury
    /// This function can only be called by the authority
    pub fn add_collateral_token(ctx: Context<AddCollateralToken>, name: String) -> Result<()> {
        add_collateral_token_handler(ctx, name)
    }

    /// Disable a collateral token
    /// Disabled tokens cannot be used for minting or new unmint requests
    /// They remain in the data structure to allow withdrawing funds from vaults
    /// This function can only be called by the authority
    pub fn disable_collateral_token(
        ctx: Context<DisableCollateralToken>,
    ) -> Result<()> {
        disable_collateral_token_handler(ctx)
    }

    /// Remove a collateral token
    /// Requires that the withdrawal vault is empty
    /// This function can only be called by the authority
    pub fn remove_collateral_token(
        ctx: Context<RemoveCollateralToken>,
    ) -> Result<()> {
        remove_collateral_token_handler(ctx)
    }

    /// Mint abrafi tokens using collateral tokens
    /// Users can mint abrafi tokens by providing any configured collateral token
    pub fn mint(ctx: Context<Mint>, amount: u64) -> Result<()> {
        mint_handler(ctx, amount)
    }

    /// Request to unmint abrafi tokens back to a collateral token
    /// This starts a cooldown period before the user can claim the tokens
    /// If user already has an active request, amounts are merged and cooldown is reset
    pub fn request_unmint(ctx: Context<RequestUnmint>, amount: u64) -> Result<()> {
        request_unmint_handler(ctx, amount)
    }

    /// Claim unminted tokens after the cooldown period has completed
    /// This burns the abrafi tokens and transfers the claimed tokens to the user
    /// Users can claim partial amounts, reducing the requested amount accordingly
    pub fn claim_unmint(ctx: Context<ClaimUnmint>, claim_amount: u64) -> Result<()> {
        claim_unmint_handler(ctx, claim_amount)
    }

    /// Cancel an unmint request (partial or full)
    /// This instruction can be called at any time to immediately cancel part or all of the request
    /// It reduces the requested amount and closes the account if fully canceled
    pub fn cancel_unmint(ctx: Context<CancelUnmint>, cancel_amount: u64) -> Result<()> {
        cancel_unmint_handler(ctx, cancel_amount)
    }

    /// Update the minimum amount that can be minted
    /// Only the operations authority can change the minimum mint amount
    pub fn update_minimum_mint_amount(
        ctx: Context<UpdateMinimumMintAmount>,
        new_minimum_amount: u64,
    ) -> Result<()> {
        update_minimum_mint_amount_handler(ctx, new_minimum_amount)
    }

    /// Update the minimum amount that can be unminted
    /// Only the operations authority can change the minimum unmint amount
    pub fn update_minimum_unmint_amount(
        ctx: Context<UpdateMinimumUnmintAmount>,
        new_minimum_amount: u64,
    ) -> Result<()> {
        update_minimum_unmint_amount_handler(ctx, new_minimum_amount)
    }

    /// Update the unmint cooldown period (in seconds)
    /// Only the authority can change the cooldown period
    /// Maximum cooldown period is 30 days (2,592,000 seconds = 720 hours)
    pub fn update_unmint_cooldown(
        ctx: Context<UpdateUnmintCooldown>,
        new_cooldown_seconds: i64,
    ) -> Result<()> {
        update_unmint_cooldown_handler(ctx, new_cooldown_seconds)
    }

    /// Update the request expiration period (in seconds)
    /// Only the authority can change the expiration period
    /// Minimum expiration period is 1 hour (3600 seconds)
    /// Maximum expiration period is 90 days (7,776,000 seconds = 2160 hours)
    pub fn update_request_expiration(
        ctx: Context<UpdateRequestExpiration>,
        new_expiration_seconds: i64,
    ) -> Result<()> {
        update_request_expiration_handler(ctx, new_expiration_seconds)
    }

    /// Test-only instruction to set request expiration period without bounds checks
    /// This allows testing expiration scenarios by setting expiration to zero or other values
    /// that would normally be rejected by the validation checks
    /// Only available in test environment (when compiled with test features)
    #[cfg(feature = "test")]
    pub fn set_request_expiration_for_testing(
        ctx: Context<UpdateRequestExpiration>,
        new_expiration_seconds: i64,
    ) -> Result<()> {
        set_request_expiration_for_testing_handler(ctx, new_expiration_seconds)
    }

    /// Set minting enabled status
    /// This function can only be called by the operations authority
    pub fn set_minting_enabled(ctx: Context<SetMintingEnabled>, enabled: bool) -> Result<()> {
        set_minting_enabled_handler(ctx, enabled)
    }

    /// Set mint whitelist enabled status
    /// This function can only be called by the program authority
    pub fn set_mint_whitelist_enabled(ctx: Context<SetMintWhitelistEnabled>, enabled: bool) -> Result<()> {
        set_mint_whitelist_enabled_handler(ctx, enabled)
    }

    /// Set unminting request enabled status
    /// This function can only be called by the operations authority
    pub fn set_unminting_request_enabled(ctx: Context<SetUnmintingRequestEnabled>, enabled: bool) -> Result<()> {
        set_unminting_request_enabled_handler(ctx, enabled)
    }

    /// Set unminting claim enabled status
    /// This function can only be called by the operations authority
    pub fn set_unminting_claim_enabled(ctx: Context<SetUnmintingClaimEnabled>, enabled: bool) -> Result<()> {
        set_unminting_claim_enabled_handler(ctx, enabled)
    }

    /// Freeze a token account
    /// This function can only be called by the compliance authority
    pub fn freeze_account(ctx: Context<FreezeAccount>, reason_code: u32) -> Result<()> {
        freeze_account_handler(ctx, reason_code)
    }

    /// Thaw a token account
    /// This function can only be called by the compliance authority
    pub fn thaw_account(ctx: Context<ThawAccount>) -> Result<()> {
        thaw_account_handler(ctx)
    }

    /// Remove tokens from withdrawal vault back to treasury (operations authority only)
    /// Allows the operations authority to withdraw excess funds from the withdrawal vault
    pub fn withdraw_from_vault(ctx: Context<WithdrawFromVault>, amount: u64) -> Result<()> {
        withdraw_from_vault_handler(ctx, amount)
    }

    /// Update the program authority (step 1: current authority sets pending authority)
    /// This sets a pending authority that must be finalized by the new authority within 24 hours
    pub fn update_authority(ctx: Context<UpdateAuthority>) -> Result<()> {
        update_authority_handler(ctx)
    }

    /// Finalize the program authority transfer (step 2: new authority accepts)
    /// This completes the authority transfer and must be called within 24 hours
    pub fn finalize_authority(ctx: Context<FinalizeAuthority>) -> Result<()> {
        finalize_authority_handler(ctx)
    }

    /// Test-only instruction to set pending authority expiration timestamp to a past date
    /// This allows testing authority expiration without waiting 24 hours
    /// Only available in test environment (when compiled with test features)
    #[cfg(feature = "test")]
    pub fn set_pending_authority_timestamp_for_testing(
        ctx: Context<SetPendingAuthorityTimestampForTesting>,
        new_timestamp: i64,
    ) -> Result<()> {
        set_pending_authority_timestamp_for_testing_handler(ctx, new_timestamp)
    }


    /// Update the operations authority (current authority only)
    /// This allows the current authority to transfer control to a new operations authority
    pub fn update_operations_authority(ctx: Context<UpdateAuthority>) -> Result<()> {
        update_operations_authority_handler(ctx)
    }

    /// Update the compliance authority (current authority only)
    /// This allows the current authority to transfer control to a new compliance authority
    pub fn update_compliance_authority(ctx: Context<UpdateAuthority>) -> Result<()> {
        update_compliance_authority_handler(ctx)
    }

    /// Transfer mint authority to a new authority (step 1: current authority sets pending mint authority)
    /// This sets a pending mint authority that must be finalized by the new authority
    /// This function can only be called by the program authority
    /// Requires that minting and request unminting are disabled
    /// To cancel a pending mint authority transfer, pass None for new_mint_authority
    pub fn transfer_mint_authority(ctx: Context<TransferMintAuthority>) -> Result<()> {
        transfer_mint_authority_handler(ctx)
    }

    /// Finalize the mint authority transfer (step 2: new mint authority accepts)
    /// This completes the mint authority transfer and must be called before it expires
    pub fn finalize_mint_authority(ctx: Context<FinalizeMintAuthority>) -> Result<()> {
        finalize_mint_authority_handler(ctx)
    }

    /// Test-only instruction to set pending mint authority expiration timestamp to any value
    /// This allows testing mint authority expiration scenarios without waiting
    /// Can be set to past values to test expiration or future values to test non-expiration
    /// Only available in test environment (when compiled with test features)
    #[cfg(feature = "test")]
    pub fn set_pending_mint_authority_timestamp_for_testing(
        ctx: Context<SetPendingMintAuthorityTimestampForTesting>,
        new_timestamp: i64,
    ) -> Result<()> {
        set_pending_mint_authority_timestamp_for_testing_handler(ctx, new_timestamp)
    }

    /// Add an address to the mint whitelist with an optional custom unmint cooldown.
    /// This function can only be called by the compliance authority.
    /// When custom_cooldown_seconds is provided, it takes effect 24 hours after this call;
    /// until then the global unmint_cooldown_seconds applies to this address.
    pub fn add_mint_whitelist(
        ctx: Context<AddMintWhitelist>,
        custom_cooldown_seconds: Option<i64>,
    ) -> Result<()> {
        add_mint_whitelist_handler(ctx, custom_cooldown_seconds)
    }

    /// Remove an address from the mint whitelist
    /// This function can only be called by the compliance authority
    /// The whitelist controls which addresses can use the mint and unmint operations
    pub fn remove_mint_whitelist(ctx: Context<RemoveMintWhitelist>) -> Result<()> {
        remove_mint_whitelist_handler(ctx)
    }

    /// Update token metadata (create if doesn't exist, update if exists)
    /// This function can only be called by the program authority
    /// The CPI will be signed by the state PDA since it is the mint authority
    pub fn update_token_metadata(
        ctx: Context<UpdateTokenMetadata>,
        name: String,
        symbol: String,
        metadata_uri: String,
    ) -> Result<()> {
        update_token_metadata_handler(ctx, name, symbol, metadata_uri)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // DATA STRUCTURE TESTS
    #[test]
    fn test_collateral_token_config_creation() {
        let name = "Test Token";
        let token_mint = Pubkey::new_unique();
        let treasury_token_account = Pubkey::new_unique();

        let config = state::CollateralTokenConfig {
            version: 1,
            name: name.to_string(),
            token_mint,
            treasury_token_account,
            disabled: false,
        };

        assert_eq!(config.name, name);
        assert_eq!(config.token_mint, token_mint);
        assert_eq!(config.treasury_token_account, treasury_token_account);
    }

    #[test]
    fn test_collateral_token_config_clone() {
        let name = "Test Token".to_string();
        let token_mint = Pubkey::new_unique();
        let treasury_token_account = Pubkey::new_unique();

        let config1 = state::CollateralTokenConfig {
            version: 1,
            name: name.to_string(),
            token_mint,
            treasury_token_account,
            disabled: false,
        };

        let config2 = config1.clone();

        assert_eq!(config1, config2);
        assert_eq!(config1.token_mint, config2.token_mint);
        assert_eq!(
            config1.treasury_token_account,
            config2.treasury_token_account
        );
        assert_eq!(config1.name, config2.name);
    }

    #[test]
    fn test_collateral_token_config_partial_eq() {
        let name1 = "Token 1";
        let token_mint1 = Pubkey::new_unique();
        let treasury_token_account1 = Pubkey::new_unique();
        let name2 = "Token 2";
        let token_mint2 = Pubkey::new_unique();
        let treasury_token_account2 = Pubkey::new_unique();

        let config1 = state::CollateralTokenConfig {
            version: 1,
            name: name1.to_string(),
            token_mint: token_mint1,
            treasury_token_account: treasury_token_account1,
            disabled: false,
        };

        let config2 = state::CollateralTokenConfig {
            version: 1,
            name: name1.to_string(),
            token_mint: token_mint1,
            treasury_token_account: treasury_token_account1,
            disabled: false,
        };

        let config3 = state::CollateralTokenConfig {
            version: 1,
            name: name1.to_string(),
            token_mint: token_mint2,
            treasury_token_account: treasury_token_account1,
            disabled: false,
        };

        let config4 = state::CollateralTokenConfig {
            version: 1,
            name: name1.to_string(),
            token_mint: token_mint1,
            treasury_token_account: treasury_token_account2,
            disabled: false,
        };

        let config5 = state::CollateralTokenConfig {
            version: 1,
            name: name2.to_string(),
            token_mint: token_mint1,
            treasury_token_account: treasury_token_account1,
            disabled: false,
        };

        // Same values should be equal
        assert_eq!(config1, config2);
        assert_eq!(config2, config1);

        // Different values should not be equal
        assert_ne!(config1, config3);
        assert_ne!(config1, config4);
        assert_ne!(config1, config5);
        assert_ne!(config3, config4);
    }

    #[test]
    fn test_user_unmint_details_creation() {
        let claim_token_mint = Pubkey::new_unique();
        let request_timestamp = 1640995200i64; // 2022-01-01 00:00:00 UTC
        let withdrawal_delay_end_timestamp = 1641600000i64; // 2022-01-08 00:00:00 UTC
        let request_expiration_timestamp = 1642204800i64; // 2022-01-15 00:00:00 UTC
        let unmint_details = state::UserUnmintDetails {
            version: 1,
            claim_token_mint,
            request_timestamp,
            withdrawal_delay_end_timestamp,
            request_expiration_timestamp,
            bump: 0,
        };

        assert_eq!(unmint_details.claim_token_mint, claim_token_mint);
        assert_eq!(unmint_details.request_timestamp, request_timestamp);
        assert_eq!(
            unmint_details.withdrawal_delay_end_timestamp,
            withdrawal_delay_end_timestamp
        );
        assert_eq!(
            unmint_details.request_expiration_timestamp,
            request_expiration_timestamp
        );
    }

    // HELPER FUNCTION TESTS
    #[test]
    fn test_find_token_config() {
        let mut state = state::ProgramState {
            version: 1,
            authority: Pubkey::new_unique(),
            pending_authority: Pubkey::default(),
            pending_authority_expiration_timestamp: 0,
            pending_mint_authority: Pubkey::default(), // No pending mint authority initially
            pending_mint_authority_expiration_timestamp: 0, // No pending mint authority expiration initially
            operations_authority: Pubkey::new_unique(),
            compliance_authority: Pubkey::new_unique(),
            _unused: Pubkey::default(), // Reserved field, not used
            abrafi_backed_token_mint: Pubkey::new_unique(),
            collateral_tokens: vec![
                state::CollateralTokenConfig {
                    version: 1,
                    name: "Token 1".to_string(),
                    token_mint: Pubkey::new_unique(),
                    treasury_token_account: Pubkey::new_unique(),
                    disabled: false,
                },
                state::CollateralTokenConfig {
                    version: 1,
                    name: "Token 2".to_string(),
                    token_mint: Pubkey::new_unique(),
                    treasury_token_account: Pubkey::new_unique(),
                    disabled: true, // Disabled token
                },
            ],
            state_bump: 0,
            vault_authority_bump: 0,
            is_minting_enabled: true,
            is_unminting_request_enabled: true,
            is_unminting_claim_enabled: true,
            is_mint_whitelist_enabled: true,
            minimum_mint_amount: 1000000,
            minimum_unmint_amount: 500000,
            unmint_cooldown_seconds: 7 * 24 * 3600, // 7 days in seconds (168 hours)
            request_expiration_seconds: 7 * 24 * 3600, // 7 days in seconds (168 hours)
        };

        let enabled_token = state.collateral_tokens[0].token_mint;
        let disabled_token = state.collateral_tokens[1].token_mint;
        let unconfigured_token = Pubkey::new_unique();

        // Test enabled token - should return Some(config) regardless of disabled status
        let config_opt = utils::find_token_config(&state, &enabled_token);
        assert!(config_opt.is_some());
        let config = config_opt.unwrap();
        assert_eq!(config.token_mint, enabled_token);
        assert_eq!(config.disabled, false);

        // Test disabled token - should return Some(config) even though it's disabled
        let config_opt = utils::find_token_config(&state, &disabled_token);
        assert!(config_opt.is_some());
        let config = config_opt.unwrap();
        assert_eq!(config.token_mint, disabled_token);
        assert_eq!(config.disabled, true);

        // Test unconfigured token - should return None
        let config_opt = utils::find_token_config(&state, &unconfigured_token);
        assert!(config_opt.is_none());

        // Test with empty collateral tokens
        state.collateral_tokens.clear();
        let config_opt = utils::find_token_config(&state, &enabled_token);
        assert!(config_opt.is_none());
    }

    #[test]
    fn test_find_token_config_mut() {
        let mut state = state::ProgramState {
            version: 1,
            authority: Pubkey::new_unique(),
            pending_authority: Pubkey::default(),
            pending_authority_expiration_timestamp: 0,
            pending_mint_authority: Pubkey::default(), // No pending mint authority initially
            pending_mint_authority_expiration_timestamp: 0, // No pending mint authority expiration initially
            operations_authority: Pubkey::new_unique(),
            compliance_authority: Pubkey::new_unique(),
            _unused: Pubkey::default(), // Reserved field, not used
            abrafi_backed_token_mint: Pubkey::new_unique(),
            collateral_tokens: vec![
                state::CollateralTokenConfig {
                    version: 1,
                    name: "Token 1".to_string(),
                    token_mint: Pubkey::new_unique(),
                    treasury_token_account: Pubkey::new_unique(),
                    disabled: false,
                },
                state::CollateralTokenConfig {
                    version: 1,
                    name: "Token 2".to_string(),
                    token_mint: Pubkey::new_unique(),
                    treasury_token_account: Pubkey::new_unique(),
                    disabled: true,
                },
            ],
            state_bump: 0,
            vault_authority_bump: 0,
            is_minting_enabled: true,
            is_unminting_request_enabled: true,
            is_unminting_claim_enabled: true,
            is_mint_whitelist_enabled: true,
            minimum_mint_amount: 1000000,
            minimum_unmint_amount: 500000,
            unmint_cooldown_seconds: 7 * 24 * 3600,
            request_expiration_seconds: 7 * 24 * 3600,
        };

        let enabled_token = state.collateral_tokens[0].token_mint;
        let disabled_token = state.collateral_tokens[1].token_mint;
        let unconfigured_token = Pubkey::new_unique();

        // Test enabled token - should return Some(mut config)
        let config_opt = utils::find_token_config_mut(&mut state, &enabled_token);
        assert!(config_opt.is_some());
        let config_ref = config_opt.as_ref().unwrap();
        assert_eq!(config_ref.token_mint, enabled_token);
        assert_eq!(config_ref.disabled, false);

        // Test that we can modify the config
        if let Some(config) = config_opt {
            config.disabled = true;
            config.name = "Modified Token".to_string();
        }
        // Verify the modification persisted
        let config_opt = utils::find_token_config(&state, &enabled_token);
        assert!(config_opt.is_some());
        let config = config_opt.unwrap();
        assert_eq!(config.disabled, true);
        assert_eq!(config.name, "Modified Token");

        // Test disabled token - should return Some(mut config)
        let config_opt = utils::find_token_config_mut(&mut state, &disabled_token);
        assert!(config_opt.is_some());
        let config_ref = config_opt.as_ref().unwrap();
        assert_eq!(config_ref.token_mint, disabled_token);
        assert_eq!(config_ref.disabled, true);

        // Test unconfigured token - should return None
        let config_opt = utils::find_token_config_mut(&mut state, &unconfigured_token);
        assert!(config_opt.is_none());

        // Test with empty collateral tokens
        state.collateral_tokens.clear();
        let config_opt = utils::find_token_config_mut(&mut state, &enabled_token);
        assert!(config_opt.is_none());
    }

    #[test]
    fn test_find_enabled_token_config() {
        let mut state = state::ProgramState {
            version: 1,
            authority: Pubkey::new_unique(),
            pending_authority: Pubkey::default(),
            pending_authority_expiration_timestamp: 0,
            pending_mint_authority: Pubkey::default(), // No pending mint authority initially
            pending_mint_authority_expiration_timestamp: 0, // No pending mint authority expiration initially
            operations_authority: Pubkey::new_unique(),
            compliance_authority: Pubkey::new_unique(),
            _unused: Pubkey::default(), // Reserved field, not used
            abrafi_backed_token_mint: Pubkey::new_unique(),
            collateral_tokens: vec![
                state::CollateralTokenConfig {
                    version: 1,
                    name: "Token 1".to_string(),
                    token_mint: Pubkey::new_unique(),
                    treasury_token_account: Pubkey::new_unique(),
                    disabled: false,
                },
                state::CollateralTokenConfig {
                    version: 1,
                    name: "Token 2".to_string(),
                    token_mint: Pubkey::new_unique(),
                    treasury_token_account: Pubkey::new_unique(),
                    disabled: true, // Disabled token
                },
            ],
            state_bump: 0,
            vault_authority_bump: 0,
            is_minting_enabled: true,
            is_unminting_request_enabled: true,
            is_unminting_claim_enabled: true,
            is_mint_whitelist_enabled: true,
            minimum_mint_amount: 1000000,
            minimum_unmint_amount: 500000,
            unmint_cooldown_seconds: 7 * 24 * 3600, // 7 days in seconds (168 hours)
            request_expiration_seconds: 7 * 24 * 3600, // 7 days in seconds (168 hours)
        };

        let enabled_token = state.collateral_tokens[0].token_mint;
        let disabled_token = state.collateral_tokens[1].token_mint;
        let unconfigured_token = Pubkey::new_unique();

        // Test enabled token - should return Some(config)
        let config_opt = utils::find_enabled_token_config(&state, &enabled_token);
        assert!(config_opt.is_some());
        let config = config_opt.unwrap();
        assert_eq!(config.token_mint, enabled_token);
        assert_eq!(config.disabled, false);

        // Test disabled token - should return None
        let config_opt = utils::find_enabled_token_config(&state, &disabled_token);
        assert!(config_opt.is_none());

        // Test unconfigured token - should return None
        let config_opt = utils::find_enabled_token_config(&state, &unconfigured_token);
        assert!(config_opt.is_none());

        // Test with empty collateral tokens
        state.collateral_tokens.clear();
        let config_opt = utils::find_enabled_token_config(&state, &enabled_token);
        assert!(config_opt.is_none());
    }

    // PDA DERIVATION TESTS
    #[test]
    fn test_pda_derivation() {
        let program_id = Pubkey::new_unique();
        let user_key = Pubkey::new_unique();
        let state_key = Pubkey::new_unique();

        // Test state PDA derivation
        let (state_address, _state_bump) =
            Pubkey::find_program_address(&[constants::STATE_SEED], &program_id);

        // Test vault authority PDA derivation
        let (vault_authority, _vault_bump) = Pubkey::find_program_address(
            &[constants::VAULT_AUTHORITY_SEED, state_key.as_ref()],
            &program_id,
        );

        // Test unmint details PDA derivation
        let claim_mint = Pubkey::new_unique();
        let (unmint_details, _unmint_bump) = Pubkey::find_program_address(
            &[constants::UNMINT_DETAILS_SEED, user_key.as_ref(), claim_mint.as_ref()],
            &program_id,
        );

        // Verify that all PDAs are unique
        let pdas = vec![state_address, vault_authority, unmint_details];

        for i in 0..pdas.len() {
            for j in (i + 1)..pdas.len() {
                assert_ne!(pdas[i], pdas[j], "PDAs should be unique");
            }
        }
    }

    // SERIALIZATION TESTS
    #[test]
    fn test_collateral_token_config_serialization() {
        let config = state::CollateralTokenConfig {
            version: 1,
            name: "Test Token".to_string(),
            token_mint: Pubkey::new_unique(),
            treasury_token_account: Pubkey::new_unique(),
            disabled: false,
        };

        // Test that the struct can be serialized and deserialized
        let serialized = config.try_to_vec().expect("Failed to serialize");
        let deserialized = state::CollateralTokenConfig::try_from_slice(&serialized)
            .expect("Failed to deserialize");

        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_user_unmint_details_serialization() {
        let unmint_details = state::UserUnmintDetails {
            version: 1,
            claim_token_mint: Pubkey::new_unique(),
            request_timestamp: 1640995200i64,
            withdrawal_delay_end_timestamp: 1641600000i64,
            request_expiration_timestamp: 1642204800i64,
            bump: 0,
        };

        // Test that the struct can be serialized and deserialized
        let serialized = unmint_details.try_to_vec().expect("Failed to serialize");
        let deserialized: state::UserUnmintDetails =
            state::UserUnmintDetails::try_from_slice(&serialized).expect("Failed to deserialize");

        assert_eq!(unmint_details.version, deserialized.version);
        assert_eq!(
            unmint_details.claim_token_mint,
            deserialized.claim_token_mint
        );
        assert_eq!(
            unmint_details.request_timestamp,
            deserialized.request_timestamp
        );
        assert_eq!(
            unmint_details.withdrawal_delay_end_timestamp,
            deserialized.withdrawal_delay_end_timestamp
        );
        assert_eq!(
            unmint_details.request_expiration_timestamp,
            deserialized.request_expiration_timestamp
        );
    }

    // EDGE CASE TESTS
    #[test]
    fn test_maximum_values() {
        // Test with maximum values
        let max_i64 = i64::MAX;

        let unmint_details = state::UserUnmintDetails {
            version: 1,
            claim_token_mint: Pubkey::new_unique(),
            request_timestamp: max_i64,
            withdrawal_delay_end_timestamp: max_i64,
            request_expiration_timestamp: max_i64,
            bump: u8::MAX,
        };

        assert_eq!(unmint_details.request_timestamp, max_i64);
        assert_eq!(unmint_details.withdrawal_delay_end_timestamp, max_i64);
        assert_eq!(unmint_details.request_expiration_timestamp, max_i64);
    }

    #[test]
    fn test_zero_values() {
        // Test with zero values
        let zero_i64 = 0i64;

        let unmint_details = state::UserUnmintDetails {
            version: 1,
            claim_token_mint: Pubkey::new_unique(),
            request_timestamp: zero_i64,
            withdrawal_delay_end_timestamp: zero_i64,
            request_expiration_timestamp: zero_i64,
            bump: 0,
        };

        assert_eq!(unmint_details.request_timestamp, zero_i64);
        assert_eq!(unmint_details.withdrawal_delay_end_timestamp, zero_i64);
        assert_eq!(unmint_details.request_expiration_timestamp, zero_i64);
    }

    #[test]
    fn test_negative_timestamps() {
        // Test with negative timestamps (edge case)
        let negative_timestamp = -1640995200i64;

        let unmint_details = state::UserUnmintDetails {
            version: 1,
            claim_token_mint: Pubkey::new_unique(),
            request_timestamp: negative_timestamp,
            withdrawal_delay_end_timestamp: negative_timestamp,
            request_expiration_timestamp: negative_timestamp,
            bump: 0,
        };

        assert_eq!(unmint_details.request_timestamp, negative_timestamp);
        assert_eq!(
            unmint_details.withdrawal_delay_end_timestamp,
            negative_timestamp
        );
        assert_eq!(
            unmint_details.request_expiration_timestamp,
            negative_timestamp
        );
    }
}
