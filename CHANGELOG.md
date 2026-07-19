# Changelog

All notable changes to this project are documented here.
This project adheres to [Semantic Versioning](https://semver.org) and
[Conventional Commits](https://www.conventionalcommits.org).

## [0.6.0] - 2026-07-19

### Features
- Add canonical DIG treasury recipient constant (#7)

## [0.5.1] - 2026-07-19

### Features
- **constants:** Add Chia-L1 mainnet/testnet11 AGG_SIG_ME constants (#6)

## [0.5.0] - 2026-07-18

### Features
- **dig-constants:** Add canonical DIG_ASSET_ID ($DIG CAT tail) (#971) (#5)

## [0.4.0] - 2026-07-17

### Features
- Set canonical non-zero genesis challenges and relay :443 (#4)

## [0.3.0] - 2026-07-13

### Features
- Add DIG_NODE_PORT constant for client->node localhost connection (#3)

## [0.2.2] - 2026-07-12

### CI
- Add flaky-test management (#489) (#2)

## [0.2.1] - 2026-07-04

### CI
- Enforce version increment in PRs (package.json / Cargo.toml)- Enforce Conventional Commits with commitlint on PRs- Enforce Conventional Commits with commitlint on PRs- Release automation (git-cliff changelog + tag on merge); publish is manual workflow_dispatch (#230)- Re-arm crates.io auto-publish on version tag (token in org secrets; auto-publish-everything #230)- Add PR quality gates (fmt/clippy/test/build) [#230] (#1)

### Chores
- **changelog:** Add git-cliff config for Conventional-Commit changelog

## [0.1.0] - 2026-04-13

### Features
- **constants:** Implement CON-005 — AGG_SIG additional data derivation

### Chores
- Add .gitignore, remove target/ from tracking- Add crates.io publish workflow and repository field


