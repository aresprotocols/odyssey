use super::*;
use ares_oracle;
// pub use ares_oracle::LOCAL_STORAGE_PRICE_REQUEST_DOMAIN;
use ares_oracle_provider_support::crypto::sr25519::AuthorityId as AresId;
use frame_support::sp_runtime::app_crypto::sp_core::u32_trait::{_1, _2};
use frame_support::sp_std::marker::PhantomData;
use frame_support::traits::EnsureOneOf;
use staking_extend::IStakingNpos;
use governance::part_technical::TechnicalCollective;

pub type EnsureRootOrHalfTechnicalCollective = EnsureOneOf<
    EnsureRoot<AccountId>,
    pallet_collective::EnsureProportionAtLeast<_1, _2, AccountId, TechnicalCollective>,
>;

parameter_types! {
    pub const UnsignedPriority: u64 = 1 << 20;
    // pub const NeedVerifierCheck: bool = true;
    pub const CalculationKind: u8 = 1;
    pub const ErrLogPoolDepth: u32 = 1000;
}

// impl ares_oracle::Config for Runtime {
//     type Event = Event;
//     type Call = Call;
//     type OffchainAppCrypto = ares_oracle::AresCrypto<AresId>;
//     type AuthorityAres = AresId;
//     type UnsignedPriority = UnsignedPriority;
//     type FindAuthor = Aura;
//     type CalculationKind = CalculationKind;
//     type RequestOrigin = EnsureRootOrHalfTechnicalCollective;
//     type AuthorityCount = AresOracle; // ares_oracle::aura_handler::Pallet<Runtime>;
//     type OracleFinanceHandler = OracleFinance;
//     type AresIStakingNpos = NoNpos<Self>;
//     type ErrLogPoolDepth = ErrLogPoolDepth;
// }

impl ares_oracle::Config for Runtime {
    type Event = Event;
    type Call = Call;
    type OffchainAppCrypto = ares_oracle::ares_crypto::AresCrypto<AresId>;
    type AuthorityAres = AresId;
    type UnsignedPriority = UnsignedPriority;
    // type FindAuthor = Babe;
    type CalculationKind = CalculationKind;
    type RequestOrigin = EnsureRootOrHalfTechnicalCollective;
    type AuthorityCount = AresOracle; // ares_oracle::aura_handler::Pallet<Runtime>;
    type OracleFinanceHandler = OracleFinance;
    type AresIStakingNpos = (); //NoNpos<Self>;
    type ErrLogPoolDepth = ErrLogPoolDepth;
}

pub struct NoNpos<T>(PhantomData<T>);
impl <A,B,T:ares_oracle::Config> IStakingNpos<A, B> for NoNpos<T> {
    type StashId = <T as frame_system::Config>::AccountId;

    fn current_staking_era() -> u32 {
        0
    }

    fn near_era_change(period_multiple: B) -> bool {
        false
    }

    fn calculate_near_era_change(period_multiple: B, current_bn: B, session_length: B, per_era: B) -> bool {
        false
    }

    fn old_npos() -> Vec<Self::StashId> {
        Vec::new()
    }

    fn pending_npos() -> Vec<(Self::StashId, Option<A>)> {
        Vec::new()
    }
}

// impl<A, B, T> IStakingNpos<A, B> for NoNpos<T>
//     where T: ares_oracle::Config
// {
//     type StashId = T::AccountId;
//     fn current_staking_era() -> u32 {
//         0
//     }
//     fn near_era_change(_leading_period: B) -> bool {
//         false
//     }
//     fn calculate_near_era_change(_period_multiple: B, _current_bn: B, _session_length: B, _per_era: B) -> bool {
//         false
//     }
//     fn old_npos() -> sp_core::sp_std::vec::Vec<Self::StashId> {
//         Vec::new()
//     }
//     fn pending_npos() -> sp_core::sp_std::vec::Vec<(Self::StashId, Option<A>)> {
//         Vec::new()
//     }
// }