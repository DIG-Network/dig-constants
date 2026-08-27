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
| `DIG_ASSET_ID` | `pub const Bytes32` | Canonical $DIG CAT asset id (TAIL hash) (§8) |
| `DIG_DECIMALS` | `pub const u32` | Decimal places $DIG carries as a CAT (§8b) |
| `CAT_MOJOS_PER_DIG` | `pub const u64` | CAT mojos in one whole $DIG (§8b) |
| `MIRROR_COIN_COLLATERAL_DIG` | `pub const u64` | Mirror-coin collateral per store, in whole $DIG (§8c) |
| `MIRROR_COIN_COLLATERAL_CAT_MOJOS` | `pub const u64` | The same collateral, in CAT mojos — the amount coins carry (§8c) |
| `MIRROR_EPOCH_GENESIS_UNIX_MS` | `pub const i64` | Genesis instant of the mirror-coin epoch clock (§8d) |
| `MIRROR_EPOCH_LENGTH_MS` | `pub const i64` | Epoch length: 7 days of wall-clock UTC (§8d) |
| `MIRROR_ROUND_LENGTH_MS` | `pub const i64` | Round length: 10 minutes (§8d) |
| `MIRROR_ROUNDS_PER_EPOCH` | `pub const i64` | Rounds per epoch: 1008 (§8d) |
| `mirror_epoch_at_unix_ms` | `const fn(i64) -> i64` | The one-based epoch number containing an instant (§8d) |
| `mirror_epoch_start_unix_ms` | `const fn(i64) -> i64` | The instant at which a given epoch begins (§8d) |

2.1. `NetworkConstants`'s field is private. Consumers MUST reach the underlying
`ConsensusConstants` only via `consensus()`; the wrapper's accessors are the stable
names for the commonly needed fields.

2.2. `DIG_MAINNET` and `DIG_TESTNET` are `const` items: their values are fixed at compile
time and identical in every build of a given crate version.

## 3. Networks: mainnet / testnet split

3.1. The crate defines exactly two networks. They differ ONLY in genesis challenge and the
AGG_SIG additional-data values derived from it (§4); every other parameter (§5) is
identical between the two.

| Network | Genesis challenge (32 bytes, hex) | Source |
|---|---|---|
| `DIG_MAINNET` | `0af981862a4df51f51ec59c312315d959931d917c375730b89b9e2b0854d1abf` | Chia mainnet header hash @ height 9,021,277, pinned 2026-07-17 |
| `DIG_TESTNET` | `088c18d6b7859d885dc2f03166e862c958f74b63b6353c3df71d103b9b806c3b` | `sha256(b"DIG_TESTNET:genesis:v1")` |

3.2. **Canonical, real-anchored, pre-launch values.** The mainnet genesis anchors the DIG L2
genesis to a real, verifiable Chia block — the Chia mainnet peak header hash at block height
9,021,277 (`0af981…1abf`), captured 2026-07-17 via coinset.org `get_blockchain_state`. The
testnet genesis is the reproducible `sha256` of a fixed documented preimage. Both are non-zero
(the gossip `network_id` gate rejects an all-zero id) and independently verifiable. These are
the PRE-LAUNCH canonical values. Per the ecosystem's pre-release status they are revisable at
true mainnet launch: if re-anchored (mainnet → the launch-time Chia header hash; testnet →
`:v2`), every `agg_sig_*_additional_data` value MUST be recomputed per §4. Consumers MUST NOT
treat signatures or coins bound to this pre-launch domain as launch-final network state.

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

**DIG mainnet** (genesis = `0af981…1abf`):

