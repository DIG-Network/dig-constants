//! DIG Network Constants
//!
//! Defines network parameters for the DIG L2 blockchain. This crate exists
//! separately so that any DIG crate can import network constants without
//! pulling in the full CLVM engine or other heavy dependencies.
//!
//! The core type is [`NetworkConstants`], which wraps `chia-consensus`'s
//! `ConsensusConstants` with DIG-specific values (genesis challenge,
//! AGG_SIG additional data, cost limits, etc.).
//!
//! # Chia L1 vs DIG L2 (do not mix)
//!
//! [`DIG_MAINNET`] / [`DIG_TESTNET`] describe the DIG **L2** network. Separately,
//! [`CHIA_L1_MAINNET_AGG_SIG_ME`] / [`CHIA_L1_TESTNET11_AGG_SIG_ME`] hold the
//! **Chia L1 (foreign chain)** genesis challenge that DIG wallet code needs as
//! AGG_SIG_ME additional data when signing L1 spends. They live here as the
//! ecosystem's single source of truth, but are DELIBERATELY distinct from the DIG
//! L2 genesis — signing an L1 spend with the DIG L2 genesis produces an invalid
//! signature. The `CHIA_L1_` prefix is the anti-mixup guard.
//!
//! # Usage
//!
//! ```rust,ignore
//! use dig_constants::DIG_MAINNET;
//!
//! let genesis = DIG_MAINNET.genesis_challenge();
//! let consensus = DIG_MAINNET.consensus();
//! ```

use chia_consensus::consensus_constants::ConsensusConstants;
use chia_protocol::Bytes32;
use hex_literal::hex;

/// DIG network constants.
///
/// Wraps `chia-consensus::ConsensusConstants` with accessors for the fields
/// that DIG validators and wallet code commonly need. The underlying
/// `ConsensusConstants` is available via [`consensus()`](Self::consensus)
/// for direct use with `chia-consensus` functions like `run_spendbundle()`.
#[derive(Debug, Clone)]
pub struct NetworkConstants {
    inner: ConsensusConstants,
}

impl NetworkConstants {
    /// The underlying `chia-consensus` constants, for passing directly to
    /// `run_spendbundle()`, `validate_clvm_and_signature()`, etc.
    pub fn consensus(&self) -> &ConsensusConstants {
        &self.inner
    }

    /// DIG genesis challenge.
    pub fn genesis_challenge(&self) -> Bytes32 {
        self.inner.genesis_challenge
    }

    /// AGG_SIG_ME additional data (== genesis_challenge on Chia L1).
    pub fn agg_sig_me_additional_data(&self) -> Bytes32 {
        self.inner.agg_sig_me_additional_data
    }

    /// Maximum CLVM cost per block.
    pub fn max_block_cost_clvm(&self) -> u64 {
        self.inner.max_block_cost_clvm
    }

    /// Cost per byte of generator program.
    pub fn cost_per_byte(&self) -> u64 {
        self.inner.cost_per_byte
    }

    /// Maximum coin amount (u64::MAX).
    pub fn max_coin_amount(&self) -> u64 {
        self.inner.max_coin_amount
    }
}

// =============================================================================
// AGG_SIG additional data derivation
//
// On Chia L1, each AGG_SIG_* variant's additional_data is:
//   sha256(genesis_challenge || opcode_byte)
// except AGG_SIG_ME which uses genesis_challenge directly.
//
// See: condition_tools.py:58-71
//   https://github.com/Chia-Network/chia-blockchain/blob/main/chia/consensus/condition_tools.py#L58
// =============================================================================

// ---------------------------------------------------------------------------
// DIG Mainnet
//
// The genesis challenge is the 32-byte consensus anchor for the DIG L2 network.
// It doubles as the gossip `network_id` gate: `dig-gossip` REJECTS an all-zero
// network_id, so this value MUST be non-zero for the node's gossip pool / DHT /
// PEX to start.
//
// DIG_MAINNET L2 genesis = the Chia mainnet header hash @ height 9,021,277
//   (0af981...1abf), pinned 2026-07-17 — anchors the DIG L2 genesis to a real,
//   verifiable Chia block (captured via coinset.org get_blockchain_state).
//
//   DIG_MAINNET_GENESIS_CHALLENGE
//     = 0af981862a4df51f51ec59c312315d959931d917c375730b89b9e2b0854d1abf
//
// This is the PRE-LAUNCH canonical DIG mainnet genesis. Per CLAUDE.md §3.7 the
// ecosystem is pre-release with no live users, so this value is revisable at
// true mainnet launch — re-anchor to the launch-time Chia header hash and
// recompute every derived value below if it is ever changed.
//
// All `agg_sig_*_additional_data` values are derived from this genesis as
// `sha256(genesis_challenge || opcode_byte)` (AGG_SIG_ME = genesis directly),
// so they were all recomputed for this genesis.
// ---------------------------------------------------------------------------

/// Canonical DIG mainnet genesis challenge.
///
/// The Chia mainnet header hash at block height 9,021,277 (`0af981…1abf`),
/// pinned 2026-07-17 — a real, verifiable, fixed 32-byte value anchoring the
/// DIG L2 genesis to a real Chia block. This is the pre-launch canonical value;
/// per §3.7 it is revisable at true mainnet launch. All
/// `agg_sig_*_additional_data` fields are derived from this.
const DIG_MAINNET_GENESIS_CHALLENGE: [u8; 32] =
    hex!("0af981862a4df51f51ec59c312315d959931d917c375730b89b9e2b0854d1abf");

