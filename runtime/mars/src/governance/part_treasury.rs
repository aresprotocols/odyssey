use super::*;
use pallet_treasury;
use part_council::CouncilCollective;
use sp_runtime::{Percent, Permill};
use frame_support::{PalletId};
// use frame_system::{EnsureOneOf, EnsureRoot};
use frame_support::traits::EnsureOneOf;
use frame_system::{EnsureRoot};

use sp_core::u32_trait::{_1, _2, _3, _5};

parameter_types! {
    pub const ProposalBond: Permill = Permill::from_percent(5);
    pub const ProposalBondMinimum: Balance = 1 * AMAS_UNITS;
    pub const SpendPeriod: BlockNumber = 1 * DAYS;
    pub const Burn: Permill = Permill::from_percent(50);
    // pub const TipCountdown: BlockNumber = 1 * DAYS;
    // pub const TipFindersFee: Percent = Percent::from_percent(20);
    pub const TipReportDepositBase: Balance = 1 * AMAS_UNITS;
    pub const DataDepositPerByte: Balance = 1 * AMAS_CENTS;
    pub const BountyDepositBase: Balance = 1 * AMAS_UNITS;
    pub const BountyDepositPayoutDelay: BlockNumber = 1 * DAYS;
    pub const TreasuryPalletId: PalletId = PalletId(*b"py/trsry");
    pub const BountyUpdatePeriod: BlockNumber = 14 * DAYS;
    pub const MaximumReasonLength: u32 = 16384;
    pub const BountyCuratorDeposit: Permill = Permill::from_percent(50);
    pub const BountyValueMinimum: Balance = 5 * AMAS_UNITS;
    pub const MaxApprovals: u32 = 100;
    pub const MaxActiveChildBountyCount: u32 = 5;
	pub const ChildBountyValueMinimum: Balance = 1 * AMAS_UNITS;
	pub const ChildBountyCuratorDepositBase: Permill = Permill::from_percent(10);
}

impl pallet_treasury::Config for Runtime {
    type PalletId = TreasuryPalletId;
    type Currency = Balances;
    type ApproveOrigin = EnsureOneOf<
        EnsureRoot<AccountId>,
        pallet_collective::EnsureProportionAtLeast<_3, _5, AccountId, CouncilCollective>,
    >;
    type RejectOrigin = EnsureOneOf<
        EnsureRoot<AccountId>,
        pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, CouncilCollective>,
    >;
    type Event = Event;
    type OnSlash = ();
    type ProposalBond = ProposalBond;
    type ProposalBondMinimum = ProposalBondMinimum;
    type ProposalBondMaximum = ();
    type SpendPeriod = SpendPeriod;
    type Burn = Burn;
    type BurnDestination = ();
    type SpendFunds = Bounties;
    type WeightInfo = pallet_treasury::weights::SubstrateWeight<Runtime>;
    type MaxApprovals = MaxApprovals;
}
