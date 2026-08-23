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
        plot_size_v2: 28,
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
        plot_v1_phase_out_epoch_bits: 8,
        min_plot_strength: 2,
        max_plot_strength: 32,
        plot_filter_v2_first_adjustment_height: 0xffff_ffff,
        plot_filter_v2_second_adjustment_height: 0xffff_ffff,
        plot_filter_v2_third_adjustment_height: 0xffff_ffff,
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

/// The always-on peer anchors a node dials at startup, as `peer_id@host:port`.
///
/// # Why a fresh node needs this
///
/// Every other way a node learns peers already requires having one: peer exchange
/// spreads the peers a live link's far end knows, the DHT answers queries routed
/// through peers already in the table, and a relay reservation only makes a node
/// *reachable*. A node installed onto a machine that has never run one therefore
/// has nothing to dial. This set is the one input that does not presuppose its own
/// output.
///
/// # Why the host is `node-rpc.dig.net` and NOT `rpc.dig.net`
///
/// These are different machines with different jobs, and confusing them ships a
/// dial at a closed port. [`RPC_DIG_NET_URL`] is the §5.3 client→node READ gateway:
/// a CloudFront distribution that terminates HTTPS and cannot carry the mTLS peer
/// protocol — its peer ports are closed. `node-rpc.dig.net` is that distribution's
/// ORIGIN, an instance that answers the peer protocol directly. Both names are
/// legitimate and they MUST NOT be collapsed into one another.
///
/// # Why each entry carries an identity
///
/// The node↔node interface is mTLS with the peer's certificate pinned by
/// `peer_id = SHA-256(TLS SPKI DER)`, so an address alone is not dialable. An entry
/// without an identity could only be dialled unpinned — accepting whatever answered
/// at that address, which is exactly what the pinning exists to deny.
///
/// # A bootstrap peer is NOT a trusted peer
///
/// Being well-known is not being trusted. An anchor gets no trust flag, bypasses no
/// corroboration, and counts as exactly one voice — the same as any peer learned by
/// exchange. Consumers MUST treat it as untrusted (NC-12) and MUST tolerate every
/// entry here being unreachable: a node whose bootstrap dials all fail is still a
/// working node, and a hard dependency on one host would make it a single point of
/// failure for every fresh node in the network.
pub const DIG_BOOTSTRAP_PEERS: &[&str] =
    &["741592c0e1e1e9b1a02d3e0bb165bfe54b7adbb5878a3c5de59893949524b68f@node-rpc.dig.net:9444"];

// =============================================================================
// DIG bootstrap peers
//
// A node with no relay reservation and no remembered peers has nothing to dial
// and forms zero connections. The bootstrap set is the always-on anchor that
// gives such a node its first dialable peer, exactly as Chia's introducers seed
// a fresh full node.
// =============================================================================

/// One always-on peer a DIG node dials at startup to obtain its FIRST peers.
///
/// Carries both halves a dial needs, because the node↔node mTLS peer interface
/// pins the peer's certificate SPKI against an EXPECTED identity: an address
/// alone is not dialable on that interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BootstrapPeer {
    /// The peer's 64-hex `peer_id` (`SHA-256(TLS SPKI DER)`), which the mTLS
    /// handshake pins the presented certificate against.
    ///
    /// `None` for an entry whose identity is not yet known. A node SKIPS such an
    /// entry rather than dialing it unpinned — an unpinned dial would accept
    /// whatever identity answered at that address, which is the property the
    /// pinning exists to deny.
    pub peer_id_hex: Option<&'static str>,
    /// The `host:port` authority the peer answers on ([`DIG_NODE_PORT`]).
    ///
    /// A hostname resolving to both families is preferred, because peer dialing
    /// is IPv6-first with IPv4 only as a fallback. A literal address MAY be
    /// used, and an IPv6 literal MUST be bracketed — `[2606:4941::1]:9778` — so
    /// the port separator stays unambiguous. An unbracketed IPv6 literal is
    /// rejected: `2606:4941::1:9778` would otherwise parse as the host
    /// `2606:4941::1` on port `9778` purely by coincidence of the last colon.
    pub authority: &'static str,
}

/// The always-on peers a DIG node dials at startup to obtain its FIRST peers.
///
/// A freshly-installed node knows no peers: peer exchange and the DHT can only
/// spread peers a node already has, and a relay reservation only makes the node
/// reachable, never populates its address book. Without a bootstrap set such a
/// node reports `connected_peers = 0` forever.
///
/// Operators override the set with the `DIG_BOOTSTRAP_PEERS` environment
/// variable (a comma-separated `peer_id@host:port` list) or disable it with
/// `DIG_BOOTSTRAP_PEERS=off` for an air-gapped node — the same shape
/// [`DIG_RELAY_URL`] uses.
///
/// # Deliberately EMPTY until the public peer endpoint is deployed
///
/// The DIG node-to-node mTLS peer interface is not yet reachable on any host:
/// the canonical peer authority is `node-rpc.dig.net`, whose infrastructure
/// ships behind a default-off flag and answers on no port today, and
/// `rpc.dig.net` itself is the CloudFront read gateway — it has no peer
/// listener at all. The published `peer_id` likewise exists only once that
/// endpoint presents its certificate.
///
/// So both halves of an entry are facts only the operator of that endpoint can
/// supply, and this set stays EMPTY rather than naming an address that answers
/// nothing. An empty set is honest and consumable — a node dials nothing and
/// falls back to its relay, PEX and DHT paths — whereas a fabricated entry
/// spends every startup dialing a dead host and reports the network as
/// unreachable rather than as unconfigured.
///
/// # Membership is NOT a trust grant
///
/// A bootstrap peer is an ordinary UNTRUSTED dialled peer (NC-12). Appearing
/// here means only "a node may learn its first addresses from this host": it
/// confers no authority over chain facts, no exemption from cross-peer
/// agreement, and no privilege over a peer learned via peer exchange. Anything
/// a bootstrap peer says is verified exactly as a stranger's claim is.
pub const DIG_BOOTSTRAP_PEERS: &[BootstrapPeer] = &[];