/// DIG mainnet constants.
///
/// Uses DIG's own genesis challenge and AGG_SIG domain separation.
/// Proof-of-space and VDF fields are set to neutral values since DIG L2
/// does not use Chia's proof-of-space consensus.
pub const DIG_MAINNET: NetworkConstants = NetworkConstants {
    inner: ConsensusConstants {
        // -- DIG-specific values --
        genesis_challenge: Bytes32::new(DIG_MAINNET_GENESIS_CHALLENGE),

        // AGG_SIG additional data: derived from genesis_challenge.
        // AGG_SIG_ME = genesis_challenge directly.
        // Others = sha256(genesis_challenge || opcode_byte).
        // Derivation: condition_tools.py:58-71
        //   https://github.com/Chia-Network/chia-blockchain/blob/main/chia/consensus/condition_tools.py#L58
        // Opcode bytes: AGG_SIG_PARENT=43, PUZZLE=44, AMOUNT=45,
        //   PUZZLE_AMOUNT=46, PARENT_AMOUNT=47, PARENT_PUZZLE=48
        // NOTE: Recompute ALL values when genesis_challenge is finalized.
        agg_sig_me_additional_data: Bytes32::new(DIG_MAINNET_GENESIS_CHALLENGE),
        agg_sig_parent_additional_data: Bytes32::new(hex!(
            "196d63b6dfbd4440656f9c1eadc686cacfaae771c565762a8cd6e51c892a0077"
        )),
        agg_sig_puzzle_additional_data: Bytes32::new(hex!(
            "9ca719659b5e2355a91ff330c8612cb58c74f1063eaff99e507602d450b1f71f"
        )),
        agg_sig_amount_additional_data: Bytes32::new(hex!(
            "d13767da4a8bd9520dbd9e039e68b3eb4b16fdcbb7e7755b5064840eaeb553ce"
        )),
        agg_sig_puzzle_amount_additional_data: Bytes32::new(hex!(
            "73eea3473bd0daa28793d4bcd218ade462b634b53af97f9a01a91f3059ac75df"
        )),
        agg_sig_parent_amount_additional_data: Bytes32::new(hex!(
            "eb7302224e77c0f269d0c8b105d4cc786775ae012ed2db49751c33c244c3f647"
        )),
        agg_sig_parent_puzzle_additional_data: Bytes32::new(hex!(
            "ccac5983685257d50ee7b439bbb502128ddb262813dde4e4a11ac6cdfc66fa8e"
        )),

        // DIG L2 cost limits
        max_block_cost_clvm: 11_000_000_000, // per-spend limit, same as Chia L1
        cost_per_byte: 12_000,
        max_coin_amount: u64::MAX,

        // Block generator limits
        max_generator_size: 1_000_000,
        max_generator_ref_list_size: 512,

        // Hard fork heights — set to 0 to always use latest consensus rules.
        // DIG L2 starts with all features enabled from block 0.
        hard_fork_height: 0,
        hard_fork2_height: 0,

        // Pre-farm puzzle hashes — not used by DIG L2, set to zero.
        genesis_pre_farm_pool_puzzle_hash: Bytes32::new([0u8; 32]),
        genesis_pre_farm_farmer_puzzle_hash: Bytes32::new([0u8; 32]),

        // -- Proof-of-space / VDF fields (not used by DIG L2) --
        // These must be valid values since ConsensusConstants is passed to
        // chia-consensus functions, but DIG does not use PoS consensus.
        slot_blocks_target: 32,
        min_blocks_per_challenge_block: 16,
        max_sub_slot_blocks: 128,
        num_sps_sub_slot: 64,
        sub_slot_iters_starting: 1 << 27,
        difficulty_constant_factor: 1 << 67,
        difficulty_starting: 7,
        difficulty_change_max_factor: 3,
        sub_epoch_blocks: 384,
        epoch_blocks: 4608,
        significant_bits: 8,
        discriminant_size_bits: 1024,
        number_zero_bits_plot_filter_v1: 9,
        number_zero_bits_plot_filter_v2: 9,
        min_plot_size_v1: 32,
        max_plot_size_v1: 50,
        min_plot_size_v2: 28,
        max_plot_size_v2: 32,
        sub_slot_time_target: 600,
        num_sp_intervals_extra: 3,
        max_future_time2: 120,
        number_of_timestamps: 11,
        max_vdf_witness_size: 64,
        mempool_block_buffer: 10,
        weight_proof_threshold: 2,
        blocks_cache_size: 4608 + (128 * 4),
        weight_proof_recent_blocks: 1000,
        max_block_count_per_requests: 32,
        pool_sub_slot_iters: 37_600_000_000,
        plot_filter_128_height: 0xffff_ffff,
        plot_filter_64_height: 0xffff_ffff,
        plot_filter_32_height: 0xffff_ffff,
        plot_difficulty_initial: 2,
        plot_difficulty_4_height: 0xffff_ffff,
        plot_difficulty_5_height: 0xffff_ffff,
        plot_difficulty_6_height: 0xffff_ffff,
        plot_difficulty_7_height: 0xffff_ffff,
        plot_difficulty_8_height: 0xffff_ffff,
    },
};

