use super::*;
use ares_oracle;
use ares_oracle_provider_support::crypto::sr25519::AuthorityId as AresId;
use frame_support::sp_std::marker::PhantomData;
use frame_support::traits::EnsureOneOf;
use staking_extend::IStakingNpos;
use governance::part_technical::TechnicalCollective;
use codec::Encode;
use frame_support::{
    sp_runtime::{
        generic::{Era, SignedPayload},
        traits,
    }
};
use frame_support::traits::EitherOfDiverse;

// An index to a block.
pub type BlockNumber = u32;
pub type EnsureRootOrHalfTechnicalCollective = EitherOfDiverse<
    EnsureRoot<AccountId>,
    pallet_collective::EnsureProportionAtLeast<AccountId, TechnicalCollective, 1, 2>,
>;

parameter_types! {
    pub const UnsignedPriority: u64 = 1 << 20;
    pub const CalculationKind: u8 = 1;
    pub const ErrLogPoolDepth: u32 = 1000;
}

impl ares_oracle::Config for Runtime {
    type Event = Event;
    type Call = Call;
    type OffchainAppCrypto = ares_oracle::ares_crypto::AresCrypto<AresId>;
    type AuthorityAres = AresId;
    type UnsignedPriority = UnsignedPriority;
    type CalculationKind = CalculationKind;
    type RequestOrigin = EnsureRootOrHalfTechnicalCollective;
    type AuthorityCount = AresOracle; // ares_oracle::aura_handler::Pallet<Runtime>;
    type OracleFinanceHandler = OracleFinance;
    type AresIStakingNpos = (); //NoNpos<Self>;
    type ErrLogPoolDepth = ErrLogPoolDepth;
    type WeightInfo = ares_oracle::weights::SubstrateWeight<Self> ;
}

// TODO::Move NoNpos to ocw-suit lib
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