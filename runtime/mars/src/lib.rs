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
use frame_support::traits::EnsureOneOf;
use frame_system::limits::{BlockLength, BlockWeights};
// use frame_system::{EnsureOneOf, EnsureRoot};
use frame_system::{EnsureRoot};
pub use pallet_balances::Call as BalancesCall;
pub use pallet_timestamp::Call as TimestampCall;
// XCM imports
use pallet_xcm::{EnsureXcm, IsMajorityOfBody, XcmPassthrough};
use polkadot_parachain::primitives::Sibling;
use sp_api::impl_runtime_apis;
// pub use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use ares_para_common::AuraId as AuraId;

use sp_core::{crypto::KeyTypeId, OpaqueMetadata};
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
use sp_runtime::{
    create_runtime_str, generic, impl_opaque_keys,
    traits::{AccountIdLookup, BlakeTwo256, Block as BlockT, ConvertInto},
    transaction_validity::{TransactionSource, TransactionValidity},
    ApplyExtrinsicResult, Percent,
};
pub use sp_runtime::{Perbill, Permill};
use sp_std::prelude::*;
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;
use xcm::latest::prelude::*;
use xcm_builder::{AccountId32Aliases, AllowTopLevelPaidExecutionFrom, AllowUnpaidExecutionFrom, CurrencyAdapter, EnsureXcmOrigin, FixedWeightBounds, IsConcrete, LocationInverter, NativeAsset, ParentAsSuperuser, ParentIsPreset, RelayChainAsNative, SiblingParachainAsNative, SiblingParachainConvertsVia, SignedAccountId32AsNative, SignedToAccountId32, SovereignSignedViaLocation, TakeWeightCredit, UsingComponents};
use xcm_executor::XcmExecutor;

// pub use constants::{
// 	currency::{deposit, CurrencyBalance, AMAS_CENTS, AMAS_MILLI_CENTS, AMAS_MILLI_UNITS, AMAS_UNITS},
// 	time::*,
// };

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
mod part_member_extend;
mod part_offchain;
pub mod part_oracle;
mod part_oracle_finance;

// mod part_staking_extend;
pub type SessionHandlers = ();
pub type SessionKeys = network::part_session::SessionKeys;
pub type StakerStatus<AccountId> = pallet_staking::StakerStatus<AccountId>;

// pub use ares_oracle_provider_support::crypto::sr25519::AuthorityId as AresId;
use ares_para_common::AresId as AresId;
use pallet_balances::NegativeImbalance;
use sp_std::marker::PhantomData;
use parachains_common::impls::DealWithFees;
use xcm_config::{KsmLocation, XcmConfig};

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
    spec_version: 118,
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

impl frame_system::Config for Runtime {
    /// The identifier used to distinguish between accounts.
    type AccountId = AccountId;
    /// The aggregated dispatch type that is available for extrinsics.
    type Call = Call;
    /// The lookup mechanism to get account ID from whatever is passed in dispatchers.
    type Lookup = AccountIdLookup<AccountId, ()>;
    /// The index type for storing how many extrinsics an account has signed.
    type Index = Index;
    /// The index type for blocks.
    type BlockNumber = BlockNumber;
    /// The type for hashing blocks and tries.
    type Hash = Hash;
    /// The hashing algorithm used.
    type Hashing = BlakeTwo256;
    /// The header type.
    type Header = generic::Header<BlockNumber, BlakeTwo256>;
    /// The ubiquitous event type.
    type Event = Event;
    /// The ubiquitous origin type.
    type Origin = Origin;
    /// Maximum number of block number to block hash mappings to keep (oldest pruned first).
    type BlockHashCount = BlockHashCount;
    /// Runtime version.
    type Version = Version;
    /// Converts a module to an index of this module in the runtime.
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<Balance>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type DbWeight = ();
    type BaseCallFilter = frame_support::traits::Everything;
    type SystemWeightInfo = ();
    type BlockWeights = RuntimeBlockWeights;
    type BlockLength = RuntimeBlockLength;
    type SS58Prefix = SS58Prefix;
    type OnSetCode = cumulus_pallet_parachain_system::ParachainSetCode<Self>;
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}


