use anchor_lang::prelude::*;
use anchor_spl::token::Mint;
use anchor_spl::metadata::{
    create_metadata_accounts_v3, update_metadata_accounts_v2, CreateMetadataAccountsV3,
    Metadata, UpdateMetadataAccountsV2,
};
use anchor_spl::metadata::mpl_token_metadata::types::DataV2;

use crate::constants::*;
use crate::error::ErrorCode;
use crate::events::TokenMetadataUpdated;
use crate::state::*;

/// Update token metadata instruction accounts
#[derive(Accounts)]
pub struct UpdateTokenMetadata<'info> {
    /// Program state account
    #[account(
        seeds = [STATE_SEED],
        bump = state.state_bump,
        has_one = authority,
        has_one = liquid_staking_token_mint,
    )]
    pub state: Account<'info, ProgramState>,

    /// Authority that can update program configuration
    #[account(mut)]
    pub authority: Signer<'info>,

    /// The liquid staking token mint
    #[account(mut)]
    pub liquid_staking_token_mint: Account<'info, Mint>,

    /// Metadata PDA account (derived from mint)
    /// CHECK: Validated by deriving PDA and checking program ID
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,

    /// Metaplex Token Metadata program
    pub token_metadata_program: Program<'info, Metadata>,

    /// System program
    pub system_program: Program<'info, System>,

    /// Rent sysvar (required for creating metadata account)
    /// CHECK: Validated by Anchor
    pub rent: Sysvar<'info, Rent>,
}

/// Update token metadata (create if doesn't exist, update if exists)
/// This instruction requires program authority and uses state PDA as update authority
/// The metadata_uri should point to a JSON file containing the full metadata including
/// description and image URL, following the Metaplex Token Metadata standard.
pub fn update_token_metadata_handler(
    ctx: Context<UpdateTokenMetadata>,
    name: String,
    symbol: String,
    metadata_uri: String,
) -> Result<()> {
    // Validate string lengths to prevent excessive data
    require!(!name.is_empty() && name.len() <= 32, ErrorCode::InvalidMetadata);
    require!(!symbol.is_empty() && symbol.len() <= 10, ErrorCode::InvalidMetadata);
    require!(!metadata_uri.is_empty() && metadata_uri.len() <= 200, ErrorCode::InvalidMetadata);

    // Extract state_bump before any mutable borrows
    let state_bump = ctx.accounts.state.state_bump;
    let mint_key = ctx.accounts.liquid_staking_token_mint.key();

    // Derive metadata PDA to verify it matches
    // Metadata PDA is derived from: ["metadata", TOKEN_METADATA_PROGRAM_ID, mint_key]
    let token_metadata_program_id = Metadata::id();
    let metadata_seeds = &[
        b"metadata",
        token_metadata_program_id.as_ref(),
        mint_key.as_ref(),
    ];
    let (expected_metadata_key, _metadata_bump) =
        Pubkey::find_program_address(metadata_seeds, &token_metadata_program_id);
    require!(
        expected_metadata_key == ctx.accounts.metadata.key(),
        ErrorCode::InvalidMintAccount
    );

    // Check if metadata account exists
    let metadata_exists = !ctx.accounts.metadata.data_is_empty()
        && ctx.accounts.metadata.owner == &token_metadata_program_id;

    // Prepare signer seeds for state PDA
    let state_seeds = &[STATE_SEED, &[state_bump]];
    let signer_seeds = &[&state_seeds[..]];

    // Clone values for event emission before they're moved into data_v2
    let event_name = name.clone();
    let event_symbol = symbol.clone();
    let event_metadata_uri = metadata_uri.clone();

    // Create DataV2 struct with metadata (used for both create and update)
    // The uri field points to a JSON metadata file that should contain description and image
    let data_v2 = DataV2 {
        name,
        symbol,
        uri: metadata_uri,
        seller_fee_basis_points: 0,
        creators: None,
        collection: None,
        uses: None,
    };

    if metadata_exists {
        // Update existing metadata using UpdateMetadataAccountsV2
        // Set up CPI context for UpdateMetadataAccountsV2
        let cpi_accounts = UpdateMetadataAccountsV2 {
            metadata: ctx.accounts.metadata.to_account_info(),
            update_authority: ctx.accounts.state.to_account_info(),
        };

        let cpi_program = ctx.accounts.token_metadata_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);

        // Update metadata: keep current update authority, update data, keep primary_sale_happened unchanged, set is_mutable to true
        update_metadata_accounts_v2(
            cpi_ctx,
            None,              // new_update_authority: None (keep current)
            Some(data_v2),     // data: Some(DataV2) with updated values
            None,              // primary_sale_happened: None (keep unchanged)
            Some(true),        // is_mutable: Some(true)
        )?;
    } else {
        // Create new metadata account using CreateMetadataAccountsV3

        // Set up CPI context for CreateMetadataAccountsV3
        let cpi_accounts = CreateMetadataAccountsV3 {
            metadata: ctx.accounts.metadata.to_account_info(),
            mint: ctx.accounts.liquid_staking_token_mint.to_account_info(),
            mint_authority: ctx.accounts.state.to_account_info(),
            payer: ctx.accounts.authority.to_account_info(),
            update_authority: ctx.accounts.state.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
            rent: ctx.accounts.rent.to_account_info(),
        };

        let cpi_program = ctx.accounts.token_metadata_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);

        // Create metadata account: is_mutable = true, update_authority_is_signer = false (state PDA will sign via CpiContext)
        create_metadata_accounts_v3(cpi_ctx, data_v2, true, false, None)?;
    }

    // Emit event for metadata update/create
    emit!(TokenMetadataUpdated {
        version: 1,
        liquid_staking_token_mint: ctx.accounts.liquid_staking_token_mint.key(),
        metadata_account: ctx.accounts.metadata.key(),
        name: event_name,
        symbol: event_symbol,
        metadata_uri: event_metadata_uri,
        was_created: !metadata_exists,
    });

    Ok(())
}