// =============================================================================
// NAT-traversal relay endpoint
//
// A DIG Node behind NAT cannot accept inbound dials, so it holds a constant
// reservation with a publicly-reachable relay to stay discoverable. The
// canonical public relay is `relay.dig.net`, serving the `RelayMessage`
// WebSocket wire (RLY-001..RLY-007) on port 9450.
//
// This constant is the single source of truth for that endpoint so consumers
// (`dig-node`, `dig-gossip`) don't each hardcode it. It MUST stay byte-identical
// to `dig-node`'s `relay::DEFAULT_RELAY_URL` (the string a node actually dials
// when `DIG_RELAY_URL` is unset) and to the `dig-relay` server's documented
// client endpoint.
//
// Port 443: the live `relay.dig.net` NLB exposes its public TLS listener on the
// standard HTTPS port 443 (the earlier :9450 listener is closed). Using 443 also
// maximizes reachability from restrictive networks that only allow outbound 443.
// =============================================================================

/// Canonical DIG NAT-traversal relay endpoint.
///
/// This is the WebSocket URL a DIG Node dials by default to obtain a relay
/// reservation (so NAT'd peers stay reachable). It is the value used unless an
/// operator overrides it via the `DIG_RELAY_URL` environment variable (or
/// disables the reservation with `DIG_RELAY_URL=off`).
///
/// Format: `wss://<host>:<port>` — the relay protocol (`RelayMessage`,
/// RLY-001..RLY-007) is JSON over a secure WebSocket. Mainnet uses the canonical
/// public deployment `relay.dig.net` on port 443 (the live NLB public TLS
/// listener; the earlier :9450 listener is closed).
///
/// Kept byte-identical to `dig-node`'s `relay::DEFAULT_RELAY_URL` and the
/// `dig-relay` server's documented client endpoint.
pub const DIG_RELAY_URL: &str = "wss://relay.dig.net:443";

// =============================================================================
// DIG Node localhost endpoint
//
// A client connecting to a local DIG node (§5.3 client→node connection order)
// resolves `dig.local` or `localhost` to reach the node via localhost TCP on
// port 9778. This constant is the single source of truth for that port so
// consumers (dig-node, dig-dns, dig-installer, SDK, CLI) don't each hardcode it.
// =============================================================================

/// The default localhost port a client uses to reach the local DIG node.
///
/// This is used to implement §5.3 client→node connection order: when a client
/// needs to connect to a DIG node, it tries `dig.local` and `localhost` on this
/// port before falling back to the public `rpc.dig.net` gateway. This constant
/// ensures all consumers (dig-node, dig-dns, dig-installer, dig-sdk, digstore CLI)
/// use an identical port, preventing port-mismatch bugs. It MUST stay byte-identical
/// to `dig-node`'s documented localhost serve port and the installer's registered
/// `dig.local` address.
pub const DIG_NODE_PORT: u16 = 9778;

/// The mDNS/local hostname the installed DIG node registers.
///
/// This is the FIRST tier of the §5.3 client→node connection order: a client
/// tries `dig.local` (on [`DIG_NODE_PORT`]) before falling back to `localhost`
/// and finally the public [`RPC_DIG_NET_URL`] gateway. This constant ensures
/// all consumers (dig-node, dig-dns, dig-installer, dig-sdk, digstore CLI) use
/// an identical hostname, preventing drift between the address the installer
/// registers and the address clients probe.
pub const DIG_LOCAL_HOST: &str = "dig.local";

/// The public DIG read gateway.
///
/// This is the FINAL-FALLBACK tier of the §5.3 client→node connection order:
/// a client falls through to this plain-HTTPS public read tier only when
/// neither `dig.local` nor `localhost` (both on [`DIG_NODE_PORT`]) responds.
/// This constant ensures all consumers (dig-download, digstore CLI, dig-sdk,
/// dig-node) reference an identical gateway URL instead of each hardcoding
/// their own copy of `rpc.dig.net`.
pub const RPC_DIG_NET_URL: &str = "https://rpc.dig.net";

// =============================================================================
// DIG CAT asset id ($DIG token)
//
// $DIG is a Chia CAT (CHIP-0004); its asset id is the TAIL program's hash,
// fixed for the token's lifetime. This is the single canonical home for that
// value — `chip35_dl_coin`, `dig-cat-decoder`, and any DIG-aware wallet/
// balance/spend code import it from HERE rather than each hardcoding a copy.
// =============================================================================

/// Canonical $DIG CAT asset id (TAIL hash) on Chia mainnet.
///
/// The single token every capsule (commit) payment is denominated in
/// (`chip35_dl_coin::build_dig_store_payment`) and the value a wallet/decoder
/// checks a CAT coin's `asset_id` against to recognize $DIG.
///
/// CONTRACT: byte-identical to `chip35_dl_coin::DIG_ASSET_ID`, digstore-chain's
/// `DIG_ASSET_ID`, and DataLayer-Driver's. Do not change without changing every
/// consumer in lockstep (SYSTEM.md → Shared contracts → DIG CAT payment).
pub const DIG_ASSET_ID: Bytes32 = Bytes32::new(hex!(
    "a406d3a9de984d03c9591c10d917593b434d5263cabe2b42f6b367df16832f81"
));

