use super::*;
use pallet_bounties;
use part_treasury::{DataDepositPerByte, MaximumReasonLength};

parameter_types! {
	pub const BountyCuratorDeposit: Permill = Permill::from_percent(50);
	pub const BountyValueMinimum: Balance = 10 * AMAS_UNITS;
	pub const BountyDepositBase: Balance = 1 * AMAS_UNITS;
	pub const CuratorDepositMultiplier: Permill = Permill::from_percent(50);
	pub const CuratorDepositMin: Balance = 1 * AMAS_UNITS;
	pub const CuratorDepositMax: Balance = 100 * AMAS_UNITS;
	pub const BountyDepositPayoutDelay: BlockNumber = 5 * DAYS;
	pub const BountyUpdatePeriod: BlockNumber = 15 * DAYS;
}

impl pallet_bounties::Config for Runtime {
	type Event = Event;
	type BountyDepositBase = BountyDepositBase;
	type BountyDepositPayoutDelay = BountyDepositPayoutDelay;
	type BountyUpdatePeriod = BountyUpdatePeriod;
	type CuratorDepositMultiplier = CuratorDepositMultiplier;
	type CuratorDepositMin = CuratorDepositMin;
	type CuratorDepositMax = CuratorDepositMax;
	type BountyValueMinimum = BountyValueMinimum;
	type DataDepositPerByte = DataDepositPerByte;
	type MaximumReasonLength = MaximumReasonLength;
	type WeightInfo = pallet_bounties::weights::SubstrateWeight<Runtime>;
	type ChildBountyManager = ChildBounties;
}