| Field | Value (hex) |
|---|---|
| `agg_sig_me_additional_data` | `0af981862a4df51f51ec59c312315d959931d917c375730b89b9e2b0854d1abf` |
| `agg_sig_parent_additional_data` | `196d63b6dfbd4440656f9c1eadc686cacfaae771c565762a8cd6e51c892a0077` |
| `agg_sig_puzzle_additional_data` | `9ca719659b5e2355a91ff330c8612cb58c74f1063eaff99e507602d450b1f71f` |
| `agg_sig_amount_additional_data` | `d13767da4a8bd9520dbd9e039e68b3eb4b16fdcbb7e7755b5064840eaeb553ce` |
| `agg_sig_puzzle_amount_additional_data` | `73eea3473bd0daa28793d4bcd218ade462b634b53af97f9a01a91f3059ac75df` |
| `agg_sig_parent_amount_additional_data` | `eb7302224e77c0f269d0c8b105d4cc786775ae012ed2db49751c33c244c3f647` |
| `agg_sig_parent_puzzle_additional_data` | `ccac5983685257d50ee7b439bbb502128ddb262813dde4e4a11ac6cdfc66fa8e` |

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

## 8. Canonical $DIG CAT asset id — `DIG_ASSET_ID`

8.1. `DIG_ASSET_ID` is the `Bytes32` constant:

```
a406d3a9de984d03c9591c10d917593b434d5263cabe2b42f6b367df16832f81
```

8.2. This is the single source of truth for the $DIG CAT's asset id (its CHIP-0004 TAIL
program hash) on Chia mainnet — the value every capsule (commit) payment is denominated in
and the value a wallet, decoder, or balance check compares a CAT coin's `asset_id` against
to recognize $DIG.

8.3. Format contract: the value MUST be the 32-byte hash above. The crate's test suite pins
this constant byte-for-byte.

8.4. Cross-repo conformance: this constant MUST remain byte-identical to
`chip35_dl_coin::DIG_ASSET_ID`, digstore-chain's `DIG_ASSET_ID`, and DataLayer-Driver's. A
change to this value is a coordinated cross-repo protocol change, never a unilateral edit
here — it would silently break $DIG recognition and payment for every consumer.

## 8a. Profile DEK at-rest byte contract — `DEK_SALT`, `IDENTITY_IKM_VERSION`, `PROFILE_DEK_LABEL`, `SYMMETRIC_KEY_LEN`

8a.1. These four constants are the single source of truth for deriving a DIG user profile's
data-encryption-key (DEK):

```
DEK_SALT              = b"dig-app:dek-salt:v1"          (&[u8])
IDENTITY_IKM_VERSION   = 2                              (u8)
PROFILE_DEK_LABEL      = b"dig-app:profile-dek:v2"       (&[u8])
SYMMETRIC_KEY_LEN      = 32                              (usize)
```

8a.2. Derivation rule:

```
HKDF-SHA256(salt = DEK_SALT,
            ikm  = IDENTITY_IKM_VERSION || identity_scalar_32,
            info = PROFILE_DEK_LABEL)
  -> SYMMETRIC_KEY_LEN bytes
```

8a.3. This is a PERMANENT at-rest byte-identical contract (§4.1/§5.1/NC-5): the DEK is derived,
never stored, from the user's identity scalar. Every sealed profile on disk was encrypted with a
DEK derived from exactly these bytes. Changing any one of the four constants re-derives a
different key and makes every already-sealed profile permanently unreadable — there is no
migration path for a derived key. A future revision to this contract MUST introduce a new
version-scoped label/version alongside the existing ones, never mutate them in place.

8a.4. Format contract: the crate's test suite pins each of the four constants byte-for-byte /
literally.

8a.5. Cross-repo conformance: these constants MUST remain byte-identical to the local literals
in `dig-app` (`crates/dig-app-core/src/keystore/secrets.rs`) and `dig-session`
(`src/unlocked.rs`, `derive_symmetric_key`). Both crates consume these constants from here rather
than duplicating the literals.

## 8b. $DIG denomination — `DIG_DECIMALS`, `CAT_MOJOS_PER_DIG`

8b.1. $DIG is a CAT with **three** decimal places:

```
DIG_DECIMALS      = 3      (u32)
CAT_MOJOS_PER_DIG = 1_000  (u64)   ==  10^DIG_DECIMALS
```

8b.2. Every $DIG amount that appears in a coin, a puzzle, or a wire message is expressed in
**CAT mojos** — the smallest indivisible unit. Whole $DIG is a display and policy unit only.
Conversion is `whole_dig * CAT_MOJOS_PER_DIG`.

