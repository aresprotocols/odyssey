use super::*;
use crate::governance::part_council::CouncilCollective;
use frame_support::traits::{EitherOfDiverse};

type EnsureRootOrHalfCouncil = EitherOfDiverse<
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionMoreThan<AccountId, CouncilCollective, 1, 2>,
>;


parameter_types! {
	// Minimum 100 bytes/ARES deposited (1 CENT/byte)
	pub const BasicDeposit: Balance = 10000 * AMAS_CENTS;       // 258 bytes on-chain
	pub const FieldDeposit: Balance = 2000 * AMAS_CENTS;        // 66 bytes on-chain
	pub const SubAccountDeposit: Balance = 2000 * AMAS_UNITS;   // 53 bytes on-chain
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