// =============================================================================
// DIG treasury recipient (destination of $DIG payments + dev-tips)
//
// Every $DIG capsule/commit payment and dev-tip is created-coin'd to the DIG
// treasury. This section is the single canonical home for that recipient in two
// equivalent forms: the on-chain inner (standard) puzzle hash and its bech32m
// address. A WRONG value here silently MISDIRECTS funds to an attacker/void —
// a custody break — so both forms are pinned byte-for-byte by tests, and a KAT
// proves the address decodes to the puzzle hash (they cannot drift apart).
//
// CONTRACT: dig-constants is the intended canonical LOWEST-level home for this
// value. The existing higher-level copies (`digstore_chain::dig`,
// `chip35_dl_coin`, `dighub-core`) SHOULD later converge to re-export from HERE.
// That convergence is a SEPARATE follow-up — this change only introduces the
// canonical constants; it does not touch those crates. Until convergence, this
// value stays byte-identical to `digstore_chain::dig` (the current source of
// truth: `TREASURY_ADDRESS` at `crates/digstore-chain/src/dig.rs:41`, from which
// it derives `treasury_inner_puzzle_hash()`, pinned by its test at dig.rs:206-209).
// =============================================================================

/// Canonical DIG treasury inner (standard) puzzle hash.
///
/// The on-chain recipient every $DIG capsule/commit payment and dev-tip is
/// created-coin'd to. A wrong value silently misdirects treasury funds (a
/// custody break), so it is pinned byte-for-byte by a test.
///
/// CONTRACT: byte-identical to what `digstore_chain::dig::treasury_inner_puzzle_hash()`
/// decodes to (pinned by that crate's test at `crates/digstore-chain/src/dig.rs:206-209`).
/// dig-constants is the intended canonical lowest-level home; higher copies
/// (`digstore_chain::dig`, `chip35_dl_coin`, `dighub-core`) should later
/// re-export from here (a separate follow-up — see the section note above).
pub const DIG_TREASURY_INNER_PUZZLE_HASH: Bytes32 = Bytes32::new(hex!(
    "ec7c304708c7d59c078d5ae098d0dea004decf47fa1cafebb266c10ad6466ce8"
));

/// Canonical DIG treasury address (bech32m form of [`DIG_TREASURY_INNER_PUZZLE_HASH`]).
///
/// The human-readable `xch1…` form of the same treasury recipient — the
/// destination of $DIG payments and dev-tips. A wrong value misdirects funds
/// (a custody break), so it is pinned by a test AND a KAT proves it decodes to
/// [`DIG_TREASURY_INNER_PUZZLE_HASH`] (the two forms cannot silently drift).
///
/// CONTRACT: digstore-chain's source-of-truth form (`digstore_chain::dig::TREASURY_ADDRESS`,
/// `crates/digstore-chain/src/dig.rs:41`), from which it derives the puzzle hash
/// at runtime. dig-constants is the intended canonical lowest-level home; higher
/// copies should later re-export from here (a separate follow-up).
pub const DIG_TREASURY_ADDRESS: &str =
    "xch1a37rq3cgcl2ecpudttsf35x75qzdan68lgw2l6ajvmqs44jxdn5qv6pk3y";

// =============================================================================
// Chia L1 (foreign chain) AGG_SIG_ME additional data
//
// The DIG wallet signs and validates spends on the Chia L1 chain. On Chia L1 the
// AGG_SIG_ME additional data IS the network genesis challenge, so every L1 spend
// signature is bound to it. This is a FOREIGN chain's value — completely distinct
// from the DIG L2 genesis (`DIG_MAINNET_GENESIS_CHALLENGE`, 0af98186…).
//
// Both the wallet's signer seam AND the engine's message-binding seam MUST read
// the SAME 32 bytes from here, or a spend the engine builds is signed with a
// different domain than it binds — a custody break (invalid, unspendable
// signatures on mainnet). This crate is the single source of truth for those
// bytes; the `[u8; 32]` shape matches the signer field directly (the engine wraps
// it once via `Bytes32::new(...)`).
//
// The value is invariant-forced: it is exactly Chia's well-known mainnet genesis
// (ccd5bb71…) / testnet11 genesis (37a90eb5…), the same values
// `chia-wallet-sdk`'s `MAINNET_CONSTANTS` / `TESTNET11_CONSTANTS` carry (asserted
// by an anti-drift dev-dependency test).
// =============================================================================

/// Chia **L1 mainnet** genesis challenge, used as AGG_SIG_ME additional data.
///
/// The 32-byte domain every Chia L1 mainnet spend signature is bound to. This is
/// the foreign-chain (Chia) value — DISTINCT from the DIG L2 genesis
/// ([`DIG_MAINNET`]); signing an L1 spend with the DIG L2 genesis yields an
/// invalid signature.
///
/// CONTRACT: DIG wallet consumers (the client signer AND the engine's
/// message-binding path) MUST both use this constant so signer == engine,
/// producing byte-identical, valid signatures. Equals Chia's canonical mainnet
/// genesis `ccd5bb71…` (== `chia_sdk_types::MAINNET_CONSTANTS.agg_sig_me_additional_data`).
pub const CHIA_L1_MAINNET_AGG_SIG_ME: [u8; 32] =
    hex!("ccd5bb71183532bff220ba46c268991a3ff07eb358e8255a65c30a2dce0e5fbb");

