// Copyright 2019-2021 Parity Technologies (UK) Ltd.
// This file is part of Cumulus.

// Cumulus is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Cumulus is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Cumulus.  If not, see <http://www.gnu.org/licenses/>.

#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

use ares_oracle::traits::IsAresOracleCall;
use frame_support::pallet_prelude::InvalidTransaction;
use codec::Encode;
use cumulus_primitives_core::relay_chain::Nonce;
use cumulus_pallet_parachain_system::RelayNumberStrictlyIncreases;
// A few exports that help ease life for downstream crates.
pub use frame_support::{
    construct_runtime, match_type, parameter_types,
    traits::{
        AllowAll, Currency, DenyAll, Imbalance, InstanceFilter, KeyOwnerProofSystem, LockIdentifier, OnUnbalanced,
        U128CurrencyToVote,
    },
    traits::{Everything, IsInVec, Randomness},
    weights::{
        constants::{BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight, WEIGHT_PER_SECOND},
        DispatchClass, IdentityFee, Weight,
    },
    PalletId, RuntimeDebug, StorageValue,
};
use frame_support::traits::{Contains, EitherOfDiverse};
use frame_support::weights::ConstantMultiplier;
use frame_system::limits::{BlockLength, BlockWeights};
// use frame_system::{EnsureOneOf, EnsureRoot};
use frame_system::{EnsureRoot};
pub use pallet_balances::Call as BalancesCall;
pub use pallet_timestamp::Call as TimestampCall;
use sp_std::convert::TryInto;
use sp_std::convert::TryFrom;
// XCM imports
use pallet_xcm::{EnsureXcm, IsMajorityOfBody, XcmPassthrough};
use polkadot_parachain::primitives::Sibling;
use sp_api::impl_runtime_apis;
// pub use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use ares_para_common::AuraId as AuraId;

use sp_core::{crypto::KeyTypeId, OpaqueMetadata};
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
use sp_runtime::{create_runtime_str, generic, impl_opaque_keys, traits::{AccountIdLookup, BlakeTwo256, Block as BlockT, ConvertInto}, transaction_validity::{TransactionSource, TransactionValidity}, ApplyExtrinsicResult, Percent, traits, MultiAddress};
pub use sp_runtime::{Perbill, Permill};
use sp_std::prelude::*;
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;
use xcm::latest::prelude::*;
use xcm_builder::{AccountId32Aliases, AllowTopLevelPaidExecutionFrom, AllowUnpaidExecutionFrom, EnsureXcmOrigin, FixedWeightBounds, IsConcrete, LocationInverter, NativeAsset, ParentAsSuperuser, ParentIsPreset, RelayChainAsNative, SiblingParachainAsNative, SiblingParachainConvertsVia, SignedAccountId32AsNative, SignedToAccountId32, SovereignSignedViaLocation, TakeWeightCredit, UsingComponents};
use xcm_executor::XcmExecutor;

use pallet_transaction_payment::CurrencyAdapter;

// pub use ares_oracle_provider_support::crypto::sr25519::AuthorityId as AresId;
use ares_para_common::AresId as AresId;
use pallet_balances::NegativeImbalance;
use pallet_transaction_payment::TargetedFeeAdjustment;
use sp_std::marker::PhantomData;
use parachains_common::impls::DealWithFees;
use sp_runtime::generic::{Era, SignedPayload};
use ares_para_common::constants::fee::WeightToFee;
use ares_para_common::constants::{AdjustmentVariable, MinimumMultiplier, TargetBlockFullness};
use xcm_config::{KsmLocation, XcmConfig};

pub use ares_para_common::{
    constants::currency::{deposit, CurrencyBalance, AMAS_CENTS, AMAS_MILLI_CENTS, AMAS_MILLI_UNITS, AMAS_UNITS},
    constants::time::*,
};

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

mod governance;
mod network;
mod utilities;
mod weights;
pub mod xcm_config;

// // mod part_price;
// // mod part_getprice;

mod part_challenge;
// mod part_member_extend;
mod part_offchain;
pub mod part_oracle;
mod part_oracle_finance;
mod part_manual_bridge;

// mod part_staking_extend;
pub type SessionHandlers = ();
pub type SessionKeys = network::part_session::SessionKeys;
pub type StakerStatus<AccountId> = pallet_staking::StakerStatus<AccountId>;


