use super::*;
pub use pallet_aura;
// use pallet_aura::Config;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;

impl cumulus_pallet_aura_ext::Config for Runtime {}

impl pallet_aura::Config for Runtime {
    type AuthorityId = AuraId;
    type DisabledValidators = Session;
}
