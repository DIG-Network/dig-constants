# dig-constants — Normative Specification

This document is the authoritative contract for the `dig-constants` Rust crate: the DIG
Network's shared network-constants library. It states what the crate provides and what
implementations and consumers MUST do. The key words MUST, MUST NOT, SHOULD, and MAY are
to be interpreted as described in RFC 2119.

Crate: `dig-constants` (import name `dig_constants`) · License: MIT ·
Edition 2021 · MSRV 1.75.0.

---

## 1. Scope and purpose

1.1. `dig-constants` defines the network parameters of the DIG L2 blockchain — the genesis
challenge, the AGG_SIG additional-data domain-separation values, CLVM cost limits, and the
canonical NAT-traversal relay endpoint — as compile-time constants.

1.2. The crate exists so that ANY DIG crate can import network constants without pulling in
the CLVM engine or other heavy dependencies. Its dependency set is deliberately minimal:
`chia-consensus` and `chia-protocol` (both pinned to the `0.26` line) and `hex-literal`.
Implementations MUST NOT add heavyweight dependencies (CLVM execution, networking, async
runtimes) to this crate.

1.3. This crate is a leaf library: it performs no I/O, holds no state, and has no runtime
configuration. Every exported value is a `const` or a pure accessor over one.

## 2. Public API surface

The crate exports exactly the following items. This surface is a contract: removing or
changing the signature or semantics of any item is a breaking change and MUST be
accompanied by a semver-major version bump.

| Item | Kind | Meaning |
|---|---|---|
| `NetworkConstants` | `struct` (Debug, Clone) | Opaque wrapper around `chia_consensus::consensus_constants::ConsensusConstants` |
| `NetworkConstants::consensus()` | `fn(&self) -> &ConsensusConstants` | The full underlying constants, for direct use with `chia-consensus` functions (`run_spendbundle()`, `validate_clvm_and_signature()`, …) |
| `NetworkConstants::genesis_challenge()` | `fn(&self) -> Bytes32` | The network's genesis challenge |
| `NetworkConstants::agg_sig_me_additional_data()` | `fn(&self) -> Bytes32` | AGG_SIG_ME domain-separation value (§4) |
| `NetworkConstants::max_block_cost_clvm()` | `fn(&self) -> u64` | Maximum CLVM cost per block (§5) |
| `NetworkConstants::cost_per_byte()` | `fn(&self) -> u64` | CLVM cost per byte of generator program (§5) |
| `NetworkConstants::max_coin_amount()` | `fn(&self) -> u64` | Maximum coin amount (`u64::MAX`) |
| `DIG_MAINNET` | `pub const NetworkConstants` | DIG mainnet parameters (§3, §5) |
| `DIG_TESTNET` | `pub const NetworkConstants` | DIG testnet parameters (§3, §5) |
| `DIG_RELAY_URL` | `pub const &str` | Canonical NAT-traversal relay endpoint (§6) |
| `DIG_NODE_PORT` | `pub const u16` | Default localhost port for client→node connection (§7) |

2.1. `NetworkConstants`'s field is private. Consumers MUST reach the underlying
`ConsensusConstants` only via `consensus()`; the wrapper's accessors are the stable
names for the commonly needed fields.

2.2. `DIG_MAINNET` and `DIG_TESTNET` are `const` items: their values are fixed at compile
time and identical in every build of a given crate version.

## 3. Networks: mainnet / testnet split

3.1. The crate defines exactly two networks. They differ ONLY in genesis challenge and the
AGG_SIG additional-data values derived from it (§4); every other parameter (§5) is
identical between the two.

| Network | Genesis challenge (32 bytes, hex) | Derivation preimage |
|---|---|---|
| `DIG_MAINNET` | `3c52e26fc23988f82b521880b15f39954c574c1adb035f54591b72055f6956be` | `sha256(b"DIG_MAINNET:genesis:v1")` |
| `DIG_TESTNET` | `088c18d6b7859d885dc2f03166e862c958f74b63b6353c3df71d103b9b806c3b` | `sha256(b"DIG_TESTNET:genesis:v1")` |

