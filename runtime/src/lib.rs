//! The Substrate Node Template runtime. This can be compiled with `#[no_std]`, ready for Wasm.

// ideally this module would alway be no_std, but macros like construct_runtime make that difficult
#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` requires a large stack
#![recursion_limit = "256"]

// exposes pub WASM_BINARY and pub WASM_BINARY_BLOATY
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

use sr_primitives::traits::{BlakeTwo256, Block as BlockT, ConvertInto, NumberFor, StaticLookup};
use sr_primitives::transaction_validity::TransactionValidity;
use sr_primitives::Perbill;
use sr_primitives::{create_runtime_str, generic, ApplyResult};
use sr_version::RuntimeVersion;
use srml_support::{construct_runtime, parameter_types};
use substrate_application_crypto::ed25519::AppPublic;
use substrate_client::{
    block_builder::api::{self as block_builder_api, CheckInherentsResult, InherentData},
    impl_runtime_apis, runtime_api,
};
use substrate_primitives::{ed25519, OpaqueMetadata, H256};

/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core datastructures.
pub mod opaque {
    use super::Runtime;
    use sr_primitives::generic;
    pub type Header = generic::Header<
        <Runtime as srml_system::Trait>::BlockNumber,
        <Runtime as srml_system::Trait>::Hashing,
    >;
    pub type Block = generic::Block<Header, sr_primitives::OpaqueExtrinsic>;
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
    pub const MaximumBlockWeight: u32 = 1_000_000;
    pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
    pub const MaximumBlockLength: u32 = 5 * 1024 * 1024;
    pub const Version: RuntimeVersion = VERSION;
}

impl srml_system::Trait for Runtime {
    type AccountId = ed25519::Public;
    /// The aggregated dispatch type that is available for extrinsics.
    type Call = Call;
    type Lookup = Indices;
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type Header = generic::Header<Self::BlockNumber, Self::Hashing>;
    type Event = Event;
    type WeightMultiplierUpdate = ();
    type Origin = Origin;
    type BlockHashCount = ChainStateCacheSize;
    /// Maximum weight of each block. With a default weight system of 1byte == 1weight, 4mb is ok.
    type MaximumBlockWeight = MaximumBlockWeight;
    /// Maximum size of all encoded transactions (in bytes) that are allowed in one block.
    type MaximumBlockLength = MaximumBlockLength;
    type AvailableBlockRatio = AvailableBlockRatio;
    type Version = Version;
}

impl srml_aura::Trait for Runtime {
    type AuthorityId = AppPublic;
}

impl srml_indices::Trait for Runtime {
    type AccountIndex = u32;
    type ResolveHint = srml_indices::SimpleResolveHint<Self::AccountId, Self::AccountIndex>;
    type IsDeadAccount = ();
    type Event = Event;
}

impl srml_timestamp::Trait for Runtime {
    type Moment = u64;
    type OnTimestampSet = Aura;
    type MinimumPeriod = MinimumPeriod;
}

parameter_types! {
    pub const MinimumPeriod: u64 = 5;
}

pub type Address = <Indices as StaticLookup>::Source;
pub type Header = <Runtime as system::Trait>::Header;
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
pub type SignedBlock = generic::SignedBlock<Block>;
pub type BlockId = generic::BlockId<Block>;
pub type SignedExtra = (
    srml_system::CheckVersion<Runtime>,
    srml_system::CheckGenesis<Runtime>,
    srml_system::CheckEra<Runtime>,
    srml_system::CheckNonce<Runtime>,
    srml_system::CheckWeight<Runtime>,
);
pub type UncheckedExtrinsic =
    generic::UncheckedExtrinsic<Address, Call, ed25519::Signature, SignedExtra>;
pub type CheckedExtrinsic = generic::CheckedExtrinsic<ed25519::Public, Call, SignedExtra>;
pub type Executive =
    srml_executive::Executive<Runtime, Block, system::ChainContext<Runtime>, Runtime, AllModules>;

parameter_types! {
    pub const PdockExistentialDeposit: u128 = 1;
    pub const PdockTransferFee: u128 = 1;
    pub const PdockCreationFee: u128 = 1;
    pub const PdockTransactionBaseFee: u128 = 1;
    pub const PdockTransactionByteFee: u128 = 1;
    pub const PstableExistentialDeposit: u128 = 1;
    pub const PstableTransferFee: u128 = 1;
    pub const PstableCreationFee: u128 = 1;
    pub const PstableTransactionBaseFee: u128 = 1;
    pub const PstableTransactionByteFee: u128 = 1;
}

/// Configure PDOCK token
impl srml_balances::Trait<srml_balances::Instance0> for Runtime {
    type Balance = u128;
    type OnFreeBalanceZero = ();
    type OnNewAccount = ();
    type Event = Event;
    type TransactionPayment = ();
    type DustRemoval = ();
    type TransferPayment = ();
    type ExistentialDeposit = PdockExistentialDeposit;
    type TransferFee = PdockTransferFee;
    type CreationFee = PdockCreationFee;
    type TransactionBaseFee = PdockTransactionBaseFee;
    type TransactionByteFee = PdockTransactionByteFee;
    type WeightToFee = ConvertInto;
}

/// Configure PSTABLE token
impl srml_balances::Trait<srml_balances::Instance1> for Runtime {
    type Balance = u128;
    type OnFreeBalanceZero = ();
    type OnNewAccount = ();
    type Event = Event;
    type TransactionPayment = ();
    type DustRemoval = ();
    type TransferPayment = ();
    type ExistentialDeposit = PstableExistentialDeposit;
    type TransferFee = PstableTransferFee;
    type CreationFee = PstableCreationFee;
    type TransactionBaseFee = PstableTransactionBaseFee;
    type TransactionByteFee = PstableTransactionByteFee;
    type WeightToFee = ConvertInto;
}

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
		Indices: srml_indices::{Module, Call, Storage, Event<T>, Config<T>},
        Pdock: srml_balances::<Instance0>::{Module, Call, Storage, Event<T>, Config<T>},
        Pstable: srml_balances::<Instance1>::{Module, Call, Storage, Event<T>, Config<T>},
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

    impl substrate_consensus_aura_primitives::AuraApi<Block, AppPublic> for Runtime {
        fn slot_duration() -> u64 {
            Aura::slot_duration()
        }

        fn authorities() -> Vec<AppPublic> {
            Aura::authorities()
        }
    }

    impl substrate_offchain_primitives::OffchainWorkerApi<Block> for Runtime {
        fn offchain_worker(n: NumberFor<Block>) {
            Executive::offchain_worker(n)
        }
    }
}
