# Contributing to dig-constants

Thanks for your interest in improving dig-constants. This crate holds network
constants — genesis challenge, consensus parameters, and network configuration
for the DIG L2 blockchain — so accuracy matters.

## What this crate is

`dig-constants` defines network parameters (DIG L2 mainnet/testnet and Chia L1
references) as a minimal, lightweight building block. Any DIG crate can import
these constants without pulling in heavy dependencies like the CLVM engine.

## Reporting an issue

Open an issue on [GitHub](https://github.com/DIG-Network/dig-constants/issues).
For a report to be actionable, include:

- What constant or behavior is wrong, with the actual and expected values
- Where you discovered it (which crate/binary, which operation)
- If it's a consensus/genesis value, include a reference (on-chain hash, another
  implementation, etc.)

## Prerequisites

- [Rust](https://rustup.rs), **1.75.0 or later** (from `Cargo.toml`'s `rust-version` field)

## Build & test

This is a single crate with no workspace:

```sh
# Build the crate
cargo build

# Run the test suite
cargo test
```

The test suite includes consensus-constant KATs (known-answer tests) that validate
against `chia-sdk-types` and verify bech32 encoding of the treasury address.

## The gate (must pass before a PR is merged)

CI runs these checks on every PR (`.github/workflows/ci.yml`); run them locally
first:

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo nextest run --workspace --all-features --retries 2
cargo llvm-cov --workspace --all-features --summary-only
```

The first three are required checks; the coverage report is measure-only (informational).

## Commit conventions

- Use Conventional Commits: `type(scope): summary`
  - `type` ∈ `feat|fix|docs|style|refactor|perf|test|build|ci|chore`
  - Examples: `feat(constants): add new consensus param`, `fix(lib): correct genesis value`, `docs: update CONTRIBUTING.md`
  - Keep one logical change per commit where practical
- **Every commit must include a `Co-Authored-By` trailer** when co-authored or
  machine-generated. Format:
  ```
  docs: add CONTRIBUTING.md

  Co-Authored-By: Claude <noreply@anthropic.com>
  ```

## Pull requests

1. Branch from `main`.
2. Make the gate green locally.
3. Bump the version in `Cargo.toml` (and `Cargo.lock` if present) as part of your
   commit; `patch` for fixes/docs, `minor` for new constants, `major` for
   breaking changes. State the rationale in the PR.
4. Open a PR with a clear description of the change and its rationale.
5. `main` is protected: PR required, all checks green, zero unresolved threads.

## Where things live

The crate is organized around network targets and consensus:

- **`DIG_MAINNET` / `DIG_TESTNET`** — DIG L2 network constants
- **`CHIA_L1_MAINNET_AGG_SIG_ME` / `CHIA_L1_TESTNET11_AGG_SIG_ME`** — Chia L1
  genesis challenges (used by DIG wallet code signing L1 spends; **distinct from
  DIG L2 genesis**)
- **`NetworkConstants`** — Wrapper around `chia-consensus::ConsensusConstants`,
  the main type
- **`src/lib.rs`** — All exports

## Security

For anything security-relevant, report the issue privately to the maintainer
rather than opening a public GitHub issue. The consensus rules this crate
encodes are central to L2 validity.
