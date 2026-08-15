// SPDX-License-Identifier: BUSL-1.1
// Copyright (C) 2026 AbraFi Ltd., Business Source License 1.1.
// Change Date: 2nd anniversary of this version's publication.
// Change License: Apache-2.0. See LICENSE at repo root.

// Program IDs for each token deployment.
// Replace each compile_error! with the real declare_id! once the program ID is known.

#[cfg(feature = "usdaf")]
compile_error!("abrafi-yield-router USDAF program ID not set — replace this line with: declare_id!(\"PROGRAM_ID\");");

#[cfg(feature = "solaf")]
compile_error!("abrafi-yield-router SOLAF program ID not set — replace this line with: declare_id!(\"PROGRAM_ID\");");

#[cfg(not(any(feature = "usdaf", feature = "solaf")))]
compile_error!("Specify a token feature: --features usdaf  or  --features solaf");
