//! The Substrate Node Template runtime. This can be compiled with `#[no_std]`, ready for Wasm.

#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

use parity_codec::{Decode, Encode};
use sr_primitives::traits::{self, BlakeTwo256, Block as BlockT, NumberFor, StaticLookup};
use sr_primitives::transaction_validity::TransactionValidity;
use sr_primitives::{create_runtime_str, generic, ApplyResult};
use sr_std::prelude::*;
use sr_version::RuntimeVersion;
use srml_support::{construct_runtime, parameter_types};
use substrate_client::{
    block_builder::api::{self as block_builder_api, CheckInherentsResult, InherentData},
    impl_runtime_apis, runtime_api,
};
use substrate_primitives::{ed25519, OpaqueMetadata, H256};

/// Used for the module template in `./template.rs`
mod template;

type Block = generic::Block<<Runtime as srml_system::Trait>::Header, UncheckedExtrinsic>;

/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core datastructures.
pub mod opaque {
    use super::*;
    use core::fmt;
    use serde::{Deserialize, Serialize};

    /// Opaque, encoded, unchecked extrinsic.
    #[derive(PartialEq, Eq, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct UncheckedExtrinsic(Vec<u8>);

    impl fmt::Debug for UncheckedExtrinsic {
        fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
            for byte in &self.0 {
                write!(fmt, "{:02x}", byte)?;
            }
            Ok(())
        }
    }

    impl traits::Extrinsic for UncheckedExtrinsic {
        fn is_signed(&self) -> Option<bool> {
            None
        }
    }

    /// Opaque block header type.
    type Header = generic::Header<<Runtime as srml_system::Trait>::BlockNumber, BlakeTwo256>;

    /// Opaque block type.
    pub type Block = generic::Block<Header, UncheckedExtrinsic>;
}

/// This runtime version.
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: create_runtime_str!("node-template"),
    impl_name: create_runtime_str!("node-template"),
    authoring_version: 3,
    spec_version: 4,
    impl_version: 4,
    apis: RUNTIME_API_VERSIONS, // it's not clear where RUNTIME_API_VERSIONS is defined
};

parameter_types! {
    pub const ChainStateCacheSize: <Runtime as srml_system::Trait>::BlockNumber = 250;
}

impl srml_system::Trait for Runtime {
    /// The identifier used to distinguish between accounts.
    type AccountId = ed25519::Public;
    /// The lookup mechanism to get account ID from whatever is passed in dispatchers.
    type Lookup = Indices;
    /// The index type for storing how many extrinsics an account has signed.
    type Index = u64;
    /// The index type for blocks.
    type BlockNumber = u64;
    /// The type for hashing blocks and tries.
    type Hash = H256;
    /// The hashing algorithm used.
    type Hashing = BlakeTwo256;
    /// The header type.
    type Header = generic::Header<Self::BlockNumber, BlakeTwo256>;
    /// The ubiquitous event type.
    type Event = Event;
    /// Update weight (to fee) multiplier per-block.
    type WeightMultiplierUpdate = ();
    /// The ubiquitous origin type.
    type Origin = Origin; // it's unclear where Origin is defined
    /// Maximum number of block number to block hash mappings to keep (oldest pruned first).
    type BlockHashCount = ChainStateCacheSize;
}

impl srml_aura::Trait for Runtime {
    type HandleReport = ();
    type AuthorityId = ed25519::Public;
}

impl srml_indices::Trait for Runtime {
    /// The type for recording indexing into the account enumeration. If this ever overflows, there
    /// will be problems!
    type AccountIndex = u32;
    /// Use the standard means of resolving an index hint from an id.
    type ResolveHint = srml_indices::SimpleResolveHint<Self::AccountId, Self::AccountIndex>;
    /// Determine whether an account is dead.
    type IsDeadAccount = Balances;
    /// The ubiquitous event type.
    type Event = Event;
}

parameter_types! {
    pub const MinimumPeriod: u64 = 5;
}
impl srml_timestamp::Trait for Runtime {
    /// A timestamp: seconds since the unix epoch.
    type Moment = u64;
    type OnTimestampSet = Aura;
    type MinimumPeriod = MinimumPeriod;
}

