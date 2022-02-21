use super::*;
use pallet_collator_selection;

// pub type Balance = u64;
// pub type BlockNumber = u64;
// pub type AskPeriodNum = u64;
pub type SessionIndex = u32;
// pub const DOLLARS: u64 = 1_000_000_000_000;

parameter_types! {
    pub const AresFinancePalletId: PalletId = PalletId(*b"ocw/fund");
    pub const BasicDollars: Balance = AMAS_UNITS;
    // pub const AskPeriod: BlockNumber = 20 ; // * 10
    // pub const RewardPeriodCycle: AskPeriodNum = 3; // * 2 * 24
    // pub const RewardSlot: AskPeriodNum = 1; //
    pub const AskPerEra: SessionIndex = 6 * 24;
    pub const HistoryDepth: u32 = 5;
}

// impl oracle_finance::Config for Runtime {
//     type Event = Event;
//     type PalletId = AresFinancePalletId;
//     type Currency = pallet_balances::Pallet<Self>;
//     type BasicDollars = BasicDollars;
//     type AskPeriod = AskPeriod;
//     type RewardPeriodCycle = RewardPeriodCycle;
//     type RewardSlot = RewardSlot;
//     type OnSlash = (); // Treasury;
// }

impl oracle_finance::Config for Runtime {
    type Event = Event;
    type PalletId = AresFinancePalletId;
    type Currency = pallet_balances::Pallet<Self>;
    type BasicDollars = BasicDollars;
    type OnSlash = ();
    type HistoryDepth = HistoryDepth;
    type SessionManager = CollatorSelection; //pallet_session::historical::NoteHistoricalRoot<Self, Staking>;
    type AskPerEra = AskPerEra;
    type ValidatorId = <Self as frame_system::Config>::AccountId;
}