pub type SlowAdjustingFeeUpdate<R> =
TargetedFeeAdjustment<R, TargetBlockFullness, AdjustmentVariable, MinimumMultiplier>;

/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data.will.del like extrinsics, allowing for them to continue syncing the network through
/// upgrades to even the core data.will.del structures.
// pub mod opaque {
// 	pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;
//
// 	use super::*;
//
// 	/// Opaque block header type.
// 	pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
// 	/// Opaque block type.
// 	pub type Block = generic::Block<Header, UncheckedExtrinsic>;
// 	/// Opaque block identifier type.
// 	pub type BlockId = generic::BlockId<Block>;
// }

// pub use ares_para_common;
// use ares_para_common::{
// 	opaque::{Block, Header, BlockId}
// };


/// This runtime version.
#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: create_runtime_str!("ares-mars"),
    impl_name: create_runtime_str!("ares-mars"),
    authoring_version: 1,
    spec_version: 121,
    impl_version: 1,
    apis: RUNTIME_API_VERSIONS,
    transaction_version: 1,
    state_version: 0,
};

// 1 in 4 blocks (on average, not counting collisions) will be primary babe blocks.
pub const PRIMARY_PROBABILITY: (u64, u64) = (1, 4);

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
    NativeVersion {
        runtime_version: VERSION,
        can_author_with: Default::default(),
    }
}

/// We assume that ~10% of the block weight is consumed by `on_initalize` handlers.
/// This is used to limit the maximal weight of a single extrinsic.
const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(10);
/// We allow `Normal` extrinsics to fill up the block up to 75%, the rest can be used
/// by  Operational  extrinsics.
const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);
/// We allow for .5 seconds of compute with a 12 second average block time.
const MAXIMUM_BLOCK_WEIGHT: Weight = WEIGHT_PER_SECOND / 2;

parameter_types! {
	pub const BlockHashCount: BlockNumber = 250;
	pub const Version: RuntimeVersion = VERSION;
	pub RuntimeBlockLength: BlockLength =
		BlockLength::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
	pub RuntimeBlockWeights: BlockWeights = BlockWeights::builder()
		.base_block(BlockExecutionWeight::get())
		.for_class(DispatchClass::all(), |weights| {
			weights.base_extrinsic = ExtrinsicBaseWeight::get();
		})
		.for_class(DispatchClass::Normal, |weights| {
			weights.max_total = Some(NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT);
		})
		.for_class(DispatchClass::Operational, |weights| {
			weights.max_total = Some(MAXIMUM_BLOCK_WEIGHT);
			// Operational transactions have some extra reserved space, so that they
			// are included even if block reached `MAXIMUM_BLOCK_WEIGHT`.
			weights.reserved = Some(
				MAXIMUM_BLOCK_WEIGHT - NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT
			);
		})
		.avg_block_initialization(AVERAGE_ON_INITIALIZE_RATIO)
		.build_or_panic();
	pub const SS58Prefix: u8 = 42;
}

// Configure FRAME pallets to include in runtime.
impl frame_system::Config for Runtime {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = RuntimeBlockWeights;
	type BlockLength = RuntimeBlockLength;
	type AccountId = AccountId;
	type Call = Call;
	type Lookup = AccountIdLookup<AccountId, ()>;
	type Index = Index;
	type BlockNumber = BlockNumber;
	type Hash = Hash;
	type Hashing = BlakeTwo256;
	type Header = Header;
	type Event = Event;
	type Origin = Origin;
	type BlockHashCount = BlockHashCount;
	type DbWeight = RocksDbWeight;
	type Version = Version;
	type PalletInfo = PalletInfo;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type AccountData = pallet_balances::AccountData<Balance>;
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = cumulus_pallet_parachain_system::ParachainSetCode<Self>;
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
	pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
}

impl pallet_timestamp::Config for Runtime {
    /// A timestamp: milliseconds since the unix epoch.
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = MinimumPeriod;
    type WeightInfo = ();
}

parameter_types! {
	pub const ExistentialDeposit: u128 = 1 * AMAS_MILLI_UNITS;
	pub const TransferFee: u128 = 1 * AMAS_MILLI_UNITS;
	pub const CreationFee: u128 = 1 * AMAS_MILLI_UNITS;
	pub const TransactionByteFee: u128 = 1 * AMAS_MILLI_CENTS;
	pub const MaxLocks: u32 = 50;
	pub const MaxReserves: u32 = 50;
	pub const OperationalFeeMultiplier: u8 = 5;
}