/// Chia **L1 testnet11** genesis challenge, used as AGG_SIG_ME additional data.
///
/// The 32-byte domain every Chia L1 testnet11 spend signature is bound to. As
/// with [`CHIA_L1_MAINNET_AGG_SIG_ME`], this is the foreign-chain (Chia) value,
/// DISTINCT from the DIG L2 genesis ([`DIG_TESTNET`]).
///
/// CONTRACT: DIG wallet consumers (signer AND engine) MUST both use this constant
/// so signer == engine on testnet11. Equals Chia's canonical testnet11 genesis
/// `37a90eb5…` (== `chia_sdk_types::TESTNET11_CONSTANTS.agg_sig_me_additional_data`).
pub const CHIA_L1_TESTNET11_AGG_SIG_ME: [u8; 32] =
    hex!("37a90eb5185a9c4439a91ddc98bbadce7b4feba060d50116a067de66bf236615");

// ---------------------------------------------------------------------------
// DIG Testnet
// ---------------------------------------------------------------------------

/// Canonical DIG testnet genesis challenge.
///
/// Deterministically derived as `sha256(b"DIG_TESTNET:genesis:v1")` — distinct
/// from mainnet so the two networks never share a `network_id`. Non-zero so the
/// gossip network_id gate accepts it. Pre-launch canonical value (§3.7),
/// revisable at true launch; all derived agg_sig data below follows it.
///   = 088c18d6b7859d885dc2f03166e862c958f74b63b6353c3df71d103b9b806c3b
const DIG_TESTNET_GENESIS_CHALLENGE: [u8; 32] =
    hex!("088c18d6b7859d885dc2f03166e862c958f74b63b6353c3df71d103b9b806c3b");

/// DIG testnet constants.
///
/// Same structure as mainnet but with a different genesis challenge.
/// Useful for testing without risking mainnet state.
pub const DIG_TESTNET: NetworkConstants = NetworkConstants {
    inner: ConsensusConstants {
        genesis_challenge: Bytes32::new(DIG_TESTNET_GENESIS_CHALLENGE),
        // AGG_SIG_ME = genesis_challenge. Others = sha256(genesis || opcode_byte).
        agg_sig_me_additional_data: Bytes32::new(DIG_TESTNET_GENESIS_CHALLENGE),
        agg_sig_parent_additional_data: Bytes32::new(hex!(
            "85b3963bdeb9848af970a9bbd1d36809ae41491ffd67aee7f27e8883936d495c"
        )),
        agg_sig_puzzle_additional_data: Bytes32::new(hex!(
            "66aba1939e128e1465d58fde414325630e891747c1428d76ebce193cbe966301"
        )),
        agg_sig_amount_additional_data: Bytes32::new(hex!(
            "eccab86920a6d982a68898b2dcb7c150383529fcd532fe84c693fb4592c38ae3"
        )),
        agg_sig_puzzle_amount_additional_data: Bytes32::new(hex!(
            "eb088fad0d4caba66e29130fb07407e60a7545d035d19a188fef0855c874084e"
        )),
        agg_sig_parent_amount_additional_data: Bytes32::new(hex!(
            "232aec0a351ba4936b04920e074aebcc621a458f6b1461c4b28c658552f2f35d"
        )),
        agg_sig_parent_puzzle_additional_data: Bytes32::new(hex!(
            "96263ac395703ab9b3b0f0587e79185f4a9898574a28b4491015ddcf9d321873"
        )),
        // All other fields same as mainnet
        max_block_cost_clvm: 11_000_000_000,
        cost_per_byte: 12_000,
        max_coin_amount: u64::MAX,
        max_generator_size: 1_000_000,
        max_generator_ref_list_size: 512,
        hard_fork_height: 0,
        hard_fork2_height: 0,
        genesis_pre_farm_pool_puzzle_hash: Bytes32::new([0u8; 32]),
        genesis_pre_farm_farmer_puzzle_hash: Bytes32::new([0u8; 32]),
        slot_blocks_target: 32,
        min_blocks_per_challenge_block: 16,
        max_sub_slot_blocks: 128,
        num_sps_sub_slot: 64,
        sub_slot_iters_starting: 1 << 27,
        difficulty_constant_factor: 1 << 67,
        difficulty_starting: 7,
        difficulty_change_max_factor: 3,
        sub_epoch_blocks: 384,
        epoch_blocks: 4608,
        significant_bits: 8,
        discriminant_size_bits: 1024,
        number_zero_bits_plot_filter_v1: 9,
        number_zero_bits_plot_filter_v2: 9,
        min_plot_size_v1: 32,
        max_plot_size_v1: 50,
        min_plot_size_v2: 28,
        max_plot_size_v2: 32,
        sub_slot_time_target: 600,
        num_sp_intervals_extra: 3,
        max_future_time2: 120,
        number_of_timestamps: 11,
        max_vdf_witness_size: 64,
        mempool_block_buffer: 10,
        weight_proof_threshold: 2,
        blocks_cache_size: 4608 + (128 * 4),
        weight_proof_recent_blocks: 1000,
        max_block_count_per_requests: 32,
        pool_sub_slot_iters: 37_600_000_000,
        plot_filter_128_height: 0xffff_ffff,
        plot_filter_64_height: 0xffff_ffff,
        plot_filter_32_height: 0xffff_ffff,
        plot_difficulty_initial: 2,
        plot_difficulty_4_height: 0xffff_ffff,
        plot_difficulty_5_height: 0xffff_ffff,
        plot_difficulty_6_height: 0xffff_ffff,
        plot_difficulty_7_height: 0xffff_ffff,
        plot_difficulty_8_height: 0xffff_ffff,
    },
};

