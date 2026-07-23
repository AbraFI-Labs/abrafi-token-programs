// Program IDs for each token deployment.
// Replace each compile_error! with the real declare_id! once the program ID is known.

#[cfg(feature = "usdaf")]
declare_id!("H9Cyw2RRHStwtKTAgGPm3nU48xwQ2Scy8u2gjg82dyQQ");

#[cfg(feature = "solaf")]
compile_error!("abrafi-backed-token SOLAF program ID not set — replace this line with: declare_id!(\"PROGRAM_ID\");");

#[cfg(not(any(feature = "usdaf", feature = "solaf")))]
compile_error!("Specify a token feature: --features usdaf  or  --features solaf");