// pub struct DealWithFees<R>(PhantomData<R>);
// impl<R> OnUnbalanced<NegativeImbalance<R>> for DealWithFees<R>
// 	where
// 		R: pallet_balances::Config + pallet_collator_selection::Config,
// 		AccountIdOf<R>:
// 		From<polkadot_primitives::v1::AccountId> + Into<polkadot_primitives::v1::AccountId>,
// 		<R as frame_system::Config>::Event: From<pallet_balances::Event<R>>,
// {
// 	fn on_unbalanceds<B>(mut fees_then_tips: impl Iterator<Item = NegativeImbalance<R>>) {
// 		if let Some(mut fees) = fees_then_tips.next() {
// 			if let Some(tips) = fees_then_tips.next() {
// 				tips.merge_into(&mut fees);
// 			}
// 			<ToStakingPot<R> as OnUnbalanced<_>>::on_unbalanced(fees);
// 		}
// 	}
// }

// impl pallet_transaction_payment::Config for Runtime {
// 	type OnChargeTransaction =
// 	pallet_transaction_payment::CurrencyAdapter<Balances, ()>;
// 	type TransactionByteFee = TransactionByteFee;
// 	type WeightToFee = ();
// 	type FeeMultiplierUpdate = SlowAdjustingFeeUpdate<Self>;
// 	type OperationalFeeMultiplier = OperationalFeeMultiplier;
// }

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

// impl pallet_randomness_collective_flip::Config for Runtime {}
impl pallet_transaction_payment::Config for Runtime {
    type OnChargeTransaction = pallet_transaction_payment::CurrencyAdapter<Balances, DealWithFees<Runtime>>;
    // DealWithFees<Runtime>
    type TransactionByteFee = TransactionByteFee;
    type WeightToFee = IdentityFee<Balance>;
    type FeeMultiplierUpdate = ();
    type OperationalFeeMultiplier = OperationalFeeMultiplier;
}

impl pallet_sudo::Config for Runtime {
    type Call = Call;
    type Event = Event;
}

parameter_types! {
	pub const ReservedXcmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT / 4;
	pub const ReservedDmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT / 4;
}

// impl cumulus_pallet_parachain_system::Config for Runtime {
//     type Event = Event;
//     type OnValidationData = ();
//     type SelfParaId = parachain_info::Pallet<Runtime>;
//     type OutboundXcmpMessageSource = XcmpQueue;
//     type DmpMessageHandler = DmpQueue;
//     type ReservedDmpWeight = ReservedDmpWeight;
//     type XcmpMessageHandler = XcmpQueue;
//     type ReservedXcmpWeight = ReservedXcmpWeight;
// }

impl cumulus_pallet_parachain_system::Config for Runtime {
	type Event = Event;
	type OnSystemEvent = ();
	type SelfParaId = parachain_info::Pallet<Runtime>;
	type DmpMessageHandler = DmpQueue;
	type ReservedDmpWeight = ReservedDmpWeight;
	type OutboundXcmpMessageSource = XcmpQueue;
	type XcmpMessageHandler = XcmpQueue;
	type ReservedXcmpWeight = ReservedXcmpWeight;
}
impl parachain_info::Config for Runtime {}

parameter_types! {
	pub const RelayLocation: MultiLocation = MultiLocation::parent();
	pub const RelayNetwork: NetworkId = NetworkId::Any;
	pub RelayChainOrigin: Origin = cumulus_pallet_xcm::Origin::Relay.into();
	pub Ancestry: MultiLocation = Parachain(ParachainInfo::parachain_id().into()).into();
}

