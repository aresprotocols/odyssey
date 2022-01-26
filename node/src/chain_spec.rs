// Copyright 2019-2021 Parity Technologies (UK) Ltd.
// This file is part of Cumulus.

// Cumulus is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Cumulus is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Cumulus.  If not, see <http://www.gnu.org/licenses/>.

use cumulus_primitives_core::ParaId;
use hex_literal::hex;
use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use sc_service::ChainType;
use serde::{Deserialize, Serialize};
use sp_core::{crypto::UncheckedInto, sr25519, Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};

// use nimbus_primitives::NimbusId;
use sp_runtime::Perbill;

use sc_telemetry::serde_json;
use ares_para_common::Signature;

// use mars_runtime::{
//     AccountId as MarsRuntimeAccountId, Signature, AuraId, AresId,
//     constants::currency as MarsCurrency
// };

pub mod mars;
pub mod odyssey;
// pub mod template;

// pub type MarsChainSpec = sc_service::GenericChainSpec<mars_runtime::GenesisConfig, Extensions>;
// pub type OdysseyChainSpec = sc_service::GenericChainSpec<odyssey_runtime::GenesisConfig, Extensions>;

// pub const PARA_ID_OF_MARS: ParaId = ParaId::new(2008);
// pub const PARA_ID_OF_MARS: ParaId = ParaId::new(2000);
// pub const PARA_ID_OF_ODYSSEY: ParaId = ParaId::new(2028);

/// Helper function to generate a crypto pair from seed
// pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
//     TPublic::Pair::from_string(&format!("//{}", seed), None)
//         .expect("static values are valid; qed")
//         .public()
// }


/// The extensions for the [`ChainSpec`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ChainSpecGroup, ChainSpecExtension)]
#[serde(deny_unknown_fields)]
pub struct Extensions {
    /// The relay chain of the Parachain.
    pub relay_chain: String,
    /// The id of the Parachain.
    pub para_id: u32,
}

impl Extensions {
    /// Try to get the extension from the given `ChainSpec`.
    pub fn try_get(chain_spec: &dyn sc_service::ChainSpec) -> Option<&Self> {
        sc_chain_spec::get_extension(chain_spec.extensions())
    }
}

type AccountPublic = <Signature as Verify>::Signer;

// pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> MarsRuntimeAccountId
// where
//     AccountPublic: From<<TPublic::Pair as Pair>::Public>,
// {
//     AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
// }


// pub fn get_pair_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
//     TPublic::Pair::from_string(&format!("//{}", seed), None)
//         .expect("static values are valid; qed")
//         .public()
// }

// pub fn get_collator_keys_from_seed(seed: &str) -> AuraId {
//     get_pair_from_seed::<AuraId>(seed)
// }