// ---- -

// --- -

impl pallet_balances::Config for Runtime {
    /// The type for recording an account's balance.
    type Balance = Balance;
    /// The ubiquitous event type.
    type Event = Event;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
    type MaxLocks = MaxLocks;
    type MaxReserves = MaxReserves;
    type ReserveIdentifier = [u8; 8];
}

impl pallet_transaction_payment::Config for Runtime {
	type Event = Event;
	type OnChargeTransaction = CurrencyAdapter<Balances, DealWithFees<Runtime>>;
	type OperationalFeeMultiplier = OperationalFeeMultiplier;
	type WeightToFee = WeightToFee;
	type LengthToFee = ConstantMultiplier<Balance, TransactionByteFee>;
	type FeeMultiplierUpdate = SlowAdjustingFeeUpdate<Self>;
}

impl pallet_sudo::Config for Runtime {
    type Call = Call;
    type Event = Event;
}

parameter_types! {
	pub const ReservedXcmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT / 4;
	pub const ReservedDmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT / 4;
}

impl cumulus_pallet_parachain_system::Config for Runtime {
	type Event = Event;
	type OnSystemEvent = ();
	type SelfParaId = parachain_info::Pallet<Runtime>;
	type DmpMessageHandler = DmpQueue;
	type ReservedDmpWeight = ReservedDmpWeight;
	type OutboundXcmpMessageSource = XcmpQueue;
	type XcmpMessageHandler = XcmpQueue;
	type ReservedXcmpWeight = ReservedXcmpWeight;
	type CheckAssociatedRelayNumber = RelayNumberStrictlyIncreases;
}

impl parachain_info::Config for Runtime {}

parameter_types! {
	pub const RelayLocation: MultiLocation = MultiLocation::parent();
	pub const RelayNetwork: NetworkId = NetworkId::Any;
	pub RelayChainOrigin: Origin = cumulus_pallet_xcm::Origin::Relay.into();
	pub Ancestry: MultiLocation = Parachain(ParachainInfo::parachain_id().into()).into();
}

impl cumulus_pallet_xcmp_queue::Config for Runtime {
	type Event = Event;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type ChannelInfo = ParachainSystem;
	type VersionWrapper = PolkadotXcm;
	type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
	type ControllerOrigin = EitherOfDiverse<
		EnsureRoot<AccountId>,
		EnsureXcm<IsMajorityOfBody<KsmLocation, ExecutiveBody>>,
	>;
	type ControllerOriginConverter = xcm_config::XcmOriginToTransactDispatchOrigin;
	type WeightInfo = ();
}

impl cumulus_pallet_dmp_queue::Config for Runtime {
    type Event = Event;
    type XcmExecutor = XcmExecutor<XcmConfig>;
    type ExecuteOverweightOrigin = frame_system::EnsureRoot<AccountId>;
}

parameter_types! {
	pub const AssetDeposit: Balance = 1 * AMAS_MILLI_UNITS;
	pub const ApprovalDeposit: Balance = 100 * AMAS_MILLI_UNITS;
	pub const AssetAccountDeposit: Balance = deposit(1, 16);
	pub const AssetsStringLimit: u32 = 50;
	pub const MetadataDepositBase: Balance = 1 * AMAS_MILLI_UNITS;
	pub const MetadataDepositPerByte: Balance = 10 * AMAS_MILLI_UNITS;
	pub const UnitBody: BodyId = BodyId::Unit;
	pub const ExecutiveBody: BodyId = BodyId::Executive;
}

/// A majority of the Unit body from Rococo over XCM is our required administration origin.
pub type AdminOrigin = EnsureXcm<IsMajorityOfBody<RelayLocation, UnitBody>>;