// /// Type for specifying how a `MultiLocation` can be converted into an `AccountId`. This is used
// /// when determining ownership of accounts for asset transacting and when attempting to use XCM
// /// `Transact` in order to determine the dispatch Origin.
// pub type LocationToAccountId = (
//     // The parent (Relay-chain) origin converts to the default `AccountId`.
//     ParentIsDefault<AccountId>,
//     // Sibling parachain origins convert to AccountId via the `ParaId::into`.
//     SiblingParachainConvertsVia<Sibling, AccountId>,
//     // Straight up local `AccountId32` origins just alias directly to `AccountId`.
//     AccountId32Aliases<RelayNetwork, AccountId>,
// );
//
// /// Means for transacting assets on this chain.
// pub type LocalAssetTransactor = CurrencyAdapter<
//     // Use this currency:
//     Balances,
//     // Use this currency when it is a fungible asset matching the given location or name:
//     IsConcrete<RelayLocation>,
//     // Do a simple punn to convert an AccountId32 MultiLocation into a native chain account ID:
//     LocationToAccountId,
//     // Our chain's account ID type (we can't get away without mentioning it explicitly):
//     AccountId,
//     // We don't track any teleports.
//     (),
// >;
//
// /// This is the type we use to convert an (incoming) XCM origin into a local `Origin` instance,
// /// ready for dispatching a transaction with Xcm's `Transact`. There is an `OriginKind` which can
// /// biases the kind of local `Origin` it will become.
// pub type XcmOriginToTransactDispatchOrigin = (
//     // Sovereign account converter; this attempts to derive an `AccountId` from the origin location
//     // using `LocationToAccountId` and then turn that into the usual `Signed` origin. Useful for
//     // foreign chains who want to have a local sovereign account on this chain which they control.
//     SovereignSignedViaLocation<LocationToAccountId, Origin>,
//     // Native converter for Relay-chain (Parent) location; will converts to a `Relay` origin when
//     // recognised.
//     RelayChainAsNative<RelayChainOrigin, Origin>,
//     // Native converter for sibling Parachains; will convert to a `SiblingPara` origin when
//     // recognised.
//     SiblingParachainAsNative<cumulus_pallet_xcm::Origin, Origin>,
//     // Superuser converter for the Relay-chain (Parent) location. This will allow it to issue a
//     // transaction from the Root origin.
//     ParentAsSuperuser<Origin>,
//     // Native signed account converter; this just converts an `AccountId32` origin into a normal
//     // `Origin::Signed` origin of the same 32-byte value.
//     SignedAccountId32AsNative<RelayNetwork, Origin>,
//     // Xcm origins can be represented natively under the Xcm pallet's Xcm origin.
//     XcmPassthrough<Origin>,
// );
//
// parameter_types! {
// 	// One XCM operation is 1_000_000 weight - almost certainly a conservative estimate.
// 	pub UnitWeightCost: Weight = 1_000_000;
// 	pub const MaxInstructions: u32 = 100;
// 	// One ROC buys 1 second of weight.
// 	pub const WeightPrice: (MultiLocation, u128) = (MultiLocation::parent(), AMAS_UNITS);
// }
//
// match_type! {
// 	pub type ParentOrParentsUnitPlurality: impl Contains<MultiLocation> = {
// 		MultiLocation { parents: 1, interior: Here } |
// 		MultiLocation { parents: 1, interior: X1(Plurality { id: BodyId::Unit, .. }) }
// 	};
// }
//
// pub type Barrier = (
//     TakeWeightCredit,
//     AllowTopLevelPaidExecutionFrom<Everything>,
//     AllowUnpaidExecutionFrom<ParentOrParentsUnitPlurality>,
//     AllowUnpaidExecutionFrom<Everything>,
//     /*AllowUnpaidExecutionFrom<SpecParachain>,
//      * ^^^ Parent & its unit plurality gets free execution */
// );
//
// pub struct XcmConfig;
//
// impl xcm_executor::Config for XcmConfig {
//     type Call = Call;
//     type XcmSender = XcmRouter;
//     // How to withdraw and deposit an asset.
//     type AssetTransactor = LocalAssetTransactor;
//     type OriginConverter = XcmOriginToTransactDispatchOrigin;
//     type IsReserve = NativeAsset;
//     type IsTeleporter = NativeAsset;
//     // <- should be enough to allow teleportation of ROC
//     type LocationInverter = LocationInverter<Ancestry>;
//     type Barrier = Barrier;
//     type Weigher = FixedWeightBounds<UnitWeightCost, Call, MaxInstructions>;
//     type Trader = UsingComponents<IdentityFee<Balance>, RelayLocation, AccountId, Balances, ()>;
//     type ResponseHandler = ();
//     type AssetTrap = PolkadotXcm;
//     type AssetClaims = PolkadotXcm;
//     type SubscriptionService = PolkadotXcm;
// }
//
// /// No local origins on this chain are allowed to dispatch XCM sends/executions.
// pub type LocalOriginToLocation = SignedToAccountId32<Origin, AccountId, RelayNetwork>;
//
// /// The means for routing XCM messages which are not for local execution into the right message
// /// queues.
// pub type XcmRouter = (
//     // Two routers - use UMP to communicate with the relay chain:
//     cumulus_primitives_utility::ParentAsUmp<ParachainSystem, ()>,
//     // ..and XCMP to communicate with the sibling chains.
//     XcmpQueue,
// );
//
// impl pallet_xcm::Config for Runtime {
//     type Event = Event;
//     type SendXcmOrigin = EnsureXcmOrigin<Origin, LocalOriginToLocation>;
//     type XcmRouter = XcmRouter;
//     type ExecuteXcmOrigin = EnsureXcmOrigin<Origin, LocalOriginToLocation>;
//     type XcmExecuteFilter = Everything;
//     type XcmExecutor = XcmExecutor<XcmConfig>;
//     type XcmTeleportFilter = Everything;
//     type XcmReserveTransferFilter = Everything;
//     type Weigher = FixedWeightBounds<UnitWeightCost, Call, MaxInstructions>;
//     type LocationInverter = LocationInverter<Ancestry>;
//     type Origin = Origin;
//     type Call = Call;
//     const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;
//     type AdvertisedXcmVersion = pallet_xcm::CurrentXcmVersion;
// }
//
// impl cumulus_pallet_xcm::Config for Runtime {
//     type Event = Event;
//     type XcmExecutor = XcmExecutor<XcmConfig>;
// }
//
// impl cumulus_pallet_xcmp_queue::Config for Runtime {
//     type Event = Event;
//     type XcmExecutor = XcmExecutor<XcmConfig>;
//     type ChannelInfo = ParachainSystem;
//     type VersionWrapper = ();
// }