3.2. **Canonical, deterministic, pre-launch value.** Each genesis challenge is derived
reproducibly as `sha256` of a fixed documented preimage (above) — never random bytes — so it
is non-zero (the gossip `network_id` gate rejects an all-zero id) and any party can reproduce
it. This is the PRE-LAUNCH canonical value. Per the ecosystem's pre-release status it is
revisable at true mainnet launch: if re-anchored, bump the preimage version (`:v2`) and
recompute every `agg_sig_*_additional_data` value per §4. Consumers MUST NOT treat signatures
or coins bound to this pre-launch domain as launch-final network state.

3.3. A transaction signed for one network is invalid on the other: because the AGG_SIG
additional data differs per network (§4), BLS signatures do not verify across the
mainnet/testnet boundary. Consumers MUST select the network by choosing the constant
(`DIG_MAINNET` vs `DIG_TESTNET`) and MUST NOT mix values from the two.

## 4. AGG_SIG additional-data derivation (normative rule)

4.1. DIG follows the Chia L1 derivation rule for the per-condition AGG_SIG domain
separators (reference: `chia-blockchain` `chia/consensus/condition_tools.py`, lines 58–71):

- `AGG_SIG_ME` additional data **is the genesis challenge itself** (no hashing).
- Every other `AGG_SIG_*` variant's additional data is
  **`sha256(genesis_challenge || opcode_byte)`**, where `opcode_byte` is the single-byte
  CLVM condition opcode of that variant.

4.2. Opcode bytes:

| Condition | Opcode byte |
|---|---|
| `AGG_SIG_PARENT` | 43 |
| `AGG_SIG_PUZZLE` | 44 |
| `AGG_SIG_AMOUNT` | 45 |
| `AGG_SIG_PUZZLE_AMOUNT` | 46 |
| `AGG_SIG_PARENT_AMOUNT` | 47 |
| `AGG_SIG_PARENT_PUZZLE` | 48 |

4.3. The values baked into this crate MUST equal the rule in §4.1 applied to the network's
genesis challenge. The current values (which do satisfy the rule for the §3 genesis
challenges) are:

**DIG mainnet** (genesis = `3c52e26f…56be`):

| Field | Value (hex) |
|---|---|
| `agg_sig_me_additional_data` | `3c52e26fc23988f82b521880b15f39954c574c1adb035f54591b72055f6956be` |
| `agg_sig_parent_additional_data` | `f9d47f4a0151514b0aedfb9601e9518a7bc62bd2f3c3ad22df242bc8756fce65` |
| `agg_sig_puzzle_additional_data` | `d1d4ec69b8ace48f94d9c0896d15ee8d017f1803ae57be045e9e043f7b280bf4` |
| `agg_sig_amount_additional_data` | `3a5ca9817b7e8f9f497c48f90048458fa4bb296c15bd5d0c25fcbec4a40b8cc5` |
| `agg_sig_puzzle_amount_additional_data` | `72376495dda328e6621894099949090eccd04e2768fa2a0af4f25ca36bbe5079` |
| `agg_sig_parent_amount_additional_data` | `783345440738d69ce52bafcfa4d6fbef1ed810e217c35288962ec1b3e42136f3` |
| `agg_sig_parent_puzzle_additional_data` | `dcdfb533e4d4ccb7f0539ca608e0e04b4054b19993ca2eafe86dd93b3df05b44` |

**DIG testnet** (genesis = `088c18d6…6c3b`):

| Field | Value (hex) |
|---|---|
| `agg_sig_me_additional_data` | `088c18d6b7859d885dc2f03166e862c958f74b63b6353c3df71d103b9b806c3b` |
| `agg_sig_parent_additional_data` | `85b3963bdeb9848af970a9bbd1d36809ae41491ffd67aee7f27e8883936d495c` |
| `agg_sig_puzzle_additional_data` | `66aba1939e128e1465d58fde414325630e891747c1428d76ebce193cbe966301` |
| `agg_sig_amount_additional_data` | `eccab86920a6d982a68898b2dcb7c150383529fcd532fe84c693fb4592c38ae3` |
| `agg_sig_puzzle_amount_additional_data` | `eb088fad0d4caba66e29130fb07407e60a7545d035d19a188fef0855c874084e` |
| `agg_sig_parent_amount_additional_data` | `232aec0a351ba4936b04920e074aebcc621a458f6b1461c4b28c658552f2f35d` |
| `agg_sig_parent_puzzle_additional_data` | `96263ac395703ab9b3b0f0587e79185f4a9898574a28b4491015ddcf9d321873` |