8b.3. Consumers MUST convert through `CAT_MOJOS_PER_DIG` rather than writing a bare factor of
1000 beside an amount. A misplaced factor in either direction is a real-money defect.

## 8c. Mirror-coin collateral — `MIRROR_COIN_COLLATERAL_DIG`, `MIRROR_COIN_COLLATERAL_CAT_MOJOS`

8c.1. A DIG store mirror locks collateral in a mirror coin. The canonical amount is:

```
MIRROR_COIN_COLLATERAL_DIG       = 20       (u64, whole $DIG)
MIRROR_COIN_COLLATERAL_CAT_MOJOS = 20_000   (u64, CAT mojos)  == 20 * CAT_MOJOS_PER_DIG
```

8c.2. `MIRROR_COIN_COLLATERAL_CAT_MOJOS` is the value a `MirrorAdvertisement`'s `collateral`
field carries and the amount a node locks when creating a mirror coin. Its unit is CAT mojos —
**not** XCH mojos and **not** whole $DIG.

8c.3. This is network **policy**, not a wire rule. `dig-mirror-coin` deliberately bakes no
amount into its puzzles; the amount here is the ecosystem's current answer and may be
re-decided without a format change. Consumers MUST NOT encode a floor or a policy of their own.

8c.4. Cross-repo conformance: dig-node (creation), the DIG App (display and shortfall) and the
dig CLI (audit) all read this constant rather than a local literal.

8c.5. The legacy system's equivalent locked 0.0003 XCH (300,000,000 XCH mojos). The DIG figure
differs in both asset and magnitude, deliberately; it MUST NOT be reconciled toward the legacy
literal.

8c.6. Format contract: the test suite pins 20 and 20,000 to their literals AND to each other
through `CAT_MOJOS_PER_DIG`, so editing one without the others fails.

## 8d. Mirror-coin epoch clock — `MIRROR_EPOCH_*`, `MIRROR_ROUND*`, `mirror_epoch_*`

8d.1. Mirror coins are scoped to an **epoch**, a pure wall-clock UTC schedule with no chain
input:

```
MIRROR_EPOCH_GENESIS_UNIX_MS = 1_725_321_600_000   (2024-09-03T00:00:00Z)
MIRROR_EPOCH_LENGTH_MS       =   604_800_000       (7 days, hard-coded)
MIRROR_ROUND_LENGTH_MS       =       600_000       (10 minutes)
MIRROR_ROUNDS_PER_EPOCH      =         1_008       ( == EPOCH / ROUND )
```

8d.2. Normative epoch rule — the epoch is **one-based**:

```
epoch(now_ms) = floor((now_ms - MIRROR_EPOCH_GENESIS_UNIX_MS) / MIRROR_EPOCH_LENGTH_MS) + 1
```

`floor` is floored division (Rust `div_euclid`), matching JavaScript `Math.floor`, so instants
before genesis yield zero or a negative number rather than truncating toward zero. The genesis
instant itself is epoch **1**.

8d.3. `mirror_epoch_start_unix_ms(epoch)` is the exact inverse on that numbering:
`GENESIS + (epoch - 1) * EPOCH_LENGTH`.

8d.4. The epoch number is an **input to coin identity**:
`dig_mirror_coin::morph_store_launcher_id(launcher_id, epoch)` derives the hint under which a
mirror coin is announced and found. A consumer computing a different epoch number does not
merely mislabel — it creates or queries coins under a hint no peer uses, orphaning an epoch's
coins. Consumers MUST use this clock and MUST NOT re-derive a genesis or a window length.

8d.5. This clock is NOT the `dig-epoch` crate, which defines L2 epoch geometry anchored to L1
block heights with BlockProduction / Checkpoint / Finalization phases. The two notions share a
word only; `dig-epoch` MUST NOT be substituted here.

8d.6. Format contract: the test suite pins the genesis instant by recomputing it from the Unix
epoch, and pins the boundaries at genesis, one millisecond before genesis, the last millisecond
of an epoch, and the rollover instant, plus a hand-computed known-good pair.