construct_runtime! {
	pub enum Runtime where
		Block = Block,
		NodeBlock = generic::Block<Header, sp_runtime::OpaqueExtrinsic>,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		// System support stuff.
		System: frame_system::{Pallet, Call, Storage, Config, Event<T>}=0,
		Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent}=1,
		ParachainSystem: cumulus_pallet_parachain_system::{
			Pallet, Call, Config, Storage, Inherent, Event<T>, ValidateUnsigned,
		}=2,
		ParachainInfo: parachain_info::{Pallet, Storage, Config}=3,
		Sudo: pallet_sudo::{Pallet, Call, Storage, Config<T>, Event<T>}=4,
		// RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Pallet, Storage},

		// Monetary stuff.
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>}=5,
		TransactionPayment: pallet_transaction_payment=6,

		// Network
		// Collator support. The order of these 4 are important and shall not change.
		Authorship: pallet_authorship::{Pallet, Call, Storage}=7,
		CollatorSelection: pallet_collator_selection::{Pallet, Call, Storage, Event<T>, Config<T>}=8,
		Session: pallet_session::{Pallet, Call, Storage, Event, Config<T>}=9,
		Aura: pallet_aura::{Pallet, Storage, Config<T>}=10,
		AuraExt: cumulus_pallet_aura_ext::{Pallet, Storage, Config}=11,

		// Governance
		Democracy: pallet_democracy::{Pallet, Call, Storage, Config<T>, Event<T>}=12,
		Council: pallet_collective::<Instance1>::{Pallet, Call, Storage, Origin<T>, Event<T>, Config<T>}=13,
		TechnicalCommittee: pallet_collective::<Instance2>::{Pallet, Call, Storage, Origin<T>, Event<T>, Config<T>}=14,
		Treasury: pallet_treasury::{Pallet, Call, Storage, Config, Event<T>}=15,
		Bounties: pallet_bounties::{Pallet, Call, Storage, Event<T>}=16,
		Scheduler: pallet_scheduler::{Pallet, Call, Storage, Event<T>}=17,
		Vesting: pallet_vesting::{Pallet, Call, Storage, Event<T>, Config<T>}=18,
		Elections: pallet_elections_phragmen::{Pallet, Call, Storage, Event<T>, Config<T>}=19,

		// Ares Suit
		AresChallenge: pallet_ares_challenge::<Instance1>::{Pallet, Call, Storage, Event<T>}=20,
		OracleFinance: oracle_finance::{Pallet, Call, Storage, Event<T>}=21,
		AresOracle: ares_oracle::{Pallet, Call, Storage, Event<T>, ValidateUnsigned, Config<T>}=22,

		// XCM helpers.
		XcmpQueue: cumulus_pallet_xcmp_queue::{Pallet, Call, Storage, Event<T>}=23,
		PolkadotXcm: pallet_xcm::{Pallet, Call, Event<T>, Origin}=24 ,
		CumulusXcm: cumulus_pallet_xcm::{Pallet, Call, Event<T>, Origin}=25 ,
		DmpQueue: cumulus_pallet_dmp_queue::{Pallet, Call, Storage, Event<T>}=26,

		// Handy utilities.
		Utility: pallet_utility::{Pallet, Call, Event}=27,
		Multisig: pallet_multisig::{Pallet, Call, Storage, Event<T>}=28,
		Proxy: pallet_proxy::{Pallet, Call, Storage, Event<T>}=29,

		Assets: pallet_assets::{Pallet, Call, Storage, Event<T>}=30,
		Identity: pallet_identity::{Pallet, Call, Storage, Event<T>}=31,
		ManualBridge: manual_bridge::{Pallet, Call, Storage, Event<T>, Config<T>}=32,
		ChildBounties: pallet_child_bounties=33,
	}
}

impl pallet_assets::Config for Runtime {
    type Event = Event;
    type Balance = u64;
    type AssetId = u32;
    type Currency = Balances;
    type ForceOrigin = AdminOrigin;
    type AssetDeposit = AssetDeposit;
    type MetadataDepositBase = MetadataDepositBase;
    type MetadataDepositPerByte = MetadataDepositPerByte;
    type ApprovalDeposit = ApprovalDeposit;
    type StringLimit = AssetsStringLimit;
    type Freezer = ();
    type Extra = ();
    type WeightInfo = pallet_assets::weights::SubstrateWeight<Runtime>;
	type AssetAccountDeposit = AssetAccountDeposit; // Add on polkadot 0.9.17
}

