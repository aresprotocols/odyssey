use super::*;
use frame_support::traits::{EitherOfDiverse, EnsureOneOf};
use frame_system::EnsureRoot;
use pallet_treasury;
use part_council::{self, CouncilCollective};
use sp_runtime::Percent;

parameter_types! {
	pub const ProposalBond: Permill = Permill::from_percent(5);
	pub const ProposalBondMinimum: Balance = 100 * AMAS_UNITS;
	pub const ProposalBondMaximum: Balance = 500 * AMAS_UNITS;
	pub const SpendPeriod: BlockNumber = 1 * DAYS;
	pub const Burn: Permill = Permill::from_percent(50);

	pub const TreasuryPalletId: PalletId = PalletId(*b"py/trsry");
	pub const MaxApprovals: u32 = 100;

	pub const TipCountdown: BlockNumber = 1 * DAYS;
	pub const TipFindersFee: Percent = Percent::from_percent(20);
	pub const TipReportDepositBase: Balance = 1 * AMAS_UNITS;
	pub const DataDepositPerByte: Balance = 1 * AMAS_UNITS;
	pub const MaximumReasonLength: u32 = 300;
}

impl pallet_treasury::Config for Runtime {
    type PalletId = TreasuryPalletId;
    type Currency = Balances;
    type ApproveOrigin = EitherOfDiverse<
        EnsureRoot<AccountId>,
        pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 3, 5>,
    >;
    type RejectOrigin = EitherOfDiverse<
        EnsureRoot<AccountId>,
        pallet_collective::EnsureProportionMoreThan<AccountId, CouncilCollective, 1, 2>,
    >;
    type Event = Event;
    type OnSlash = Treasury;
    type ProposalBond = ProposalBond;
    type ProposalBondMinimum = ProposalBondMinimum;
    type ProposalBondMaximum = ();
    type SpendPeriod = SpendPeriod;
    type Burn = Burn;
    type BurnDestination = ();
    type SpendFunds = Bounties;
    type WeightInfo = pallet_treasury::weights::SubstrateWeight<Runtime>;
    type MaxApprovals = MaxApprovals;
    type SpendOrigin = frame_support::traits::NeverEnsureOrigin<u128>;
}