## 9. Invariants and error behavior

9.1. The crate has no fallible API: no function returns `Result`, panics, or performs I/O.
The two epoch `const fn`s are total arithmetic over `i64` wall-clock milliseconds; they can
overflow only for inputs far outside any representable date.
All values are compile-time constants; misuse is impossible at runtime.

9.2. Invariants that MUST hold in every release:

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
  the `0.36` line); a `ConsensusConstants` layout change upstream is a breaking change here
  and requires a semver-major bump.
- I-8: `DIG_ASSET_ID` equals the pinned $DIG CAT tail hash (§8.1; until a coordinated
  cross-repo change per §8.4).
- I-9: `CAT_MOJOS_PER_DIG == 10^DIG_DECIMALS`, and
  `MIRROR_COIN_COLLATERAL_CAT_MOJOS == MIRROR_COIN_COLLATERAL_DIG * CAT_MOJOS_PER_DIG`
  (§8b, §8c).
- I-10: `mirror_epoch_at_unix_ms(MIRROR_EPOCH_GENESIS_UNIX_MS) == 1` — the epoch numbering is
  one-based (§8d.2).
- I-11: `MIRROR_ROUNDS_PER_EPOCH == MIRROR_EPOCH_LENGTH_MS / MIRROR_ROUND_LENGTH_MS == 1008`
  (§8d.1).

## 10. Versioning and compatibility

10.1. The crate follows semver. Additive changes (new constants, new accessors, new
networks) are minor; removing/renaming an export, changing any published constant value,
or bumping the `chia-*` dependency line is major-worthy because downstream signature and
validation behavior depends on exact values.

10.2. Re-anchoring a genesis challenge at true launch (§3.2 — mainnet to the launch-time Chia
header hash, testnet to a `:v2` preimage) is the one planned value-changing event; it MUST
recompute all §4 values in the same commit and ship as a new version that all consumers adopt
together.

## 11. Release and CI gates

11.1. Releases are tag-driven: pushing a `v*` tag (or a manual `workflow_dispatch`) runs the
`Publish to crates.io` workflow, which gates on `cargo fmt --check`,
`cargo clippy --all-targets --all-features -D warnings`, `cargo test --all-features`, and
`cargo doc --no-deps`, then publishes to crates.io (secret `CARGO_REGISTRY_TOKEN`) and
creates a GitHub Release. A release whose test job fails MUST NOT publish.

11.2. There is no CI workflow on plain pushes to `main`; the gates in §11.1 run on release
tags and manual dispatch.

## 12. Conformance summary

| # | Requirement | Level |
|---|---|---|
| C-1 | AGG_SIG_ME additional data equals the genesis challenge | MUST |
| C-2 | Other AGG_SIG additional data equal `sha256(genesis \|\| opcode_byte)` (opcodes 43–48) | MUST |
| C-3 | Genesis challenges are non-zero, verifiable pinned values (mainnet = Chia header hash @ 9,021,277; testnet = `sha256` of preimage); re-anchoring recomputes all §4 values | MUST |
| C-4 | Consumers select a network by constant; never mix mainnet/testnet values | MUST |
| C-5 | Only §3–§5.1 fields carry DIG semantics; PoS/VDF fields are inert filler | MUST NOT rely |
| C-6 | `DIG_RELAY_URL` byte-identical to `dig-node`'s default and `dig-relay`'s endpoint | MUST |
| C-7 | Relay endpoint uses `wss://`, host `relay.dig.net`, port `443` | MUST |
| C-8 | `DIG_NODE_PORT == 9778` (client→node localhost connection port) | MUST |
| C-9 | Constant-value changes ship as coordinated semver-major releases | MUST |
| C-10 | Crate stays dependency-light (no CLVM engine / networking / async runtime) | MUST |
| C-11 | Release publishes only after fmt/clippy/test/doc gates pass | MUST |
| C-12 | `DIG_ASSET_ID` byte-identical to `chip35_dl_coin::DIG_ASSET_ID` (the $DIG CAT tail hash) | MUST |