parameter_types! {
    pub const ExistentialDeposit: u128 = 500;
    pub const TransferFee: u128 = 0;
    pub const CreationFee: u128 = 0;
    pub const TransactionBaseFee: u128 = 1;
    pub const TransactionByteFee: u128 = 0;
}

impl srml_balances::Trait for Runtime {
    /// The type for recording an account's balance.
    type Balance = u128;
    /// What to do if an account's free balance gets zeroed.
    type OnFreeBalanceZero = ();
    /// What to do if a new account is created.
    type OnNewAccount = Indices;
    /// The ubiquitous event type.
    type Event = Event;

    type TransactionPayment = ();
    type DustRemoval = ();
    type TransferPayment = ();
    type ExistentialDeposit = ExistentialDeposit;
    type TransferFee = TransferFee;
    type CreationFee = CreationFee;
    type TransactionBaseFee = TransactionBaseFee;
    type TransactionByteFee = TransactionByteFee;
}

impl srml_sudo::Trait for Runtime {
    /// The ubiquitous event type.
    type Event = Event;
    type Proposal = Call;
}

/// Used for the module template in `./template.rs`
impl template::Trait for Runtime {
    type Event = Event;
}

/// Unchecked extrinsic type as expected by this runtime.
type UncheckedExtrinsic = generic::UncheckedMortalCompactExtrinsic<
    <Indices as StaticLookup>::Source, // it's not clear where Indices is defined
    <Runtime as srml_system::Trait>::Index,
    Call,
    ed25519::Signature,
>;

/// Executive: handles dispatch to the various modules.
type Executive = srml_executive::Executive<
    Runtime,
    Block,
    srml_system::ChainContext<Runtime>,
    Balances,
    Runtime,
    AllModules,
>;

use srml_system as system; // https://github.com/paritytech/substrate/issues/3295
construct_runtime!(
	pub enum Runtime where
		Block = Block,
		NodeBlock = opaque::Block,
		UncheckedExtrinsic = UncheckedExtrinsic
	{
		System: system::{Module, Call, Storage, Config, Event},
		Timestamp: srml_timestamp::{Module, Call, Storage, Inherent},
		Aura: srml_aura::{Module, Config<T>, Inherent(Timestamp)},
		Indices: srml_indices::{default, Config<T>},
		Balances: srml_balances,
		Sudo: srml_sudo,
		TemplateModule: template::{Module, Call, Storage, Event<T>},
	}
);

// Implement our runtime API endpoints. This is just a bunch of proxying.
impl_runtime_apis! {
    impl runtime_api::Core<Block> for Runtime {
        fn version() -> RuntimeVersion {
            VERSION
        }

        fn execute_block(block: Block) {
            Executive::execute_block(block)
        }

        fn initialize_block(header: &<Block as BlockT>::Header) {
            Executive::initialize_block(header)
        }
    }

    impl runtime_api::Metadata<Block> for Runtime {
        fn metadata() -> OpaqueMetadata {
            Runtime::metadata().into()
        }
    }

    impl block_builder_api::BlockBuilder<Block> for Runtime {
        fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyResult {
            Executive::apply_extrinsic(extrinsic)
        }

        fn finalize_block() -> <Block as BlockT>::Header {
            Executive::finalize_block()
        }

        fn inherent_extrinsics(data: InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
            data.create_extrinsics()
        }

        fn check_inherents(block: Block, data: InherentData) -> CheckInherentsResult {
            data.check_extrinsics(&block)
        }

        fn random_seed() -> <Block as BlockT>::Hash {
            System::random_seed()
        }
    }

    impl runtime_api::TaggedTransactionQueue<Block> for Runtime {
        fn validate_transaction(tx: <Block as BlockT>::Extrinsic) -> TransactionValidity {
            Executive::validate_transaction(tx)
        }
    }

    impl substrate_consensus_aura_primitives::AuraApi<Block, ed25519::Public> for Runtime {
        fn slot_duration() -> u64 {
            Aura::slot_duration()
        }

        fn authorities() -> Vec<ed25519::Public> {
            Aura::authorities()
        }
    }

    impl substrate_offchain_primitives::OffchainWorkerApi<Block> for Runtime {
        fn offchain_worker(n: NumberFor<Block>) {
            Executive::offchain_worker(n)
        }
    }
}