4.4. **Security property.** These values are the BLS signature domain separators for DIG:
a signer commits to `message || coin-binding || additional_data`, so a signature made for
DIG is not replayable on Chia L1 (or any other chain whose genesis challenge differs) and
not replayable across the DIG mainnet/testnet boundary. Any change to a genesis challenge
without recomputing ALL derived values breaks signature validation network-wide and MUST
NOT ship.

## 5. Consensus parameters (both networks)

5.1. **DIG-specific limits.** These are normative for DIG L2 block and spend validation:

| Parameter | Value | Notes |
|---|---|---|
| `max_block_cost_clvm` | `11_000_000_000` | Maximum CLVM cost per block (same value as Chia L1) |
| `cost_per_byte` | `12_000` | CLVM cost charged per byte of generator program |
| `max_coin_amount` | `u64::MAX` | Maximum single-coin amount |
| `max_generator_size` | `1_000_000` | Maximum block-generator program size (bytes) |
| `max_generator_ref_list_size` | `512` | Maximum back-reference list length |
| `hard_fork_height` / `hard_fork2_height` | `0` | All Chia consensus-rule hard forks active from block 0 — DIG L2 always uses the latest CLVM/consensus rules |
| `genesis_pre_farm_pool_puzzle_hash` / `genesis_pre_farm_farmer_puzzle_hash` | 32 zero bytes | DIG L2 has no pre-farm |

5.2. **Proof-of-space / VDF fields.** DIG L2 does not use Chia's proof-of-space/VDF
consensus. The remaining `ConsensusConstants` fields (slot/sub-slot geometry, difficulty,
plot filters and sizes, VDF discriminant size, weight-proof parameters, etc.) are populated
with valid Chia-shaped values ONLY because `ConsensusConstants` is passed whole to
`chia-consensus` validation functions. Notably, all `plot_filter_*_height` and
`plot_difficulty_{4..8}_height` fields are set to `0xffff_ffff` (never reached).
Consumers MUST NOT rely on these PoS/VDF fields for any DIG semantics; only the fields in
§3–§5.1 are DIG-normative.

## 6. Canonical relay endpoint — `DIG_RELAY_URL`

6.1. `DIG_RELAY_URL` is the string constant:

```
wss://relay.dig.net:443
```

6.2. This is the single source of truth for the DIG NAT-traversal relay endpoint: the
secure-WebSocket URL a DIG Node dials by default to obtain a relay reservation so
NAT'd peers stay reachable. The protocol served at this endpoint is the `RelayMessage`
JSON-over-WebSocket wire (message types RLY-001..RLY-007), implemented by the `dig-relay`
server and documented on the docs.dig.net Protocol pages.

6.3. Format contract: the value MUST use the `wss://` scheme (secure WebSocket), the
canonical public host `relay.dig.net`, and port `443` (the live NLB public TLS listener; the
earlier `:9450` listener is closed). The crate's test suite pins the
constant byte-for-byte and asserts each of these three format properties.

6.4. Override semantics (defined by the consumer, stated here for the contract): a node
uses `DIG_RELAY_URL` unless the operator sets the `DIG_RELAY_URL` environment variable to
another endpoint, or disables the reservation entirely with `DIG_RELAY_URL=off`.

