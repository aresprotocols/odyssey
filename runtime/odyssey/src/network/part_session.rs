use super::*;
use pallet_collator_selection;
use pallet_session;

impl_opaque_keys! {
    pub struct SessionKeys {
        pub aura: Aura,
        pub ares: AresOracle,
    }
}

parameter_types! {
    // pub const DisabledValidatorsThreshold: Perbill = Perbill::from_percent(17);
    pub const Period: u32 = EPOCH_DURATION_IN_BLOCKS as u32; // 100 block = 10min  [10b = 1min] [10min = 100b]
    pub const Offset: u32 = 0;
}

impl pallet_session::Config for Runtime {
    type Event = Event;
    type ValidatorId = <Self as frame_system::Config>::AccountId;
    // we don't have stash and controller, thus we don't need the convert as well.
    type ValidatorIdOf = pallet_collator_selection::IdentityCollator;
    type ShouldEndSession = pallet_session::PeriodicSessions<Period, Offset>;
    type NextSessionRotation = pallet_session::PeriodicSessions<Period, Offset>;
    type SessionManager = CollatorSelection;
    // Essentially just Aura, but lets be pedantic.
    // type SessionHandler = <SessionKeys as sp_runtime::traits::OpaqueKeys>::KeyTypeIdProviders;
    type SessionHandler = <SessionKeys as sp_runtime::traits::OpaqueKeys>::KeyTypeIdProviders;
    type Keys = SessionKeys;
    // type DisabledValidatorsThreshold = DisabledValidatorsThreshold;
    type WeightInfo = weights::pallet_session::WeightInfo<Runtime>;
}

// impl pallet_session::historical::Config for Runtime {
//     type FullIdentification = pallet_staking::Exposure<AccountId, Balance>;
//     type FullIdentificationOf = pallet_staking::ExposureOf<Runtime>;
// }
