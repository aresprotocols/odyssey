use super::*;
use manual_bridge;

impl manual_bridge::Config for Runtime {
    type Currency = pallet_balances::Pallet<Self>;
    type Event = Event;
    type RequestOrigin = frame_system::EnsureRoot<AccountId>;
    type WeightInfo = manual_bridge::weights::SubstrateWeight<Self>;
}