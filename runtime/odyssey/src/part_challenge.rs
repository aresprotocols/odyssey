use super::*;
use ares_para_common::constants::currency::AMAS_UNITS;
use governance;
pub use pallet_ares_challenge;
pub use pallet_collective;
use sp_core::u32_trait::{_3, _4};

parameter_types! {
	pub const MinimumDeposit: Balance = 100 * AMAS_UNITS ;
	pub const BidderMinimumDeposit: Balance = 1000 * AMAS_UNITS ;
	pub const ChallengePalletId: PalletId = PalletId(*b"py/ardem");
	pub const MinimumThreshold: u32 = governance::part_elections::DesiredMembers::get() / 3 * 2;
}

pub type Challenge = pallet_ares_challenge::Instance1;
impl pallet_ares_challenge::Config<Challenge> for Runtime {
    type Event = Event;
    type MinimumDeposit = MinimumDeposit;
    type PalletId = ChallengePalletId;
    type CouncilMajorityOrigin = pallet_collective::EnsureProportionAtLeast<_3, _4, AccountId, governance::part_council::CouncilCollective>;
    type Currency = Balances;
    type SlashProposer = AresChallenge;
    type IsAuthority = Aura; //Aura Or Babe
    type AuthorityId = AuraId;
    type Proposal = Call;
    type MinimumThreshold = MinimumThreshold;
}

