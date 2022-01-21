use super::*;
use ares_oracle;
pub use ares_oracle::LOCAL_STORAGE_PRICE_REQUEST_DOMAIN;
use ares_oracle_provider_support::crypto::sr25519::AuthorityId as AresId;
use frame_support::sp_runtime::app_crypto::sp_core::u32_trait::{_1, _2};
use governance::part_technical::TechnicalCollective;

pub type EnsureRootOrHalfTechnicalCollective = EnsureOneOf<
    AccountId,
    EnsureRoot<AccountId>,
    pallet_collective::EnsureProportionAtLeast<_1, _2, AccountId, TechnicalCollective>,
>;

impl ares_oracle::aura_handler::Config for Runtime {}

parameter_types! {
    pub const UnsignedPriority: u64 = 1 << 20;
    // pub const NeedVerifierCheck: bool = true;
    pub const CalculationKind: u8 = 1;
    pub const ErrLogPoolDepth: u32 = 1000;
}

impl ares_oracle::Config for Runtime {
    type Event = Event;
    type Call = Call;
    type OffchainAppCrypto = ares_oracle::AresCrypto<AresId>;
    type AuthorityAres = AresId;
    type UnsignedPriority = UnsignedPriority;
    type FindAuthor = Aura;
    type CalculationKind = CalculationKind;
    type RequestOrigin = EnsureRootOrHalfTechnicalCollective;
    type AuthorityCount = AresOracle; // ares_oracle::aura_handler::Pallet<Runtime>;
    type OracleFinanceHandler = OracleFinance;
    type AresIStakingNpos = Self;
    type ErrLogPoolDepth = ErrLogPoolDepth;
}
