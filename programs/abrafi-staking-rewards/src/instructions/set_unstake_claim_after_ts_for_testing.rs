use anchor_lang::prelude::*;
use crate::constants::UNSTAKE_REQUEST_SEED;
use crate::state::UnstakeRequest;

#[cfg(feature = "test")]
#[derive(Accounts)]
pub struct SetUnstakeClaimAfterTsForTesting<'info> {
    #[account(
        mut,
        seeds = [UNSTAKE_REQUEST_SEED, user.key().as_ref()],
        bump = unstake_request.bump,
        has_one = user,
    )]
    pub unstake_request: Account<'info, UnstakeRequest>,
    pub user: Signer<'info>,
}

#[cfg(feature = "test")]
pub fn set_unstake_claim_after_ts_for_testing_handler(
    ctx: Context<SetUnstakeClaimAfterTsForTesting>,
    new_ts: i64,
) -> Result<()> {
    ctx.accounts.unstake_request.claim_after_ts = new_ts;
    Ok(())
}