/// The environment variable an operator uses to override [`DIG_BOOTSTRAP_PEERS`].
///
/// Named here so a consumer never spells it as a literal and drifts from the
/// documented contract.
pub const DIG_BOOTSTRAP_PEERS_ENV: &str = "DIG_BOOTSTRAP_PEERS";

/// The sentinel value of [`DIG_BOOTSTRAP_PEERS_ENV`] that disables bootstrap
/// dialing entirely, for an air-gapped node.
pub const DIG_BOOTSTRAP_PEERS_DISABLED: &str = "off";

/// A bootstrap authority split into the two parts a dial needs.
///
/// Borrows from the input, so parsing a `&'static str` out of
/// [`DIG_BOOTSTRAP_PEERS`] yields a `BootstrapAuthority<'static>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BootstrapAuthority<'a> {
    /// The host, with any IPv6 brackets REMOVED — `[::1]:9778` yields `::1`,
    /// which is the form an address parser and a TLS name check both expect.
    pub host: &'a str,
    /// The TCP port, already parsed and known non-zero.
    pub port: u16,
}

impl BootstrapAuthority<'_> {
    /// Whether this authority is on the canonical [`DIG_NODE_PORT`].
    ///
    /// Kept OUT of parsing on purpose: an entry in [`DIG_BOOTSTRAP_PEERS`] MUST
    /// be on the canonical port, but an operator override MAY legitimately name
    /// a peer on another port, so refusing a non-default port inside the parser
    /// would reject valid operator input.
    pub fn is_on_default_node_port(&self) -> bool {
        self.port == DIG_NODE_PORT
    }
}

/// Why a bootstrap authority could not be parsed.
///
/// Each variant is a DIFFERENT operator mistake with a different fix, so they
/// stay distinguishable rather than collapsing into one bad-input value: an
/// error that cannot say which branch fired sends the operator to re-read the
/// wrong part of their configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootstrapAuthorityError {
    /// The authority was empty.
    Empty,
    /// The value is a URL, not a `host:port` authority (it contains `://`).
    LooksLikeUrl,
    /// A bracketed IPv6 literal opened with `[` and never closed with `]`.
    UnclosedBracket,
    /// An IPv6 literal was written without brackets.
    ///
    /// `2606:4941::1:9778` is ambiguous: a naive split at the last colon reads
    /// it as host `2606:4941::1` on port `9778` purely by coincidence, so it is
    /// refused rather than guessed. Write `[2606:4941::1]:9778`.
    UnbracketedIpv6Literal,
    /// No `:port` was present at all.
    MissingPort,
    /// The host part was empty, as in `:9778`.
    EmptyHost,
    /// The port was not a base-10 `u16`.
    UnparseablePort,
    /// The port parsed as `0`, which is not a dialable port.
    ZeroPort,
}

impl core::fmt::Display for BootstrapAuthorityError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let message = match self {
            Self::Empty => "bootstrap authority is empty",
            Self::LooksLikeUrl => {
                "bootstrap authority must be a host:port authority, not a URL (remove the scheme)"
            }
            Self::UnclosedBracket => {
                "bootstrap authority opens an IPv6 bracket with [ but never closes it with ]"
            }
            Self::UnbracketedIpv6Literal => {
                "bootstrap authority looks like an unbracketed IPv6 literal; write it as [addr]:port"
            }
            Self::MissingPort => "bootstrap authority has no :port",
            Self::EmptyHost => "bootstrap authority has an empty host",
            Self::UnparseablePort => "bootstrap authority port is not a number in 1..=65535",
            Self::ZeroPort => "bootstrap authority port 0 is not dialable",
        };
        f.write_str(message)
    }
}

impl std::error::Error for BootstrapAuthorityError {}