impl cumulus_pallet_xcmp_queue::Config for Runtime {
	type Event = Event;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type ChannelInfo = ParachainSystem;
	type VersionWrapper = PolkadotXcm;
	type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
	type ControllerOrigin = EnsureOneOf<EnsureRoot<AccountId>, EnsureXcm<IsMajorityOfBody<KsmLocation, ExecutiveBody>>>;
	type ControllerOriginConverter = xcm_config::XcmOriginToTransactDispatchOrigin;
}

impl cumulus_pallet_dmp_queue::Config for Runtime {
    type Event = Event;
    type XcmExecutor = XcmExecutor<XcmConfig>;
    type ExecuteOverweightOrigin = frame_system::EnsureRoot<AccountId>;
}

// impl cumulus_ping::Config for Runtime {
// 	type Event = Event;
// 	type Origin = Origin;
// 	type Call = Call;
// 	type XcmSender = XcmRouter;
// }

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
		System: frame_system::{Pallet, Call, Storage, Config, Event<T>},
		Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
		ParachainSystem: cumulus_pallet_parachain_system::{
			Pallet, Call, Config, Storage, Inherent, Event<T>, ValidateUnsigned,
		},
		ParachainInfo: parachain_info::{Pallet, Storage, Config},
		Sudo: pallet_sudo::{Pallet, Call, Storage, Config<T>, Event<T>},
		// RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Pallet, Storage},

		// Monetary stuff.
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		TransactionPayment: pallet_transaction_payment::{Pallet, Storage},
		// ParachainStaking: parachain_staking::{Pallet, Call, Storage, Event<T>, Config<T>} = 31,
		// Assets: pallet_assets::{Pallet, Call, Storage, Event<T>} = 32,

		// Network
		// Collator support. The order of these 4 are important and shall not change.
		Authorship: pallet_authorship::{Pallet, Call, Storage},
		CollatorSelection: pallet_collator_selection::{Pallet, Call, Storage, Event<T>, Config<T>},
		Session: pallet_session::{Pallet, Call, Storage, Event, Config<T>},
		Aura: pallet_aura::{Pallet, Storage, Config<T>},
		AuraExt: cumulus_pallet_aura_ext::{Pallet, Storage, Config},
		// Staking: pallet_staking::{Pallet, Call, Config<T>, Storage, Event<T>},
		// Historical: pallet_session::historical::{Pallet},
		// ElectionProviderMultiPhase: pallet_election_provider_multi_phase::{Pallet, Call, Storage, Event<T>, ValidateUnsigned},


		// Governance
		Democracy: pallet_democracy::{Pallet, Call, Storage, Config<T>, Event<T>},
		Council: pallet_ares_collective::<Instance1>::{Pallet, Call, Storage, Origin<T>, Event<T>, Config<T>},
		TechnicalCommittee: pallet_collective::<Instance2>::{Pallet, Call, Storage, Origin<T>, Event<T>, Config<T>},
		Treasury: pallet_treasury::{Pallet, Call, Storage, Config, Event<T>},
		Bounties: pallet_bounties::{Pallet, Call, Storage, Event<T>},
		Scheduler: pallet_scheduler::{Pallet, Call, Storage, Event<T>},
		Vesting: pallet_vesting::{Pallet, Call, Storage, Event<T>, Config<T>},
		Elections: pallet_elections_phragmen::{Pallet, Call, Storage, Event<T>, Config<T>},
		// Staking: pallet_staking::{Pallet, Call, Config<T>, Storage, Event<T>},

		// Ares Suit
		AresChallenge: pallet_ares_challenge::{Pallet, Call, Storage, Event<T>},
		// MemberExtend: member_extend::{Pallet},
		OracleFinance: oracle_finance::{Pallet, Call, Storage, Event<T>},
		AresOracle: ares_oracle::{Pallet, Call, Storage, Event<T>, ValidateUnsigned, Config<T>},
		// StakingExtend: staking_extend::{Pallet},

		// XCM helpers.
		XcmpQueue: cumulus_pallet_xcmp_queue::{Pallet, Call, Storage, Event<T>},
		PolkadotXcm: pallet_xcm::{Pallet, Call, Event<T>, Origin} ,
		CumulusXcm: cumulus_pallet_xcm::{Pallet, Call, Event<T>, Origin} ,
		DmpQueue: cumulus_pallet_dmp_queue::{Pallet, Call, Storage, Event<T>},

		// Handy utilities.
		Utility: pallet_utility::{Pallet, Call, Event},
		Multisig: pallet_multisig::{Pallet, Call, Storage, Event<T>},
		Proxy: pallet_proxy::{Pallet, Call, Storage, Event<T>},

		// Spambot: cumulus_ping::{Pallet, Call, Storage, Event<T>} = 99,
		// Price: pallet_price::{Pallet, Call, Storage, Event<T>} = 100,
		// GetPrice: pallet_getprice::{Pallet, Call, Storage, Event<T>} = 101,

		Assets: pallet_assets::{Pallet, Call, Storage, Event<T>},
		// Uniques: pallet_uniques::{Pallet, Call, Storage, Event<T>},

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
		fn apply_extrinsic(
			extrinsic: <Block as BlockT>::Extrinsic,
		) -> ApplyExtrinsicResult {
			Executive::apply_extrinsic(extrinsic)
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
			Executive::validate_transaction(source, tx, block_hash)
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

	impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
		fn offchain_worker(header: &<Block as BlockT>::Header) {
			Executive::offchain_worker(header)
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
