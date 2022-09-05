use super::*;
use crate::governance::part_council::CouncilCollective;
use frame_support::traits::EnsureOneOf;
use sp_core::u32_trait::{_1, _2};

type EnsureRootOrHalfCouncil = EnsureOneOf<
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, CouncilCollective>,
>;

parameter_types! {
	// Minimum 100 bytes/ARES deposited (1 CENT/byte)
	pub const BasicDeposit: Balance = 1000 * AMAS_UNITS;       // 258 bytes on-chain
	pub const FieldDeposit: Balance = 250 * AMAS_UNITS;        // 66 bytes on-chain
	pub const SubAccountDeposit: Balance = 200 * AMAS_UNITS;   // 53 bytes on-chain
	pub const MaxSubAccounts: u32 = 100;
	pub const MaxAdditionalFields: u32 = 100;
	pub const MaxRegistrars: u32 = 20;
}

impl pallet_identity::Config for Runtime {
	type Event = Event;
	type Currency = Balances;
	type BasicDeposit = BasicDeposit;
	type FieldDeposit = FieldDeposit;
	type SubAccountDeposit = SubAccountDeposit;
	type MaxSubAccounts = MaxSubAccounts;
	type MaxAdditionalFields = MaxAdditionalFields;
	type MaxRegistrars = MaxRegistrars;
	type Slashed = Treasury;
	type ForceOrigin = EnsureRootOrHalfCouncil;
	type RegistrarOrigin = EnsureRootOrHalfCouncil;
	type WeightInfo = pallet_identity::weights::SubstrateWeight<Runtime>;
}
