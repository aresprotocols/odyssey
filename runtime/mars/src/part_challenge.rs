use super::*;
use ares_para_common::{AccountId, AuraId};
use crate::governance;
use crate::governance::part_democracy::MinimumDeposit;

parameter_types! {
	pub const BidderMinimumDeposit: Balance = 10 * AMAS_UNITS ;
	pub const ChallengePalletId: PalletId = PalletId(*b"py/ardem");
	pub const MinimumThreshold: u32 = governance::part_elections::DesiredMembers::get() / 3 * 2;
}

pub type Challenge = pallet_ares_challenge::Instance1;
impl pallet_ares_challenge::Config<Challenge> for Runtime {
    type Event = Event;
    type MinimumDeposit = MinimumDeposit;
    type PalletId = ChallengePalletId;
    type CouncilMajorityOrigin = pallet_collective::EnsureProportionAtLeast<AccountId, governance::part_council::CouncilCollective,3,4>;
    type Currency = Balances;
    type SlashProposer = AresChallenge;
    type IsAuthority = Aura; //Aura Or Babe
    type AuthorityId = AuraId;
    type Proposal = Call; // (Aura or Babe) AuthorityId
    type MinimumThreshold = MinimumThreshold;
    type WeightInfo = pallet_ares_challenge::weights::SubstrateWeight<Self, Challenge>;
}