parameter_types! {
	/// Minimum round length is 2 minutes (10 * 12 second block times)
	pub const MinBlocksPerRound: u32 = 10;
	/// Default BlocksPerRound is every hour (300 * 12 second block times)
	pub const DefaultBlocksPerRound: u32 = 300;
	/// Collator candidate exits are delayed by 2 hours (2 * 300 * block_time)
	pub const LeaveCandidatesDelay: u32 = 2;
	/// Nominator exits are delayed by 2 hours (2 * 300 * block_time)
	pub const LeaveNominatorsDelay: u32 = 2;
	/// Nomination revocations are delayed by 2 hours (2 * 300 * block_time)
	pub const RevokeNominationDelay: u32 = 2;
	/// Reward payments are delayed by 2 hours (2 * 300 * block_time)
	pub const RewardPaymentDelay: u32 = 2;
	/// Minimum 8 collators selected per round, default at genesis and minimum forever after
	pub const MinSelectedCandidates: u32 = 8;
	/// Maximum 100 nominators per collator
	pub const MaxNominatorsPerCollator: u32 = 100;
	/// Maximum 100 collators per nominator
	pub const MaxCollatorsPerNominator: u32 = 100;
	/// Default fixed percent a collator takes off the top of due rewards is 20%
	pub const DefaultCollatorCommission: Perbill = Perbill::from_percent(20);
	/// Default percent of inflation set aside for parachain bond every round
	pub const DefaultParachainBondReservePercent: Percent = Percent::from_percent(30);
	/// Minimum stake required to become a collator is 1_000
	pub const MinCollatorStk: u128 = 1000 * AMAS_UNITS;
	/// Minimum stake required to be reserved to be a candidate is 1_000
	pub const MinCollatorCandidateStk: u128 = 1000 * AMAS_UNITS;
	/// Minimum stake required to be reserved to be a nominator is 5
	pub const MinNominatorStk: u128 = 5 * AMAS_UNITS;
}

// pub type Signature = sp_runtime::MultiSignature;
pub type Signature = ares_para_common::Signature;
// 	<<Signature as sp_runtime::traits::Verify>::Signer as sp_runtime::traits::IdentifyAccount>::AccountId;
pub type AccountId = ares_para_common::AccountId;
// pub type Balance = CurrencyBalance;
pub type Balance = ares_para_common::Balance;
// pub type Index = u32;
pub type Index = ares_para_common::Index;
// pub type Hash = sp_core::H256;
pub type Hash = ares_para_common::Hash;
// pub type BlockNumber = u32;
pub type BlockNumber = ares_para_common::BlockNumber;
// pub type Address = sp_runtime::MultiAddress<AccountId, ()>;
pub type Address = ares_para_common::Address;
// pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
pub type Header = ares_para_common::Header;
// pub type Block = generic::Block<Header, UncheckedExtrinsic>;
pub type Block = ares_para_common::Block<Call, SignedExtra>;
// pub type SignedBlock = generic::SignedBlock<Block>;
pub type SignedBlock = ares_para_common::SignedBlock<Call, SignedExtra>;
// pub type BlockId = generic::BlockId<Block>;
pub type BlockId = ares_para_common::BlockId<Call, SignedExtra>;
/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
    frame_system::CheckSpecVersion<Runtime>,
    frame_system::CheckGenesis<Runtime>,
    frame_system::CheckEra<Runtime>,
    frame_system::CheckNonce<Runtime>,
    frame_system::CheckWeight<Runtime>,
    pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
);
// /// Unchecked extrinsic type as expected by this runtime.
// pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, Call, Signature, SignedExtra>;
// /// Extrinsic type that has already been checked.
// pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, Call, SignedExtra>;
// /// Executive: handles dispatch to the various modules.
// pub type Executive =
// 	frame_executive::Executive<Runtime, Block, frame_system::ChainContext<Runtime>, Runtime, AllPallets>;

pub type UncheckedExtrinsic = ares_para_common::UncheckedExtrinsic<Call, SignedExtra>;
pub type CheckedExtrinsic = ares_para_common::CheckedExtrinsic<Call, SignedExtra>;
pub type Executive = ares_para_common::Executive<Runtime, AllPallets, Call, SignedExtra>;

impl IsAresOracleCall<Runtime, Call> for Call {
	fn try_get_pallet_call(in_call: &Call) -> Option<&ares_oracle::pallet::Call<Runtime>> {
		if let Self::AresOracle(
			x_call
		) = in_call {
			return Some(x_call);
		}
		None
	}
}

