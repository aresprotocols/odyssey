use super::*;

use ares_para_common::constants::currency::{AMAS_CENTS, AMAS_UNITS};
use ares_para_common::constants::time::MINUTES;
use frame_support::traits::{EitherOfDiverse};
use pallet_democracy;
use part_council::CouncilCollective;
use part_technical::TechnicalCollective;

parameter_types! {
    pub const LaunchPeriod: BlockNumber = 28 * 24 * 60 * MINUTES;
	// pub LaunchPeriod: BlockNumber = prod_or_fast!(28 * DAYS, 1, "ARES_LAUNCH_PERIOD");
    pub const VotingPeriod: BlockNumber = 28 * 24 * 60 * MINUTES;
	// pub VotingPeriod: BlockNumber = prod_or_fast!(28 * DAYS, 1 * MINUTES, "ARES_VOTING_PERIOD");
    pub const FastTrackVotingPeriod: BlockNumber = 3 * MINUTES;
	// pub FastTrackVotingPeriod: BlockNumber = prod_or_fast!(3 * HOURS, 1 * MINUTES, "ARES_FAST_TRACK_VOTING_PERIOD");
    pub const MinimumDeposit: Balance = 10000 * AMAS_UNITS;
	// pub const MinimumDeposit: Balance = 100 * DOLLARS * ARES_AMOUNT_MULT;
    pub const EnactmentPeriod: BlockNumber = 30 * 24 * 60 * MINUTES;
	// pub EnactmentPeriod: BlockNumber = prod_or_fast!(28 * DAYS, 1, "ARES_ENACTMENT_PERIOD");
    pub const CooloffPeriod: BlockNumber = 28 * 24 * 60 * MINUTES;
	// pub CooloffPeriod: BlockNumber = prod_or_fast!(7 * DAYS, 1, "ARES_COOLOFF_PERIOD");
	pub const InstantAllowed: bool = true;
    pub const PreimageByteDeposit: Balance = 100 * AMAS_CENTS;
	pub const MaxVotes: u32 = 100;
	pub const MaxProposals: u32 = 100;
}

impl pallet_democracy::Config for Runtime {
    type Proposal = Call;
    type Event = Event;
    type Currency = Balances;
    type EnactmentPeriod = EnactmentPeriod;
    type LaunchPeriod = LaunchPeriod;
    type VotingPeriod = VotingPeriod;
    type VoteLockingPeriod = EnactmentPeriod; // Same as EnactmentPeriod
    type MinimumDeposit = MinimumDeposit;
    type ExternalOrigin = pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 1, 2>;
    type ExternalMajorityOrigin = pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 3, 5>;
    type ExternalDefaultOrigin = pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 1, 1>;
    type FastTrackOrigin = pallet_collective::EnsureProportionAtLeast<AccountId, TechnicalCollective, 2, 3>;
    type InstantOrigin = pallet_collective::EnsureProportionAtLeast<AccountId, TechnicalCollective, 1, 1>;
    type InstantAllowed = InstantAllowed;
    type FastTrackVotingPeriod = FastTrackVotingPeriod;
    type CancellationOrigin = pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 2, 3>;
    type CancelProposalOrigin = EitherOfDiverse<
        EnsureRoot<AccountId>,
        pallet_collective::EnsureProportionAtLeast<AccountId, TechnicalCollective, 1, 1>,
    >;
    type BlacklistOrigin = EnsureRoot<AccountId>;
    type VetoOrigin = pallet_collective::EnsureMember<AccountId, TechnicalCollective>;
    type CooloffPeriod = CooloffPeriod;
    type PreimageByteDeposit = PreimageByteDeposit;
    type OperationalPreimageOrigin = pallet_collective::EnsureMember<AccountId, CouncilCollective>;
    type Slash = Treasury;
    type Scheduler = Scheduler;
    type PalletsOrigin = OriginCaller;
    type MaxVotes = MaxVotes;
    type WeightInfo = pallet_democracy::weights::SubstrateWeight<Runtime>;
    type MaxProposals = MaxProposals;
}