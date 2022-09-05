use super::*;
use pallet_collator_selection;
use sp_runtime::traits::Convert;

pub type SessionIndex = u32;

parameter_types! {
	pub const AresFinancePalletId: PalletId = PalletId(*b"aoe/fund");
	pub const BasicDollars: Balance = AMAS_UNITS;
	pub const AskPerEra: SessionIndex = 6;
	pub const HistoryDepth: u32 = 84;
}

impl oracle_finance::Config for Runtime {
    type Event = Event;
    type PalletId = AresFinancePalletId;
    type Currency = pallet_balances::Pallet<Self>;
    type BasicDollars = BasicDollars;
    type OnSlash = Treasury;
    type HistoryDepth = HistoryDepth;
    type SessionManager = CollatorSelection;
    type AskPerEra = AskPerEra;
    type ValidatorId = <Self as frame_system::Config>::AccountId;
    // type ValidatorIdOf = pallet_staking::StashOf<Self>;
    type ValidatorIdOf = SelfIsSelf<Self>;
}


pub struct SelfIsSelf<T>(sp_std::marker::PhantomData<T>);
impl<T: oracle_finance::Config> Convert<T::AccountId, Option<T::AccountId>> for SelfIsSelf<T> {
    fn convert(controller: T::AccountId) -> Option<T::AccountId> {
        Some(controller)
    }
}