/// Parse a bootstrap `host:port` authority.
///
/// This is the ONE implementation of the authority rule for the whole ecosystem.
/// A consumer parsing the [`DIG_BOOTSTRAP_PEERS_ENV`] override MUST call this
/// rather than re-derive it: the bracketed-IPv6 case is exactly where two
/// implementations drift, because a wrong version differs from the right one on
/// that single input shape and agrees on every other.
///
/// The port is returned as parsed and is NOT required to be [`DIG_NODE_PORT`];
/// see [`BootstrapAuthority::is_on_default_node_port`] for why.
///
/// # Examples
///
/// ```
/// use dig_constants::{parse_bootstrap_authority, BootstrapAuthorityError};
///
/// let parsed = parse_bootstrap_authority("[2606:4941::1]:9778").unwrap();
/// assert_eq!(parsed.host, "2606:4941::1"); // brackets stripped
/// assert_eq!(parsed.port, 9778);
///
/// assert_eq!(
///     parse_bootstrap_authority("2606:4941::1:9778"),
///     Err(BootstrapAuthorityError::UnbracketedIpv6Literal)
/// );
/// ```
pub fn parse_bootstrap_authority(
    authority: &str,
) -> Result<BootstrapAuthority<'_>, BootstrapAuthorityError> {
    if authority.is_empty() {
        return Err(BootstrapAuthorityError::Empty);
    }
    if authority.contains("://") {
        return Err(BootstrapAuthorityError::LooksLikeUrl);
    }

    let (host, port_text) = match authority.strip_prefix('[') {
        Some(rest) => {
            let (host, after) = rest
                .split_once(']')
                .ok_or(BootstrapAuthorityError::UnclosedBracket)?;
            let port_text = after
                .strip_prefix(':')
                .ok_or(BootstrapAuthorityError::MissingPort)?;
            (host, port_text)
        }
        None => {
            let (host, port_text) = authority
                .rsplit_once(':')
                .ok_or(BootstrapAuthorityError::MissingPort)?;
            if host.contains(':') {
                return Err(BootstrapAuthorityError::UnbracketedIpv6Literal);
            }
            (host, port_text)
        }
    };

    if host.is_empty() {
        return Err(BootstrapAuthorityError::EmptyHost);
    }
    let port: u16 = port_text
        .parse()
        .map_err(|_| BootstrapAuthorityError::UnparseablePort)?;
    if port == 0 {
        return Err(BootstrapAuthorityError::ZeroPort);
    }
    Ok(BootstrapAuthority { host, port })
}

/// Whether a string is a well-formed DIG `peer_id`: exactly 64 hex characters
/// (`SHA-256(TLS SPKI DER)`, hex-encoded).
///
/// Exported because the mTLS dial pins against this value, so a malformed one
/// does not fail loudly — it makes an entry permanently unusable while still
/// LOOKING configured, which an operator cannot distinguish from an unreachable
/// peer.
pub fn is_wellformed_peer_id(peer_id_hex: &str) -> bool {
    peer_id_hex.len() == 64 && peer_id_hex.chars().all(|c| c.is_ascii_hexdigit())
}

/// A parsed override entry: an optional pinned identity plus its authority.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParsedBootstrapPeer<'a> {
    /// The pinned `peer_id`, or `None` when the entry named an address only.
    ///
    /// An entry with `None` MUST be skipped rather than dialed unpinned.
    pub peer_id_hex: Option<&'a str>,
    /// The parsed authority.
    pub authority: BootstrapAuthority<'a>,
}

/// Why a bootstrap override entry could not be parsed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootstrapPeerError {
    /// The `peer_id@` part was present but empty.
    EmptyPeerId,
    /// The `peer_id@` part was not 64 hex characters.
    MalformedPeerId,
    /// The authority half was not parseable; carries the specific reason.
    Authority(BootstrapAuthorityError),
}

impl core::fmt::Display for BootstrapPeerError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::EmptyPeerId => f.write_str("bootstrap entry has an empty peer_id before @"),
            Self::MalformedPeerId => {
                f.write_str("bootstrap peer_id must be exactly 64 hex characters")
            }
            Self::Authority(inner) => write!(f, "{inner}"),
        }
    }
}

impl std::error::Error for BootstrapPeerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Authority(inner) => Some(inner),
            _ => None,
        }
    }
}

impl From<BootstrapAuthorityError> for BootstrapPeerError {
    fn from(error: BootstrapAuthorityError) -> Self {
        Self::Authority(error)
    }
}

/// Parse ONE entry of the [`DIG_BOOTSTRAP_PEERS_ENV`] override, in the
/// documented `peer_id@host:port` form.
///
/// A bare `host:port` with no `peer_id@` is accepted and yields
/// `peer_id_hex: None` — the caller then SKIPS it rather than dialing unpinned.
///
/// The caller splits the variable on `,` and handles the
/// [`DIG_BOOTSTRAP_PEERS_DISABLED`] sentinel; this function owns the per-entry
/// grammar, which is the part two implementations would otherwise disagree on.
///
/// # Examples
///
/// ```
/// use dig_constants::{parse_bootstrap_peer, BootstrapPeerError};
///
/// let id = "a".repeat(64);
/// let entry = format!("{id}@[::1]:9778");
/// let parsed = parse_bootstrap_peer(&entry).unwrap();
/// assert_eq!(parsed.peer_id_hex, Some(id.as_str()));
/// assert_eq!(parsed.authority.host, "::1");
///
/// // A short identity is reported as a bad identity, not as a bad address.
/// assert_eq!(
///     parse_bootstrap_peer("abc@node-rpc.dig.net:9778"),
///     Err(BootstrapPeerError::MalformedPeerId)
/// );
/// ```
pub fn parse_bootstrap_peer(entry: &str) -> Result<ParsedBootstrapPeer<'_>, BootstrapPeerError> {
    let (peer_id_hex, authority_text) = match entry.split_once('@') {
        Some((peer_id_hex, authority_text)) => {
            if peer_id_hex.is_empty() {
                return Err(BootstrapPeerError::EmptyPeerId);
            }
            if !is_wellformed_peer_id(peer_id_hex) {
                return Err(BootstrapPeerError::MalformedPeerId);
            }
            (Some(peer_id_hex), authority_text)
        }
        None => (None, entry),
    };
    Ok(ParsedBootstrapPeer {
        peer_id_hex,
        authority: parse_bootstrap_authority(authority_text)?,
    })
}

