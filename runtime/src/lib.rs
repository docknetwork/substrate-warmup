//! The Substrate Node Template runtime. This can be compiled with `#[no_std]`, ready for Wasm.

// ideally this module would alway be no_std, but macros like construct_runtime make that difficult
#![cfg_attr(not(feature = "std"), no_std)]

// `construct_runtime!` requires a large stack
#![recursion_limit = "256"]

// exposes pub WASM_BINARY and pub WASM_BINARY_BLOATY
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

    type Header = generic::Header<<Runtime as srml_system::Trait>::BlockNumber, BlakeTwo256>;
    pub type Block = generic::Block<Header, UncheckedExtrinsic>;
}

pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: create_runtime_str!("node-template"),
    impl_name: create_runtime_str!("node-template"),
    authoring_version: 3,
    spec_version: 4,
    impl_version: 4,
    apis: RUNTIME_API_VERSIONS,
};

parameter_types! {
    pub const ChainStateCacheSize: <Runtime as srml_system::Trait>::BlockNumber = 250;
}

impl srml_system::Trait for Runtime {
    type AccountId = ed25519::Public;
    type Lookup = Indices;
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type Header = generic::Header<Self::BlockNumber, BlakeTwo256>;
    type Event = Event;
    type WeightMultiplierUpdate = ();
    type Origin = Origin;
    type BlockHashCount = ChainStateCacheSize;
}

impl srml_aura::Trait for Runtime {
    type HandleReport = ();
    type AuthorityId = ed25519::Public;
}

impl srml_indices::Trait for Runtime {
    type AccountIndex = u32;
    type ResolveHint = srml_indices::SimpleResolveHint<Self::AccountId, Self::AccountIndex>;
    type IsDeadAccount = Balances;
    type Event = Event;
}

parameter_types! {
    pub const MinimumPeriod: u64 = 5;
}

impl srml_timestamp::Trait for Runtime {
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
    type Balance = u128;
    type OnFreeBalanceZero = ();
    type OnNewAccount = Indices;
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
    type Event = Event;
    type Proposal = Call;
}

impl template::Trait for Runtime {
    type Event = Event;
}

type UncheckedExtrinsic = generic::UncheckedMortalCompactExtrinsic<
    <Indices as StaticLookup>::Source,
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

// just a bunch of proxying
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
