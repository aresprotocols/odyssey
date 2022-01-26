use super::*;
use constants::currency::AMAS_UNITS;
use governance;
pub use pallet_ares_challenge;
pub use pallet_ares_collective;
use sp_core::u32_trait::{_3, _4};

parameter_types! {
    pub const MinimumDeposit: Balance = 100 * AMAS_UNITS;
    pub const BidderMinimumDeposit: Balance = 1000 * AMAS_UNITS;
    pub const DemoPalletId: PalletId = PalletId(*b"py/ardem");
}

impl pallet_ares_challenge::Config for Runtime {
    type Event = Event;
    type MinimumDeposit = MinimumDeposit;
    type PalletId = DemoPalletId;
    type CouncilMajorityOrigin = pallet_ares_collective::EnsureProportionAtLeast<
        _3,
        _4,
        AccountId,
        governance::part_council::CouncilCollective,
    >;
    type Currency = Balances;
    type SlashProposer = AresChallenge;
    type BidderMinimumDeposit = BidderMinimumDeposit;
    type IsAuthority = Aura; //Aura Or Babe
    type AuthorityId = AuraId; // (Aura or Babe) AuthorityId
                               // type FindAuthor = pallet_aura::FindAccountFromAuthorIndex<Self, Aura>;
}