impl BootstrapPeer {
    /// Parse this entry's authority.
    ///
    /// Returned as a `Result` rather than assumed valid so a consumer handles a
    /// canonical entry and an operator-supplied one through one code path; the
    /// crate test suite pins every canonical entry as parseable.
    pub fn parse_authority(&self) -> Result<BootstrapAuthority<'static>, BootstrapAuthorityError> {
        parse_bootstrap_authority(self.authority)
    }

    /// Whether this entry carries a pinned identity and may therefore be dialed.
    ///
    /// An entry without one is SKIPPED: an unpinned dial accepts whatever
    /// identity answers at the address, which is the property pinning denies.
    pub fn is_dialable(&self) -> bool {
        self.peer_id_hex.is_some_and(is_wellformed_peer_id)
    }
}

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
        plot_size_v2: 28,
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
        plot_v1_phase_out_epoch_bits: 8,
        min_plot_strength: 2,
        max_plot_strength: 32,
        plot_filter_v2_first_adjustment_height: 0xffff_ffff,
        plot_filter_v2_second_adjustment_height: 0xffff_ffff,
        plot_filter_v2_third_adjustment_height: 0xffff_ffff,
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

// =============================================================================
// Profile sealing X25519 byte contract
//
// A DIG user profile's per-profile X25519 *sealing* keypair (used by the DIG
// App to seal/unseal `DIGCHAT1` messages for dig-chat, §NC-1 end-to-end
// encryption) is derived, never stored, from the same identity scalar as the
// DEK — but under a DISTINCT HKDF `info` label, which is the sole thing that
// domain-separates the sealing key from the at-rest DEK:
//
//   HKDF-SHA256(salt = DEK_SALT,
//               ikm  = IDENTITY_IKM_VERSION || identity_scalar_32,
//               info = PROFILE_SEALING_X25519_LABEL)
//     -> SYMMETRIC_KEY_LEN (32) bytes, then CLAMPED to an X25519 scalar
//
// This label reuses the already-frozen DEK_SALT + IDENTITY_IKM_VERSION on
// purpose; only the `info` label differs. The 32-byte HKDF output is clamped
// to a valid X25519 secret scalar by the CONSUMER (dig-account) — this crate
// owns ONLY the frozen label bytes, not the clamp/derivation.
//
// The label is a PERMANENT byte-identical contract (§4.1/§5.1/NC-1): every
// message already sealed on the network was encrypted under a keypair derived
// from EXACTLY these bytes. Changing it re-derives a different keypair and
// makes every already-sealed message permanently unopenable — there is no
// migration path for a derived (never-stored) key. Treat it as frozen; only
// ever ADD a new version-scoped label (`…:v2`) alongside it, never mutate it.
//
// Consumers (this crate is their single source of truth — do not duplicate the
// literal locally):
//   - dig-account: derives the sealing keypair via
//     `seed.profile_derive_symmetric_key(ix, PROFILE_SEALING_X25519_LABEL)`
//     then clamps the output to an X25519 scalar.
// =============================================================================