6.5. Cross-repo conformance: this constant MUST remain byte-identical to the default
relay URL compiled into `dig-node` (its `relay` module's `DEFAULT_RELAY_URL`) and to the
`dig-relay` server's documented client endpoint. A change to scheme, host, or port is a
coordinated cross-repo protocol change, never a unilateral edit here.

## 7. Default node localhost port — `DIG_NODE_PORT`

7.1. `DIG_NODE_PORT` is the u16 constant:

```
9778
```

7.2. This is the single source of truth for the default localhost port a client uses to reach
a local DIG node (per §5.3 client→node connection order). When a client resolves `dig.local`
or `localhost`, it dials this port to reach the installed local DIG node. The constants ensures
all consumers (dig-node, dig-dns, dig-installer, dig-sdk, digstore CLI) use an identical port,
preventing port-mismatch bugs and silent failures.

7.3. Format contract: the value MUST be `9778`. The crate's test suite pins this constant
byte-for-byte.

7.4. Override semantics (defined by the consumer, stated here for the contract): a client uses
`DIG_NODE_PORT` unless explicitly configured with a custom node URL.

7.5. Cross-repo conformance: this constant MUST remain byte-identical to the port the `dig-node`
service binds on localhost and to the port the `dig-installer` registers for `dig.local`. A
change to the port is a coordinated cross-repo protocol change, never a unilateral edit here.

## 8. Invariants and error behavior

8.1. The crate has no fallible API: no function returns `Result`, panics, or performs I/O.
All values are compile-time constants; misuse is impossible at runtime.

8.2. Invariants that MUST hold in every release:

- I-1: `agg_sig_me_additional_data == genesis_challenge` for each network.
- I-2: every other `agg_sig_*_additional_data == sha256(genesis_challenge || opcode_byte)`
  per §4.1–§4.2.
- I-3: `DIG_MAINNET.genesis_challenge() != DIG_TESTNET.genesis_challenge()`.
- I-4: mainnet and testnet agree on every non-genesis-derived field (§5).
- I-5: `DIG_RELAY_URL == "wss://relay.dig.net:443"` (until a coordinated cross-repo change
  per §6.5).
- I-6: `DIG_NODE_PORT == 9778` (the default localhost port; until a coordinated cross-repo
  change per §7).
- I-7: the `chia-consensus`/`chia-protocol` dependency versions move in lockstep (currently
  the `0.26` line); a `ConsensusConstants` layout change upstream is a breaking change here
  and requires a semver-major bump.

## 9. Versioning and compatibility

9.1. The crate follows semver. Additive changes (new constants, new accessors, new
networks) are minor; removing/renaming an export, changing any published constant value,
or bumping the `chia-*` dependency line is major-worthy because downstream signature and
validation behavior depends on exact values.

9.2. Re-anchoring a genesis challenge at true launch (§3.2 — bumping the derivation preimage
to `:v2`) is the one planned value-changing event; it MUST recompute all §4 values in the
same commit and ship as a new version that all consumers adopt together.

## 10. Release and CI gates

10.1. Releases are tag-driven: pushing a `v*` tag (or a manual `workflow_dispatch`) runs the
`Publish to crates.io` workflow, which gates on `cargo fmt --check`,
`cargo clippy --all-targets --all-features -D warnings`, `cargo test --all-features`, and
`cargo doc --no-deps`, then publishes to crates.io (secret `CARGO_REGISTRY_TOKEN`) and
creates a GitHub Release. A release whose test job fails MUST NOT publish.

10.2. There is no CI workflow on plain pushes to `main`; the gates in §10.1 run on release
tags and manual dispatch.

## 11. Conformance summary

| # | Requirement | Level |
|---|---|---|
| C-1 | AGG_SIG_ME additional data equals the genesis challenge | MUST |
| C-2 | Other AGG_SIG additional data equal `sha256(genesis \|\| opcode_byte)` (opcodes 43–48) | MUST |
| C-3 | Genesis challenges are `sha256` of their documented preimage (non-zero, reproducible); re-anchoring recomputes all §4 values | MUST |
| C-4 | Consumers select a network by constant; never mix mainnet/testnet values | MUST |
| C-5 | Only §3–§5.1 fields carry DIG semantics; PoS/VDF fields are inert filler | MUST NOT rely |
| C-6 | `DIG_RELAY_URL` byte-identical to `dig-node`'s default and `dig-relay`'s endpoint | MUST |
| C-7 | Relay endpoint uses `wss://`, host `relay.dig.net`, port `443` | MUST |
| C-8 | `DIG_NODE_PORT == 9778` (client→node localhost connection port) | MUST |
| C-9 | Constant-value changes ship as coordinated semver-major releases | MUST |
| C-10 | Crate stays dependency-light (no CLVM engine / networking / async runtime) | MUST |
| C-11 | Release publishes only after fmt/clippy/test/doc gates pass | MUST |