// =============================================================================
// Profile DEK at-rest byte contract
//
// A DIG user profile's data-encryption-key (DEK) is derived, never stored, from
// the user's identity scalar via HKDF-SHA256:
//
//   HKDF-SHA256(salt = DEK_SALT,
//               ikm  = IDENTITY_IKM_VERSION || identity_scalar_32,
//               info = PROFILE_DEK_LABEL)
//     -> SYMMETRIC_KEY_LEN bytes
//
// These four values are a PERMANENT at-rest byte-identical contract (§4.1/§5.1/
// NC-5): every sealed profile on disk was encrypted with a DEK derived from
// EXACTLY these bytes. Changing any one of them re-derives a different key and
// makes every already-sealed profile permanently unreadable — there is no
// migration path for a derived (never-stored) key. Treat this section as
// frozen; only ever ADD a new version-scoped label/version alongside it.
//
// Consumers (this crate is their single source of truth — do not duplicate the
// literals locally):
//   - dig-app:    crates/dig-app-core/src/keystore/secrets.rs
//   - dig-session: src/unlocked.rs (derive_symmetric_key)
// =============================================================================

/// HKDF salt for the per-profile DEK derivation.
///
/// Part of the frozen [profile DEK byte contract](self#profile-dek-at-rest-byte-contract)
/// — see the section comment above. Consumed by dig-app's
/// `keystore/secrets.rs` and dig-session's `derive_symmetric_key`.
pub const DEK_SALT: &[u8] = b"dig-app:dek-salt:v1";

/// Version byte prefixed to the 32-byte identity scalar to form the DEK's HKDF
/// input key material (`IDENTITY_IKM_VERSION || identity_scalar_32`).
///
/// Part of the frozen [profile DEK byte contract](self#profile-dek-at-rest-byte-contract).
/// Consumed by dig-app's `keystore/secrets.rs` and dig-session's
/// `derive_symmetric_key`.
pub const IDENTITY_IKM_VERSION: u8 = 2;

/// HKDF info/label for the per-profile DEK derivation.
///
/// Part of the frozen [profile DEK byte contract](self#profile-dek-at-rest-byte-contract).
/// Consumed by dig-app's `keystore/secrets.rs` and dig-session's
/// `derive_symmetric_key`.
pub const PROFILE_DEK_LABEL: &[u8] = b"dig-app:profile-dek:v2";

/// Output length, in bytes, of the derived per-profile DEK (HKDF-SHA256's
/// natural output for a symmetric AEAD key).
///
/// Part of the frozen [profile DEK byte contract](self#profile-dek-at-rest-byte-contract).
/// Consumed by dig-app's `keystore/secrets.rs` and dig-session's
/// `derive_symmetric_key`.
pub const SYMMETRIC_KEY_LEN: usize = 32;

#[cfg(test)]
mod tests {
    use super::*;

    /// The canonical relay endpoint must equal exactly what a DIG Node dials by
    /// default. This pins the value byte-for-byte against `dig-node`'s
    /// `relay::DEFAULT_RELAY_URL` (`wss://relay.dig.net:9450`) and the
    /// `dig-relay` server's documented client endpoint. If either side ever
    /// changes the scheme, host, or port, this guard fails so the shared
    /// contract can't silently drift.
    #[test]
    fn dig_relay_url_is_canonical_endpoint() {
        assert_eq!(DIG_RELAY_URL, "wss://relay.dig.net:443");
    }

    /// The relay endpoint is a secure-WebSocket URL pointing at the canonical
    /// public host on the relay protocol port.
    #[test]
    fn dig_relay_url_is_well_formed() {
        assert!(
            DIG_RELAY_URL.starts_with("wss://"),
            "relay must use secure WebSocket"
        );
        assert!(
            DIG_RELAY_URL.contains("relay.dig.net"),
            "relay must point at the canonical host"
        );
        assert!(
            DIG_RELAY_URL.ends_with(":443"),
            "relay must use the live NLB public TLS port 443"
        );
    }

    /// The DIG node localhost port must equal the expected default.
    ///
    /// This guards against accidental mutations and ensures all consumers
    /// (dig-node, dig-dns, dig-installer, dig-sdk, digstore) use a consistent
    /// port when connecting to the local node on `dig.local` or `localhost`.
    #[test]
    fn dig_node_port_is_canonical() {
        assert_eq!(DIG_NODE_PORT, 9778);
    }

    /// The local-node hostname must equal the expected default.
    ///
    /// This guards the first tier of the §5.3 client→node connection order —
    /// a drift here would desync the installer's registered address from what
    /// clients probe.
    #[test]
    fn dig_local_host_is_canonical() {
        assert_eq!(DIG_LOCAL_HOST, "dig.local");
    }