/// HKDF info/label for deriving a profile's per-profile X25519 **sealing**
/// keypair — the key the DIG App uses to seal/unseal `DIGCHAT1` messages.
///
/// Part of the frozen [profile sealing X25519 byte
/// contract](self#profile-sealing-x25519-byte-contract) — see the section
/// comment above. Reuses [`DEK_SALT`] + [`IDENTITY_IKM_VERSION`]; this distinct
/// `info` label is what domain-separates the sealing key from [`PROFILE_DEK_LABEL`].
/// The 32-byte HKDF output is clamped to an X25519 scalar by the consumer
/// (dig-account), not here.
pub const PROFILE_SEALING_X25519_LABEL: &[u8] = b"dig-app:profile-sealing-x25519:v1";

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

    /// The authority shape MUST be able to express an IPv6 endpoint.
    ///
    /// Peer dialing is IPv6-first, so a representation that cannot carry an IPv6
    /// address at all would be wrong on arrival — and the failure would surface
    /// only on an IPv6-only host, long after the constant shipped. Also pins the
    /// bracket STRIPPING, since a host handed onward as `[::1]` fails every
    /// address parse and TLS name check downstream.
    #[test]
    fn parses_a_bracketed_ipv6_endpoint_and_strips_the_brackets() {
        let parsed = parse_bootstrap_authority("[2606:4941::1]:9778").expect("must parse");
        assert_eq!(parsed.host, "2606:4941::1");
        assert_eq!(parsed.port, 9778);
        assert!(parsed.is_on_default_node_port());

        let loopback = parse_bootstrap_authority("[::1]:9778").expect("must parse");
        assert_eq!(loopback.host, "::1");
    }

    /// Hostnames and IPv4 literals parse unchanged.
    #[test]
    fn parses_hostname_and_ipv4_authorities() {
        let named = parse_bootstrap_authority("node-rpc.dig.net:9778").expect("must parse");
        assert_eq!((named.host, named.port), ("node-rpc.dig.net", 9778));

        let v4 = parse_bootstrap_authority("44.217.228.224:9778").expect("must parse");
        assert_eq!((v4.host, v4.port), ("44.217.228.224", 9778));
    }

    /// Every rejection MUST name the specific operator mistake.
    ///
    /// These are four different fixes — bracket the address, add a port, correct
    /// a typo, supply a host — so a single shared bad-input error would send the
    /// operator to re-read the wrong part of their configuration. Asserting the
    /// exact variant is what stops a later refactor collapsing them.
    #[test]
    fn each_rejection_names_its_own_specific_mistake() {
        use BootstrapAuthorityError as E;
        assert_eq!(parse_bootstrap_authority(""), Err(E::Empty));
        assert_eq!(
            parse_bootstrap_authority("https://node-rpc.dig.net:9778"),
            Err(E::LooksLikeUrl)
        );
        assert_eq!(
            parse_bootstrap_authority("[2606:4941::1:9778"),
            Err(E::UnclosedBracket)
        );
        assert_eq!(
            parse_bootstrap_authority("2606:4941::1:9778"),
            Err(E::UnbracketedIpv6Literal),
            "an unbracketed IPv6 literal is ambiguous and must be rejected AS SUCH"
        );
        assert_eq!(
            parse_bootstrap_authority("node-rpc.dig.net"),
            Err(E::MissingPort)
        );
        assert_eq!(parse_bootstrap_authority("[::1]"), Err(E::MissingPort));
        assert_eq!(parse_bootstrap_authority(":9778"), Err(E::EmptyHost));
        assert_eq!(
            parse_bootstrap_authority("node-rpc.dig.net:http"),
            Err(E::UnparseablePort)
        );
        assert_eq!(
            parse_bootstrap_authority("node-rpc.dig.net:99999"),
            Err(E::UnparseablePort),
            "a port above u16 range is unparseable, not merely non-canonical"
        );
        assert_eq!(
            parse_bootstrap_authority("node-rpc.dig.net:0"),
            Err(E::ZeroPort)
        );
    }

    /// Every variant renders a message that identifies its own branch.
    ///
    /// A distinguishable enum whose `Display` collapses to one sentence is the
    /// same defect wearing a type: the operator reads the message, not the
    /// variant.
    #[test]
    fn every_authority_error_renders_a_distinct_message() {
        use BootstrapAuthorityError as E;
        let all = [
            E::Empty,
            E::LooksLikeUrl,
            E::UnclosedBracket,
            E::UnbracketedIpv6Literal,
            E::MissingPort,
            E::EmptyHost,
            E::UnparseablePort,
            E::ZeroPort,
        ];
        let rendered: Vec<String> = all.iter().map(|e| e.to_string()).collect();
        for (index, message) in rendered.iter().enumerate() {
            assert!(
                !message.is_empty(),
                "variant {index} renders an empty message"
            );
            assert_eq!(
                rendered.iter().filter(|other| *other == message).count(),
                1,
                "message {message:?} is shared by more than one variant"
            );
        }
    }

    /// The canonical port is NOT enforced by the parser, and that is deliberate.
    ///
    /// An operator override may name a peer on another port. Pinned from both
    /// sides so neither half can drift: the canonical port reports `true`, a
    /// neighbouring port parses fine and reports `false`.
    #[test]
    fn default_node_port_is_reported_not_enforced() {
        let canonical = parse_bootstrap_authority("node-rpc.dig.net:9778").expect("must parse");
        assert!(canonical.is_on_default_node_port());

        let other = parse_bootstrap_authority("node-rpc.dig.net:9779").expect("must parse");
        assert_eq!(other.port, 9779);
        assert!(
            !other.is_on_default_node_port(),
            "a non-canonical port must be reported as such, or the check is vacuous"
        );
    }

    /// A `peer_id` MUST be exactly 64 hex characters.
    ///
    /// The dial pins the presented certificate against this value, so a
    /// malformed one does not fail loudly — it makes the entry permanently
    /// unusable while still LOOKING configured. Pinned from both sides: 64
    /// passes, 63 and 65 fail.
    #[test]
    fn peer_id_must_be_exactly_64_hex_chars() {
        assert!(is_wellformed_peer_id(&"a".repeat(64)));
        assert!(is_wellformed_peer_id(&"0F".repeat(32)));
        assert!(!is_wellformed_peer_id(&"a".repeat(63)));
        assert!(!is_wellformed_peer_id(&"a".repeat(65)));
        assert!(!is_wellformed_peer_id(&format!("{}z", "a".repeat(63))));
        assert!(!is_wellformed_peer_id(""));
    }

    /// The override entry grammar parses `peer_id@host:port`.
    #[test]
    fn parses_a_pinned_override_entry() {
        let id = "a".repeat(64);
        let entry = format!("{id}@[2606:4941::1]:9778");
        let parsed = parse_bootstrap_peer(&entry).expect("must parse");
        assert_eq!(parsed.peer_id_hex, Some(id.as_str()));
        assert_eq!(parsed.authority.host, "2606:4941::1");
        assert_eq!(parsed.authority.port, 9778);
    }

    /// A bare authority is accepted and yields no identity, so the caller skips
    /// it rather than dialing unpinned.
    #[test]
    fn override_entry_without_an_identity_yields_none() {
        let parsed = parse_bootstrap_peer("node-rpc.dig.net:9778").expect("must parse");
        assert_eq!(parsed.peer_id_hex, None);
        assert_eq!(parsed.authority.host, "node-rpc.dig.net");
    }

    /// A bad identity and a bad address are DIFFERENT errors.
    ///
    /// Reporting a malformed identity as a bad address, or vice versa, points
    /// the operator at the wrong half of the entry they typed.
    #[test]
    fn override_entry_errors_separate_identity_from_address() {
        use BootstrapAuthorityError as A;
        use BootstrapPeerError as E;
        assert_eq!(
            parse_bootstrap_peer("abc@node-rpc.dig.net:9778"),
            Err(E::MalformedPeerId)
        );
        assert_eq!(
            parse_bootstrap_peer("@node-rpc.dig.net:9778"),
            Err(E::EmptyPeerId)
        );
        let id = "a".repeat(64);
        assert_eq!(
            parse_bootstrap_peer(&format!("{id}@2606:4941::1:9778")),
            Err(E::Authority(A::UnbracketedIpv6Literal)),
            "an address fault must surface the SPECIFIC authority reason, not a generic one"
        );
        assert_eq!(
            parse_bootstrap_peer(&format!("{id}@node-rpc.dig.net")),
            Err(E::Authority(A::MissingPort))
        );
    }

    /// The disable sentinel and the env-var name are pinned, since a consumer
    /// spelling either as a literal would drift from the documented contract.
    #[test]
    fn bootstrap_override_names_are_canonical() {
        assert_eq!(DIG_BOOTSTRAP_PEERS_ENV, "DIG_BOOTSTRAP_PEERS");
        assert_eq!(DIG_BOOTSTRAP_PEERS_DISABLED, "off");
    }

    /// An entry is dialable only when it carries a WELL-FORMED pinned identity.
    ///
    /// Three cases, because the interesting one is the middle: a `Some` holding
    /// a malformed identity must NOT count as dialable, or a typo becomes an
    /// unpinned dial.
    #[test]
    fn only_a_wellformed_pinned_entry_is_dialable() {
        let unpinned = BootstrapPeer {
            peer_id_hex: None,
            authority: "node-rpc.dig.net:9778",
        };
        assert!(!unpinned.is_dialable());

        let malformed = BootstrapPeer {
            peer_id_hex: Some("abc"),
            authority: "node-rpc.dig.net:9778",
        };
        assert!(
            !malformed.is_dialable(),
            "a malformed identity must not authorise a dial"
        );

        let valid_id = "a".repeat(64);
        let pinned = BootstrapPeer {
            peer_id_hex: Some(Box::leak(valid_id.into_boxed_str())),
            authority: "node-rpc.dig.net:9778",
        };
        assert!(pinned.is_dialable());
        assert_eq!(
            pinned.parse_authority().expect("must parse").port,
            DIG_NODE_PORT
        );
    }

    /// Every entry actually present in the canonical set MUST satisfy both
    /// shapes, on the canonical node port.
    ///
    /// The set is EMPTY today (the public peer endpoint is not deployed), so
    /// this test is a live guard for the entries an operator adds later, not the
    /// proof that the guard works — the tests above are that proof, because they
    /// drive the same exported functions with real valid and real invalid input.
    #[test]
    fn every_canonical_bootstrap_entry_is_wellformed() {
        for entry in DIG_BOOTSTRAP_PEERS {
            let parsed = entry
                .parse_authority()
                .unwrap_or_else(|e| panic!("bootstrap authority {}: {e}", entry.authority));
            assert!(
                parsed.is_on_default_node_port(),
                "bootstrap authority {} must use the canonical node port",
                entry.authority
            );
            if let Some(hex) = entry.peer_id_hex {
                assert!(
                    is_wellformed_peer_id(hex),
                    "bootstrap peer_id for {} is not 64 hex chars",
                    entry.authority
                );
            }
        }
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

    // -- Bootstrap anchor guards -------------------------------------------
    //
    // The bootstrap set is the one peer input that does not presuppose its own
    // output, so a wrong value here strands every fresh node in the network.
    // The two failure modes it can have are BOTH the value itself rather than
    // the mechanism that consumes it, which is why they are pinned here.

    /// The bootstrap set names the PEER interface host, never the read gateway.
    ///
    /// `rpc.dig.net` is CloudFront (distribution `E3L33T1REWMUIK`): it terminates
    /// HTTPS and cannot carry the mTLS peer protocol, and its :9444/:9445 are
    /// closed. The peer interface is live on `node-rpc.dig.net`, the CloudFront
    /// ORIGIN. The two hosts differ by one label, so this asserts the gateway host
    /// is ABSENT rather than merely asserting some host is present — a test that
    /// only checked "a bootstrap entry exists" passes with the closed-port host in
    /// it, which is the nearest wrong value and the one #923's own premise names.
    #[test]
    fn bootstrap_peers_name_the_peer_host_not_the_read_gateway() {
        assert!(
            !DIG_BOOTSTRAP_PEERS.is_empty(),
            "a fresh node with no bootstrap anchor has nothing to dial"
        );
        for entry in DIG_BOOTSTRAP_PEERS {
            let authority = entry.split_once('@').expect("pinned identity").1;
            let host = authority.rsplit_once(':').expect("explicit port").0;
            assert_ne!(
                host, "rpc.dig.net",
                "{entry} dials the CloudFront read gateway, whose peer ports are closed"
            );
            assert!(
                host.ends_with(".dig.net"),
                "{entry} must anchor on a DIG-operated host"
            );
        }
    }

    /// Every bootstrap entry is expressible in the operator override's own syntax:
    /// a 64-hex pinned identity, then `@host:port`.
    ///
    /// The identity half is load-bearing rather than decorative. The peer transport
    /// pins `peer_id = SHA-256(TLS SPKI DER)`, so an entry carrying no identity
    /// would either be skipped (no anchor) or dialled unpinned (accepting whatever
    /// answered at that address) — the exact outcome the pinning exists to deny.
    #[test]
    fn every_bootstrap_entry_carries_a_pinned_identity_and_an_explicit_port() {
        for entry in DIG_BOOTSTRAP_PEERS {
            let (peer_id, authority) = entry
                .split_once('@')
                .unwrap_or_else(|| panic!("{entry} carries no pinned identity"));
            assert_eq!(peer_id.len(), 64, "{entry}: peer_id must be 64 hex chars");
            assert!(
                peer_id.chars().all(|c| c.is_ascii_hexdigit()),
                "{entry}: peer_id must be hex"
            );
            let (host, port) = authority
                .rsplit_once(':')
                .unwrap_or_else(|| panic!("{entry} carries no explicit port"));
            assert!(!host.is_empty(), "{entry}: empty host");
            assert!(
                port.parse::<u16>().is_ok(),
                "{entry}: port must be a u16, got {port}"
            );
        }
    }

    /// The anchor is reachable on the peer port, not on the local-RPC port.
    ///
    /// `DIG_NODE_PORT` (9778) is the §5.3 client→node READ port and is a different
    /// role entirely; an anchor published on it would be dialled by the peer stack
    /// and never answer. Asserting the inequality keeps the two roles from
    /// collapsing into one another the way the two hostnames already can.
    #[test]
    fn bootstrap_port_is_the_peer_port_not_the_local_rpc_port() {
        for entry in DIG_BOOTSTRAP_PEERS {
            let port: u16 = entry
                .rsplit_once(':')
                .expect("explicit port")
                .1
                .parse()
                .expect("numeric port");
            assert_ne!(
                port, DIG_NODE_PORT,
                "{entry} dials the local JSON-RPC port, not the peer port"
            );
            assert_eq!(port, 9444, "{entry} must dial the DIG peer port");
        }
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

    // -- Independent genesis + AGG_SIG domain pins (#2316) -----------------
    //
    // These guard against the ORIGINAL defect class: dig-constants 0.1.0 shipped
    // an all-zeros PLACEHOLDER genesis and its six AGG_SIG additional-data
    // domains were CORRECTLY derived from that placeholder. Any test that
    // re-derives the domains from the crate's OWN genesis (like
    // `agg_sig_additional_data_matches_derivation_rule` below) passes on
    // placeholder data exactly as on real data — the values are
    // "self-consistent-wrong". The finalized value (`0af981…1abf`, 0.4.0) had no
    // independent pin, so it could silently regress to a placeholder again.
    //
    // The guards here break that self-consistency by pinning against a SECOND,
    // INDEPENDENT hardcoded copy of the genesis literal: the AGG_SIG domains are
    // derived FROM THAT LITERAL, not from `net.genesis_challenge()`. A placeholder
    // genesis with placeholder-derived domains (internally consistent) therefore
    // FAILS these tests even though it passes the derivation-rule test. Do NOT
    // change these expectations to read from the const under test — that would
    // reintroduce the self-consistency the pin exists to prevent.

    /// Independent second copy of the finalized DIG mainnet genesis (`0af981…1abf`),
    /// deliberately NOT read from [`DIG_MAINNET_GENESIS_CHALLENGE`] — this is the
    /// external witness the const is checked against.
    const EXPECTED_MAINNET_GENESIS: [u8; 32] =
        hex_literal::hex!("0af981862a4df51f51ec59c312315d959931d917c375730b89b9e2b0854d1abf");

    /// Independent second copy of the DIG testnet genesis (`088c18d6…6c3b`).
    const EXPECTED_TESTNET_GENESIS: [u8; 32] =
        hex_literal::hex!("088c18d6b7859d885dc2f03166e862c958f74b63b6353c3df71d103b9b806c3b");

    /// The mainnet genesis MUST equal the finalized literal, checked at BOTH the
    /// raw const and the public accessor against an independent hardcoded copy.
    /// If someone edits the const back to a placeholder, this fails.
    #[test]
    fn mainnet_genesis_equals_independent_literal() {
        assert_eq!(DIG_MAINNET_GENESIS_CHALLENGE, EXPECTED_MAINNET_GENESIS);
        assert_eq!(
            DIG_MAINNET.genesis_challenge(),
            Bytes32::new(EXPECTED_MAINNET_GENESIS),
        );
    }

    /// The testnet genesis MUST equal its finalized literal, at both the const
    /// and the accessor.
    #[test]
    fn testnet_genesis_equals_independent_literal() {
        assert_eq!(DIG_TESTNET_GENESIS_CHALLENGE, EXPECTED_TESTNET_GENESIS);
        assert_eq!(
            DIG_TESTNET.genesis_challenge(),
            Bytes32::new(EXPECTED_TESTNET_GENESIS),
        );
    }

    /// Belt-and-suspenders: the genesis must not be all-zeros, all-0xFF, or a
    /// trivial counting pattern — the shapes a stub/placeholder tends to take.
    #[test]
    fn mainnet_genesis_is_not_an_obvious_placeholder() {
        let g = DIG_MAINNET_GENESIS_CHALLENGE;
        assert_ne!(g, [0u8; 32], "genesis must not be all-zeros (0.1.0 stub)");
        assert_ne!(g, [0xFFu8; 32], "genesis must not be all-0xFF");
        let counting: [u8; 32] = core::array::from_fn(|i| i as u8);
        assert_ne!(g, counting, "genesis must not be a counting pattern");
        // Not a single repeated byte (e.g. 0x01010101…).
        assert!(
            g.iter().any(|&b| b != g[0]),
            "genesis must not be a single repeated byte",
        );
    }

    /// Independent AGG_SIG-domain pin (the core of the fix). Each of the six DIG
    /// mainnet AGG_SIG additional-data domains MUST equal `sha256(genesis_literal
    /// || opcode_byte)` computed from the INDEPENDENT [`EXPECTED_MAINNET_GENESIS`]
    /// literal (AGG_SIG_ME == the genesis literal directly). Because the expected
    /// values come from a hardcoded copy of the REAL genesis rather than from the
    /// crate's own const, a placeholder genesis whose domains were re-derived from
    /// the placeholder (self-consistent-wrong) FAILS here.
    #[test]
    fn mainnet_agg_sig_domains_equal_independent_literal_derivation() {
        let c = DIG_MAINNET.consensus();
        assert_eq!(
            c.agg_sig_me_additional_data,
            Bytes32::new(EXPECTED_MAINNET_GENESIS),
            "AGG_SIG_ME must equal the genesis literal directly",
        );
        let expected: Vec<Bytes32> = AGG_SIG_OPCODES
            .iter()
            .map(|&op| {
                let mut preimage = EXPECTED_MAINNET_GENESIS.to_vec();
                preimage.push(op);
                Bytes32::new(sha256(&preimage))
            })
            .collect();
        assert_eq!(c.agg_sig_parent_additional_data, expected[0]);
        assert_eq!(c.agg_sig_puzzle_additional_data, expected[1]);
        assert_eq!(c.agg_sig_amount_additional_data, expected[2]);
        assert_eq!(c.agg_sig_puzzle_amount_additional_data, expected[3]);
        assert_eq!(c.agg_sig_parent_amount_additional_data, expected[4]);
        assert_eq!(c.agg_sig_parent_puzzle_additional_data, expected[5]);
    }

    /// The same independent AGG_SIG-domain pin for DIG testnet.
    #[test]
    fn testnet_agg_sig_domains_equal_independent_literal_derivation() {
        let c = DIG_TESTNET.consensus();
        assert_eq!(
            c.agg_sig_me_additional_data,
            Bytes32::new(EXPECTED_TESTNET_GENESIS),
        );
        let expected: Vec<Bytes32> = AGG_SIG_OPCODES
            .iter()
            .map(|&op| {
                let mut preimage = EXPECTED_TESTNET_GENESIS.to_vec();
                preimage.push(op);
                Bytes32::new(sha256(&preimage))
            })
            .collect();
        assert_eq!(c.agg_sig_parent_additional_data, expected[0]);
        assert_eq!(c.agg_sig_puzzle_additional_data, expected[1]);
        assert_eq!(c.agg_sig_amount_additional_data, expected[2]);
        assert_eq!(c.agg_sig_puzzle_amount_additional_data, expected[3]);
        assert_eq!(c.agg_sig_parent_amount_additional_data, expected[4]);
        assert_eq!(c.agg_sig_parent_puzzle_additional_data, expected[5]);
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

    /// The per-profile X25519 sealing label is a PERMANENT crypto byte contract
    /// (§5.1): every `DIGCHAT1` message a DIG user has ever sealed was encrypted
    /// under a sealing key derived from EXACTLY these bytes. A drift here would
    /// re-derive a different keypair and make every already-sealed message
    /// permanently unopenable. This pins the label literally so no future edit
    /// can silently change it.
    #[test]
    fn profile_sealing_x25519_label_is_the_pinned_value() {
        assert_eq!(
            PROFILE_SEALING_X25519_LABEL,
            b"dig-app:profile-sealing-x25519:v1"
        );
    }

    /// The sealing label MUST be distinct from the DEK label — a shared `info`
    /// would derive the same 32 bytes for both the at-rest DEK and the X25519
    /// sealing key, collapsing the domain separation the two labels exist to
    /// provide. This guards that domain separation directly.
    #[test]
    fn profile_sealing_label_is_domain_separated_from_dek_label() {
        assert_ne!(PROFILE_SEALING_X25519_LABEL, PROFILE_DEK_LABEL);
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
