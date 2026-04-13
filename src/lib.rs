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
// TODO: Replace placeholder genesis challenge with the real DIG mainnet value
// once the network is launched. All agg_sig_*_additional_data values must be
// recomputed as sha256(genesis_challenge || opcode_byte) at that time.
// ---------------------------------------------------------------------------

/// Placeholder DIG mainnet genesis challenge.
///
/// This MUST be replaced with the real value before mainnet launch.
/// All `agg_sig_*_additional_data` fields are derived from this.
const DIG_MAINNET_GENESIS_CHALLENGE: [u8; 32] =
    hex!("0000000000000000000000000000000000000000000000000000000000000000");

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
            "978722459e638504a3c4ed25b0eae952f1cba668de5a44ccbb3b311eb6901218"
        )),
        agg_sig_puzzle_additional_data: Bytes32::new(hex!(
            "b5b75cf3f16babd124b3c36ac239db038cf9384b6f4343ab65121e7994fa87e4"
        )),
        agg_sig_amount_additional_data: Bytes32::new(hex!(
            "568b7e86b93e78c4a70a90902134266d5f666400d449c827c32422c14a8df42a"
        )),
        agg_sig_puzzle_amount_additional_data: Bytes32::new(hex!(
            "73f82ca7a07025c76c91a3faf2e574ffa13759597fc5d9a0573b4df70245de2e"
        )),
        agg_sig_parent_amount_additional_data: Bytes32::new(hex!(
            "3a2914bb834c69f745c1932450bad277f975b3e1b246d003e3a53e550cf74936"
        )),
        agg_sig_parent_puzzle_additional_data: Bytes32::new(hex!(
            "4f59298d607f3143532ed694fb2f10454a684c1a29ef83e250bf5e234c6720b7"
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

// ---------------------------------------------------------------------------
// DIG Testnet
// ---------------------------------------------------------------------------

/// Placeholder DIG testnet genesis challenge.
const DIG_TESTNET_GENESIS_CHALLENGE: [u8; 32] =
    hex!("0000000000000000000000000000000000000000000000000000000000000001");

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
            "6ae3f62deccdc8d56baf955e45dad1d40332a7b8e4afbb38f07719a863658054"
        )),
        agg_sig_puzzle_additional_data: Bytes32::new(hex!(
            "5d962189ce65d3b3799f032add1ab29ef94ebc0e349fe1db231752304cdd6904"
        )),
        agg_sig_amount_additional_data: Bytes32::new(hex!(
            "3724d66f2da5614aa650517a2feb7d681807d5107441c9c72579e9a751b82d67"
        )),
        agg_sig_puzzle_amount_additional_data: Bytes32::new(hex!(
            "fb6a54a5b51e9734a6ff72fe4105cd5db891d4dbaef9361586091f0d4486581b"
        )),
        agg_sig_parent_amount_additional_data: Bytes32::new(hex!(
            "a5556086f1b58dfd5966bf1aac5257c7f60774ffe5a9e219919c56640993f68b"
        )),
        agg_sig_parent_puzzle_additional_data: Bytes32::new(hex!(
            "3fcc94e67cf3975473b065f0b21a4e92dd3df498d8b4464d7da9582669ac4e48"
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