    /// The public read gateway must equal the expected default.
    ///
    /// This guards the final-fallback tier of the §5.3 client→node connection
    /// order — the gateway every consumer falls through to when no local node
    /// responds.
    #[test]
    fn rpc_dig_net_url_is_canonical() {
        assert_eq!(RPC_DIG_NET_URL, "https://rpc.dig.net");
    }

    /// The public read gateway is a plain-HTTPS URL (the public read tier,
    /// distinct from the mTLS transport node-class clients use, §5.3).
    #[test]
    fn rpc_dig_net_url_is_well_formed() {
        assert!(
            RPC_DIG_NET_URL.starts_with("https://"),
            "the public read gateway must use HTTPS"
        );
    }

    // -- Genesis challenge canonical-value guards --------------------------
    //
    // These pin the pre-launch canonical genesis challenges byte-for-byte AND
    // prove they are reproducible from their documented preimages, so the
    // values can never silently drift (a drift changes every derived signature
    // domain + the gossip network_id — a cross-repo breaking event).

    use sha2::{Digest, Sha256};

    /// AGG_SIG opcode bytes, per §4.2 of `SPEC.md` (Chia L1 `condition_tools`).
    const AGG_SIG_OPCODES: [u8; 6] = [43, 44, 45, 46, 47, 48];

    fn sha256(bytes: &[u8]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        hasher.finalize().into()
    }

    /// The genesis MUST be non-zero: `dig-gossip` rejects an all-zero
    /// `network_id`, so a zero genesis would stop the node's gossip pool / DHT /
    /// PEX from ever starting. This is the connect-enabler invariant.
    #[test]
    fn genesis_challenges_are_non_zero() {
        assert_ne!(DIG_MAINNET.genesis_challenge(), Bytes32::new([0u8; 32]));
        assert_ne!(DIG_TESTNET.genesis_challenge(), Bytes32::new([0u8; 32]));
    }

    /// The mainnet genesis is pinned to the Chia mainnet header hash @ height
    /// 9,021,277 (a real anchored value), and the testnet genesis is the
    /// reproducible `sha256` of its documented preimage. These pin both values
    /// byte-for-byte so neither can silently drift.
    #[test]
    fn genesis_challenges_are_the_pinned_values() {
        assert_eq!(
            DIG_MAINNET_GENESIS_CHALLENGE,
            hex_literal::hex!("0af981862a4df51f51ec59c312315d959931d917c375730b89b9e2b0854d1abf"),
        );
        assert_eq!(
            DIG_TESTNET_GENESIS_CHALLENGE,
            sha256(b"DIG_TESTNET:genesis:v1"),
        );
    }

    /// Mainnet and testnet MUST NOT share a genesis (no cross-network replay).
    #[test]
    fn mainnet_and_testnet_genesis_differ() {
        assert_ne!(
            DIG_MAINNET.genesis_challenge(),
            DIG_TESTNET.genesis_challenge(),
        );
    }

    /// Pins the $DIG CAT asset id byte-for-byte against the value shipped in
    /// `chip35_dl_coin::DIG_ASSET_ID` — a drift here silently breaks $DIG
    /// recognition across every consumer (wallets, decoders, payment builders).
    #[test]
    fn dig_asset_id_is_canonical() {
        assert_eq!(
            DIG_ASSET_ID,
            Bytes32::new(hex_literal::hex!(
                "a406d3a9de984d03c9591c10d917593b434d5263cabe2b42f6b367df16832f81"
            )),
        );
    }

    // -- Chia L1 AGG_SIG_ME anti-drift guards ------------------------------

    /// Literal pin: the Chia L1 AGG_SIG_ME constants equal Chia's well-known
    /// mainnet / testnet11 genesis challenges byte-for-byte. This catches any
    /// accidental mutation independently of any external crate.
    #[test]
    fn chia_l1_agg_sig_me_constants_are_the_pinned_values() {
        assert_eq!(
            CHIA_L1_MAINNET_AGG_SIG_ME,
            hex_literal::hex!("ccd5bb71183532bff220ba46c268991a3ff07eb358e8255a65c30a2dce0e5fbb"),
        );
        assert_eq!(
            CHIA_L1_TESTNET11_AGG_SIG_ME,
            hex_literal::hex!("37a90eb5185a9c4439a91ddc98bbadce7b4feba060d50116a067de66bf236615"),
        );
    }

    /// Source KAT: the Chia L1 constants MUST equal the values `chia-wallet-sdk`
    /// (via `chia-sdk-types`) uses in its `MAINNET_CONSTANTS` / `TESTNET11_CONSTANTS`.
    /// This is the primary anti-drift guard — the wallet engine binds spends with
    /// those SDK constants, so if a future SDK version ever changed the value, this
    /// fails and forces a deliberate re-pin instead of a silent custody break.
    #[test]
    fn chia_l1_agg_sig_me_matches_chia_sdk_types() {
        use chia_sdk_types::{MAINNET_CONSTANTS, TESTNET11_CONSTANTS};
        assert_eq!(
            CHIA_L1_MAINNET_AGG_SIG_ME.as_slice(),
            MAINNET_CONSTANTS.agg_sig_me_additional_data.as_ref(),
        );
        assert_eq!(
            CHIA_L1_TESTNET11_AGG_SIG_ME.as_slice(),
            TESTNET11_CONSTANTS.agg_sig_me_additional_data.as_ref(),
        );
    }