impl_runtime_apis! {
	impl sp_api::Core<Block> for Runtime {
		fn version() -> RuntimeVersion {
			VERSION
		}

		fn execute_block(block: Block) {
			Executive::execute_block(block);
		}

		fn initialize_block(header: &<Block as BlockT>::Header) {
			Executive::initialize_block(header)
		}
	}

	impl sp_api::Metadata<Block> for Runtime {
		fn metadata() -> OpaqueMetadata {
			OpaqueMetadata::new(Runtime::metadata().into())
		}
	}

	impl sp_block_builder::BlockBuilder<Block> for Runtime {
		// fn apply_extrinsic(
		// 	extrinsic: <Block as BlockT>::Extrinsic,
		// ) -> ApplyExtrinsicResult {
		// 	Executive::apply_extrinsic(extrinsic)
		// }

		fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
			let filter_result = ares_oracle::offchain_filter::AresOracleFilter::<Runtime, Address, Call, Signature, SignedExtra>::is_author_call(&extrinsic, false);
			// log::info!("Oracle filter_result = {:?} on apply_extrinsic", &filter_result);
			if filter_result {
				return Executive::apply_extrinsic(extrinsic);
			}
			ApplyExtrinsicResult::Err(frame_support::pallet_prelude::TransactionValidityError::Invalid(InvalidTransaction::Call))
		}

		fn finalize_block() -> <Block as BlockT>::Header {
			Executive::finalize_block()
		}

		fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
			data.create_extrinsics()
		}

		fn check_inherents(block: Block, data: sp_inherents::InherentData) -> sp_inherents::CheckInherentsResult {
			data.check_extrinsics(&block)
		}
	}

	impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
		fn validate_transaction(
			source: TransactionSource,
			tx: <Block as BlockT>::Extrinsic,
			block_hash: <Block as BlockT>::Hash,
		) -> TransactionValidity {
			let filter_result = ares_oracle::offchain_filter::AresOracleFilter::<Runtime, Address, Call, Signature, SignedExtra>::is_author_call(&tx, false);
			if filter_result {
				return Executive::validate_transaction(source, tx, block_hash)
			}
			Executive::validate_transaction(source, tx, block_hash)
		}
	}

	impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
		fn offchain_worker(header: &<Block as BlockT>::Header) {
			Executive::offchain_worker(header)
		}
	}

	impl sp_session::SessionKeys<Block> for Runtime {
		fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
			network::part_session::SessionKeys::generate(seed)
		}

		fn decode_session_keys(
			encoded: Vec<u8>,
		) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
			network::part_session::SessionKeys::decode_into_raw_public_keys(&encoded)
		}
	}

	impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Index> for Runtime {
		fn account_nonce(account: AccountId) -> Index {
			System::account_nonce(account)
		}
	}

	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance> for Runtime {
		fn query_info(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_info(uxt, len)
		}
		fn query_fee_details(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment::FeeDetails<Balance> {
			TransactionPayment::query_fee_details(uxt, len)
		}
	}

	impl sp_consensus_aura::AuraApi<Block, AuraId> for Runtime {
		fn slot_duration() -> sp_consensus_aura::SlotDuration {
			sp_consensus_aura::SlotDuration::from_millis(Aura::slot_duration())
		}

		fn authorities() -> Vec<AuraId> {
			Aura::authorities().into_inner()
		}
	}

	// impl cumulus_primitives_core::CollectCollationInfo<Block> for Runtime {
	// 	fn collect_collation_info() -> cumulus_primitives_core::CollationInfo {
	// 		ParachainSystem::collect_collation_info()
	// 	}
	// }

	impl cumulus_primitives_core::CollectCollationInfo<Block> for Runtime {
		fn collect_collation_info(header: &<Block as BlockT>::Header) -> cumulus_primitives_core::CollationInfo {
			ParachainSystem::collect_collation_info(header)
		}
	}
}

struct CheckInherents;

impl cumulus_pallet_parachain_system::CheckInherents<Block> for CheckInherents {
    fn check_inherents(
        block: &Block,
        relay_state_proof: &cumulus_pallet_parachain_system::RelayChainStateProof,
    ) -> sp_inherents::CheckInherentsResult {
        let relay_chain_slot = relay_state_proof
            .read_slot()
            .expect("Could not read the relay chain slot from the proof");

        let inherent_data = cumulus_primitives_timestamp::InherentDataProvider::from_relay_chain_slot_and_duration(
            relay_chain_slot,
            sp_std::time::Duration::from_secs(6),
        )
            .create_inherent_data()
            .expect("Could not create the timestamp inherent data.will.del");

        inherent_data.check_extrinsics(&block)
    }
}

cumulus_pallet_parachain_system::register_validate_block! {
	Runtime = Runtime,
	BlockExecutor = cumulus_pallet_aura_ext::BlockExecutor::<Runtime, Executive>,
	CheckInherents = CheckInherents,
}
