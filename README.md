# AbraFi Token Programs

Solana smart contracts for the AbraFi protocol, built with [Anchor](https://www.anchor-lang.com/) 0.32.x.

## Programs

| Program                    | Description                                                                                                      |
| -------------------------- | ---------------------------------------------------------------------------------------------------------------- |
| `abrafi-backed-token`      | Generalised collateral-backed token program, redeployable for multiple tokens (e.g. USDAF, SOLAF).              |
| `abrafi-staking-liquid`    | Generalised liquid staking program. Users stake an underlying token and receive a yield-bearing receipt token.   |
| `abrafi-staking-rewards`   | In-kind claimable rewards staking. Stake a token, earn more of the same token.                                   |
| `abrafi-yield-router`      | Receives yield tokens and distributes them proportionally to registered recipients based on live token balances. |

## Building

Each program supports multiple token deployments selected via a Cargo feature flag.

```bash
# Install Rust (see https://rustup.rs)
# Install Solana CLI (see https://docs.solanalabs.com/cli/install)
# Install Anchor CLI (see https://www.anchor-lang.com/docs/installation)

# Build a specific program for a specific token deployment
cargo build-sbf -- -p abrafi-backed-token    --features usdaf,usdaf-compat
cargo build-sbf -- -p abrafi-staking-liquid  --features usdaf,susdaf-compat
cargo build-sbf -- -p abrafi-staking-rewards --features solaf
cargo build-sbf -- -p abrafi-yield-router    --features usdaf
```

## Verified Builds

All programs deployed on mainnet are built using [solana-verify](https://github.com/Ellipsis-Labs/solana-verify) for reproducible, verifiable builds. You can verify any deployed program matches this source code:

```bash
# Install solana-verify
cargo install solana-verify

# Build reproducibly
solana-verify build --library-name <program_name> -- --features <token>

# Verify against a deployed program
solana-verify verify-from-repo \
  -u https://api.mainnet-beta.solana.com \
  --program-id <PROGRAM_ID> \
  https://github.com/AbraFI-Labs/abrafi-token-programs \
  --commit-hash <COMMIT_HASH> \
  --library-name <program_name> \
  --mount-path programs/<program_name>
```

Program IDs and verified commit hashes for each release are published on the [Releases](https://github.com/AbraFI-Labs/abrafi-token-programs/releases) page.

## Program Upgrade Governance

Program upgrades are gated behind a [Squads v4](https://squads.so) multisig. No single keypair can upgrade a program unilaterally — a threshold of members must approve each proposal before the upgrade executes.

## Security

To report a vulnerability, please email **<security@abrafi.io>**.

Do not open a public GitHub issue for security vulnerabilities.

## License

Licensed under the [Business Source License 1.1](./LICENSE). Use is limited to
non-production purposes unless you obtain a commercial license. Each release
automatically converts to the [Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0)
two years after that release is published.