    /// The Chia L1 (foreign chain) AGG_SIG_ME MUST NOT equal the DIG L2 genesis —
    /// this is the whole reason the constants exist. Signing an L1 spend with the
    /// DIG L2 genesis would be a custody break.
    #[test]
    fn chia_l1_agg_sig_me_differs_from_dig_l2_genesis() {
        assert_ne!(
            Bytes32::new(CHIA_L1_MAINNET_AGG_SIG_ME),
            DIG_MAINNET.genesis_challenge(),
        );
        assert_ne!(
            Bytes32::new(CHIA_L1_TESTNET11_AGG_SIG_ME),
            DIG_TESTNET.genesis_challenge(),
        );
    }

    // -- DIG treasury recipient anti-drift guards --------------------------

    /// Literal pin: the treasury inner puzzle hash equals the value
    /// `digstore_chain::dig::treasury_inner_puzzle_hash()` decodes to
    /// (byte-identical, pinned by that crate's own test at
    /// `crates/digstore-chain/src/dig.rs:206-209`). A drift here silently
    /// MISDIRECTS every $DIG capsule/commit payment and dev-tip to the wrong
    /// on-chain recipient — a custody break.
    #[test]
    fn dig_treasury_inner_puzzle_hash_is_canonical() {
        assert_eq!(
            DIG_TREASURY_INNER_PUZZLE_HASH,
            Bytes32::new(hex_literal::hex!(
                "ec7c304708c7d59c078d5ae098d0dea004decf47fa1cafebb266c10ad6466ce8"
            )),
        );
    }

    /// Literal pin: the treasury address equals digstore-chain's
    /// source-of-truth bech32m form (`digstore_chain::dig::TREASURY_ADDRESS`,
    /// `crates/digstore-chain/src/dig.rs:41`). A drift misdirects funds.
    #[test]
    fn dig_treasury_address_is_canonical() {
        assert_eq!(
            DIG_TREASURY_ADDRESS,
            "xch1a37rq3cgcl2ecpudttsf35x75qzdan68lgw2l6ajvmqs44jxdn5qv6pk3y",
        );
    }

    /// KAT: the bech32m address and the inner puzzle hash cannot silently drift
    /// apart. Decodes `DIG_TREASURY_ADDRESS` (HRP `xch`, bech32m) and asserts
    /// the 32 decoded bytes equal `DIG_TREASURY_INNER_PUZZLE_HASH`, proving the
    /// two constants encode the SAME on-chain recipient.
    #[test]
    fn dig_treasury_address_decodes_to_inner_puzzle_hash() {
        use bech32::Hrp;
        let (hrp, data) = bech32::decode(DIG_TREASURY_ADDRESS).expect("valid bech32m");
        assert_eq!(hrp, Hrp::parse("xch").unwrap(), "HRP must be xch");
        assert_eq!(
            data.as_slice(),
            DIG_TREASURY_INNER_PUZZLE_HASH.to_bytes(),
            "address must decode to the pinned inner puzzle hash",
        );
    }

    // -- Profile DEK at-rest byte-contract guards ---------------------------
    //
    // These pin every DEK-derivation constant literally so a future edit can't
    // silently drift the contract (which would make every already-sealed
    // profile permanently unreadable, §5.1).

    #[test]
    fn dek_salt_is_the_pinned_value() {
        assert_eq!(DEK_SALT, b"dig-app:dek-salt:v1");
    }

    #[test]
    fn identity_ikm_version_is_the_pinned_value() {
        assert_eq!(IDENTITY_IKM_VERSION, 2);
    }

    #[test]
    fn profile_dek_label_is_the_pinned_value() {
        assert_eq!(PROFILE_DEK_LABEL, b"dig-app:profile-dek:v2");
    }

    #[test]
    fn symmetric_key_len_is_the_pinned_value() {
        assert_eq!(SYMMETRIC_KEY_LEN, 32);
    }

    /// Every baked-in AGG_SIG additional-data value MUST equal the §4.1 rule
    /// applied to the network's genesis: AGG_SIG_ME == genesis, and each other
    /// variant == `sha256(genesis || opcode_byte)`. This regenerates the values
    /// independently and asserts the constants match — so a genesis change that
    /// forgets to recompute a derived value is caught.
    #[test]
    fn agg_sig_additional_data_matches_derivation_rule() {
        for net in [&DIG_MAINNET, &DIG_TESTNET] {
            let genesis = net.genesis_challenge();
            assert_eq!(net.agg_sig_me_additional_data(), genesis);

            let c = net.consensus();
            let derived: Vec<Bytes32> = AGG_SIG_OPCODES
                .iter()
                .map(|&op| {
                    let mut preimage = genesis.as_ref().to_vec();
                    preimage.push(op);
                    Bytes32::new(sha256(&preimage))
                })
                .collect();
            assert_eq!(c.agg_sig_parent_additional_data, derived[0]);
            assert_eq!(c.agg_sig_puzzle_additional_data, derived[1]);
            assert_eq!(c.agg_sig_amount_additional_data, derived[2]);
            assert_eq!(c.agg_sig_puzzle_amount_additional_data, derived[3]);
            assert_eq!(c.agg_sig_parent_amount_additional_data, derived[4]);
            assert_eq!(c.agg_sig_parent_puzzle_additional_data, derived[5]);
        }
    }
}
