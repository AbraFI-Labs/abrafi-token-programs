// Program IDs for each token deployment.
// Replace each compile_error! with the real declare_id! once the program ID is known.

#[cfg(feature = "usdaf")]
declare_id!("9cYNMhVvGC2LhgJYZavKVWYZ8T1oqSHYt5ehTddP51yC");

#[cfg(feature = "solaf")]
compile_error!("abrafi-staking-liquid SOLAF program ID not set — replace this line with: declare_id!(\"PROGRAM_ID\");");

#[cfg(not(any(feature = "usdaf", feature = "solaf")))]
compile_error!("Specify a token feature: --features usdaf  or  --features solaf");
