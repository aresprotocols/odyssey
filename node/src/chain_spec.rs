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
use mars_runtime::{
    AccountId as MarsRuntimeAccountId, Signature, AuraId, AresId,
    constants::currency as MarsCurrency
};

pub mod mars;
pub mod odyssey;
pub mod template;

// pub type MarsChainSpec = sc_service::GenericChainSpec<mars_runtime::GenesisConfig, Extensions>;
// pub type OdysseyChainSpec = sc_service::GenericChainSpec<odyssey_runtime::GenesisConfig, Extensions>;

// pub const PARA_ID_OF_MARS: ParaId = ParaId::new(2008);
// pub const PARA_ID_OF_MARS: ParaId = ParaId::new(2000);
// pub const PARA_ID_OF_ODYSSEY: ParaId = ParaId::new(2028);

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}


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

/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> MarsRuntimeAccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

// pub fn get_chain_spec(id: ParaId) -> ChainSpec {
//     ChainSpec::from_genesis(
//         "Local Testnet",
//         "local_testnet",
//         ChainType::Local,
//         move || {
//             testnet_genesis(
//                 vec![
//                     (
//                         //AccountId::from_str("f24FF3a9CF04c71Dbc94D0b566f7A27B94566cac").unwrap(),
//                         get_account_id_from_seed::<sr25519::Public>("Alice"),
//                         get_from_seed::<NimbusId>("Alice"),
//                         1000 * MarsCurrency::AMAS_UNITS, //1000
//                     ),
//                     (
//                         get_account_id_from_seed::<sr25519::Public>("Bob"),
//                         get_from_seed::<NimbusId>("Bob"),
//                         1000 * MarsCurrency::AMAS_UNITS, //1000
//                     ),
//                 ],
//                 vec![],
//                 get_account_id_from_seed::<sr25519::Public>("Alice"),
//                 // initial collators.
//                 vec![
//                     (
//                         get_account_id_from_seed::<sr25519::Public>("Alice"),
//                         get_collator_keys_from_seed("Alice"),
//                     ),
//                     (
//                         get_account_id_from_seed::<sr25519::Public>("Bob"),
//                         get_collator_keys_from_seed("Bob"),
//                     ),
//                 ],
//                 // vec![
//                 // 	get_from_seed::<AuraId>("Alice"),
//                 // 	get_from_seed::<AuraId>("Bob"),
//                 // ],
//                 vec![
//                     get_account_id_from_seed::<sr25519::Public>("Alice"),
//                     get_account_id_from_seed::<sr25519::Public>("Bob"),
//                     get_account_id_from_seed::<sr25519::Public>("Charlie"),
//                     get_account_id_from_seed::<sr25519::Public>("Dave"),
//                     get_account_id_from_seed::<sr25519::Public>("Eve"),
//                     get_account_id_from_seed::<sr25519::Public>("Ferdie"),
//                     get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
//                     get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
//                     get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
//                     get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
//                     get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
//                     get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
//                 ],
//                 id,
//             )
//         },
//         vec![],
//         None,
//         None,
//         Some(
//             serde_json::from_str(
//                 "{\"tokenDecimals\": 12, \"tokenSymbol\": \"AMAS\", \"SS58Prefix\": 34}",
//             )
//             .expect("Provided valid json map"),
//         ),
//         Extensions {
//             relay_chain: "westend".into(),
//             para_id: id.into(),
//         },
//     )
// }



// pub fn odyssey_development_config(id: ParaId) -> OdysseyChainSpec {
//     OdysseyChainSpec::from_genesis(
//         "Ares Odyssey",
//         "ares_odyssey",
//         ChainType::Live,
//         move || {
//             genesis_of_odyssey::odyssey_genesis(
//                 vec![
//                     // (
//                     // 	 hex!["a4cbc9391b20d6dd89d1667165c355a524120fd10fe056ba80bb63f3057a0f73"].into(),
//                     // 	 hex!["3caff95b42c408e7f0096ed4aaedb5524940c8c1a62f46c6289f34e7e5fb1166"].unchecked_into(),
//                     // 	 1000 * constants::currency::AMAS_UNITS,
//                     //  ),
//                     //  (
//                     // 	 hex!["78b90fe626734aef1a03426a1ebeaf1a98d747d00c8ef250bf1e70b8bf87ab62"].into(),
//                     // 	 hex!["86989f896cff3671e0fe36578712d555c9f4b71fe94c9d9516a0313cc789b505"].unchecked_into(),
//                     // 	 1000 * constants::currency::AMAS_UNITS, //1000
//                     //  ),
//                 ],
//                 vec![],
//                 hex!["aaf0c45982a423036601dcacc67854b38b854690d8e15bf1543e9a00e660e019"].into(),
//                 vec![
//                     (
//                         hex!("aaf0c45982a423036601dcacc67854b38b854690d8e15bf1543e9a00e660e019")
//                             .into(),
//                         hex!("aaf0c45982a423036601dcacc67854b38b854690d8e15bf1543e9a00e660e019")
//                             .unchecked_into(),
//                     ),
//                     (
//                         hex!("74a173a22757ddc9790ed388953a1ed8a5933a421858533411b36ebd41d74165")
//                             .into(),
//                         hex!("74a173a22757ddc9790ed388953a1ed8a5933a421858533411b36ebd41d74165")
//                             .unchecked_into(),
//                     ),
//                 ],
//                 vec![
//                     hex!["70214e02fb2ec155a4c7bb8c122864b3b03f58c4ac59e8d83af7dc29851df657"].into(),
//                     hex!["aaf0c45982a423036601dcacc67854b38b854690d8e15bf1543e9a00e660e019"].into(),
//                     hex!["c82c3780d981812be804345618d27228680f61bb06a22689dcacf32b9be8815a"].into(),
//                     hex!["74a173a22757ddc9790ed388953a1ed8a5933a421858533411b36ebd41d74165"].into(),
//                     hex!["acad76a1f273ab3b8e453d630d347668f1cfa9b01605800dab7126a494c04c7c"].into(),
//                     hex!["9e55f821f7b3484f15942af308001c32f113f31444f420a77422702907510669"].into(),
//                     hex!["4aa6e0eeed2e3d1f35a8eb1cd650451327ad378fb8975dbf5747016ff3be2460"].into(),
//                     hex!["587bae319ecaee13ce2dbdedfc6d66eb189e5af427666b21b4d4a08c7af0671c"].into(),
//                 ],
//                 PARA_ID_OF_ODYSSEY,
//             )
//         },
//         Vec::new(),
//         None,
//         None,
//         Some(
//             serde_json::from_str(
//                 "{\"tokenDecimals\": 12, \"tokenSymbol\": \"ARES\", \"SS58Prefix\": 34}",
//             )
//             .expect("Provided valid json map"),
//         ),
//         Extensions {
//             relay_chain: "polkadot".into(),
//             para_id: PARA_ID_OF_ODYSSEY.into(),
//         },
//     )
// }


// fn testnet_genesis(
//     candidates: Vec<(AccountId, NimbusId, Balance)>,
//     nominations: Vec<(AccountId, AccountId, Balance)>,
//     root_key: AccountId,
//     invulnerables: Vec<(AccountId, AuraId)>,
//     // initial_authorities: Vec<AuraId>,
//     endowed_accounts: Vec<AccountId>,
//     id: ParaId,
// ) -> rococo_parachain_runtime::GenesisConfig {
//     const TOTAL_ISSUANCE: Balance = constants::currency::AMAS_UNITS * 1_000_000_000; // one billion
//     let endowment: Balance = TOTAL_ISSUANCE / endowed_accounts.len() as u128;
//
//     rococo_parachain_runtime::GenesisConfig {
//         system: rococo_parachain_runtime::SystemConfig {
//             code: rococo_parachain_runtime::WASM_BINARY
//                 .expect("WASM binary was not build, please build it!")
//                 .to_vec(),
//             changes_trie_config: Default::default(),
//         },
//         balances: rococo_parachain_runtime::BalancesConfig {
//             balances: endowed_accounts
//                 .iter()
//                 .cloned()
//                 .map(|k| (k, endowment))
//                 .collect(),
//         },
//         sudo: rococo_parachain_runtime::SudoConfig {
//             key: root_key.clone(),
//         },
//         parachain_info: rococo_parachain_runtime::ParachainInfoConfig { parachain_id: id },
//         // collator_selection: rococo_parachain_runtime::CollatorSelectionConfig {
//         // 	invulnerables: invulnerables.iter().cloned().map(|(acc, _)| acc).collect(),
//         // 	candidacy_bond: AMAS_ED,
//         // 	..Default::default()
//         // },
//         // session: rococo_parachain_runtime::SessionConfig {
//         // 	keys: invulnerables
//         // 		.iter()
//         // 		.cloned()
//         // 		.map(|(acc, aura)| {
//         // 			(
//         // 				acc.clone(),                  // account id
//         // 				acc.clone(),                  // validator id
//         // 				rococo_parachain_session_keys(aura), // session keys
//         // 			)
//         // 		})
//         // 		.collect(),
//         // },
//         // democracy: DemocracyConfig::default(),
//         // council: CouncilConfig::default(),
//         // technical_committee: TechnicalCommitteeConfig {
//         // 	members: endowed_accounts
//         // 		.iter()
//         // 		.take((endowed_accounts.len() + 1) / 2)
//         // 		.cloned()
//         // 		.collect(),
//         // 	phantom: Default::default(),
//         // },
//         // treasury: Default::default(),
//         aura: Default::default(),
//         aura_ext: Default::default(),
//         parachain_system: Default::default(),
//         // vesting: rococo_parachain_runtime::VestingConfig { vesting: vec![] },
//         // parachain_staking: ParachainStakingConfig {
//         // 	candidates: candidates
//         // 		.iter()
//         // 		.cloned()
//         // 		.map(|(account, _, bond)| (account, bond))
//         // 		.collect(),
//         // 	nominations,
//         // 	inflation_config: ares_inflation_config(),
//         // },
//         // ocw_module: OCWModuleConfig {
//         // 	_phantom: Default::default(),
//         // 	request_base: Vec::new(),
//         // 	price_pool_depth: 3u32,
//         // 	price_allowable_offset: 10u8,
//         // 	price_requests: vec![
//         // 		// price_key, request_uri, parse_version, fraction_num, request interval
//         // 		("btc-usdt".as_bytes().to_vec(), "btcusdt".as_bytes().to_vec(), 2u32, 4, 2),
//         // 		("eth-usdt".as_bytes().to_vec(), "ethusdt".as_bytes().to_vec(), 2u32, 4, 2),
//         // 		("dot-usdt".as_bytes().to_vec(), "dotusdt".as_bytes().to_vec(), 2u32, 4, 2),
//         // 		("link-usdt".as_bytes().to_vec(), "linkusdt".as_bytes().to_vec(), 2u32, 4, 2),
//         //
//         // 		("ada-usdt".as_bytes().to_vec(), "adausdt".as_bytes().to_vec(), 2u32, 4, 4),
//         // 		("xrp-usdt".as_bytes().to_vec(), "xrpusdt".as_bytes().to_vec(), 2u32, 4, 4),
//         // 		("sol-usdt".as_bytes().to_vec(), "solusdt".as_bytes().to_vec(), 2u32, 4, 4),
//         // 		("uni-usdt".as_bytes().to_vec(), "uniusdt".as_bytes().to_vec(), 2u32, 4, 4),
//         // 		("bnb-usdt".as_bytes().to_vec(), "bnbusdt".as_bytes().to_vec(), 2u32, 4, 4),
//         // 		("1inch-usdt".as_bytes().to_vec(), "1INCHusdt".as_bytes().to_vec(), 2u32, 4, 4),
//         // 		("atom-usdt".as_bytes().to_vec(), "atomusdt".as_bytes().to_vec(), 2u32, 4, 4),
//         // 		("trx-usdt".as_bytes().to_vec(), "trxusdt".as_bytes().to_vec(), 2u32, 4, 4),
//         // 		("aave-usdt".as_bytes().to_vec(), "aaveusdt".as_bytes().to_vec(), 2u32, 4, 4),
//         // 		("snx-usdt".as_bytes().to_vec(), "snxusdt".as_bytes().to_vec(), 2u32, 4, 4),
//         //
//         // 		("avax-usdt".as_bytes().to_vec(), "avaxusdt".as_bytes().to_vec(), 2u32, 4, 5),
//         // 		("ltc-usdt".as_bytes().to_vec(), "ltcusdt".as_bytes().to_vec(), 2u32, 4, 5),
//         // 		("bch-usdt".as_bytes().to_vec(), "bchusdt".as_bytes().to_vec(), 2u32, 4, 5),
//         // 		("fil-usdt".as_bytes().to_vec(), "filusdt".as_bytes().to_vec(), 2u32, 4, 5),
//         // 		("etc-usdt".as_bytes().to_vec(), "etcusdt".as_bytes().to_vec(), 2u32, 4, 5),
//         // 		("eos-usdt".as_bytes().to_vec(), "eosusdt".as_bytes().to_vec(), 2u32, 4, 5),
//         // 		("dash-usdt".as_bytes().to_vec(), "dashusdt".as_bytes().to_vec(), 2u32, 4, 5),
//         // 		("comp-usdt".as_bytes().to_vec(), "compusdt".as_bytes().to_vec(), 2u32, 4, 5),
//         // 		("matic-usdt".as_bytes().to_vec(), "maticusdt".as_bytes().to_vec(), 2u32, 4, 5),
//         //
//         // 		("doge-usdt".as_bytes().to_vec(), "dogeusdt".as_bytes().to_vec(), 2u32, 4, 8),
//         // 		("luna-usdt".as_bytes().to_vec(), "lunausdt".as_bytes().to_vec(), 2u32, 4, 8),
//         // 		("ftt-usdt".as_bytes().to_vec(), "fttusdt".as_bytes().to_vec(), 2u32, 4, 8),
//         // 		("xlm-usdt".as_bytes().to_vec(), "xlmusdt".as_bytes().to_vec(), 2u32, 4, 8),
//         // 		("vet-usdt".as_bytes().to_vec(), "vetusdt".as_bytes().to_vec(), 2u32, 4, 8),
//         // 		("icp-usdt".as_bytes().to_vec(), "icpusdt".as_bytes().to_vec(), 2u32, 4, 8),
//         // 		("theta-usdt".as_bytes().to_vec(), "thetausdt".as_bytes().to_vec(), 2u32, 4, 8),
//         // 		("algo-usdt".as_bytes().to_vec(), "algousdt".as_bytes().to_vec(), 2u32, 4, 8),
//         // 		("xmr-usdt".as_bytes().to_vec(), "xmrusdt".as_bytes().to_vec(), 2u32, 4, 8),
//         // 		("xtz-usdt".as_bytes().to_vec(), "xtzusdt".as_bytes().to_vec(), 2u32, 4, 8),
//         // 		("egld-usdt".as_bytes().to_vec(), "egldusdt".as_bytes().to_vec(), 2u32, 4, 8),
//         // 		("axs-usdt".as_bytes().to_vec(), "axsusdt".as_bytes().to_vec(), 2u32, 4, 8),
//         // 		("iota-usdt".as_bytes().to_vec(), "iotausdt".as_bytes().to_vec(), 2u32, 4, 8),
//         // 		("ftm-usdt".as_bytes().to_vec(), "ftmusdt".as_bytes().to_vec(), 2u32, 4, 8),
//         // 		("ksm-usdt".as_bytes().to_vec(), "ksmusdt".as_bytes().to_vec(), 2u32, 4, 4),
//         // 		("hbar-usdt".as_bytes().to_vec(), "hbarusdt".as_bytes().to_vec(), 2u32, 4, 8),
//         // 		("neo-usdt".as_bytes().to_vec(), "neousdt".as_bytes().to_vec(), 2u32, 4, 8),
//         // 		("waves-usdt".as_bytes().to_vec(), "wavesusdt".as_bytes().to_vec(), 2u32, 4, 8),
//         // 		("mkr-usdt".as_bytes().to_vec(), "mkrusdt".as_bytes().to_vec(), 2u32, 4, 8),
//         // 		("near-usdt".as_bytes().to_vec(), "nearusdt".as_bytes().to_vec(), 2u32, 4, 8),
//         // 		("btt-usdt".as_bytes().to_vec(), "bttusdt".as_bytes().to_vec(), 2u32, 4, 8),
//         // 		("chz-usdt".as_bytes().to_vec(), "chzusdt".as_bytes().to_vec(), 2u32, 4, 8),
//         // 		("stx-usdt".as_bytes().to_vec(), "stxusdt".as_bytes().to_vec(), 2u32, 4, 8),
//         // 		("dcr-usdt".as_bytes().to_vec(), "dcrusdt".as_bytes().to_vec(), 2u32, 4, 8),
//         // 		("xem-usdt".as_bytes().to_vec(), "xemusdt".as_bytes().to_vec(), 2u32, 4, 8),
//         // 		("omg-usdt".as_bytes().to_vec(), "omgusdt".as_bytes().to_vec(), 2u32, 4, 8),
//         // 		("zec-usdt".as_bytes().to_vec(), "zecusdt".as_bytes().to_vec(), 2u32, 4, 8),
//         // 		("sushi-usdt".as_bytes().to_vec(), "sushiusdt".as_bytes().to_vec(), 2u32, 4, 8),
//         // 		("enj-usdt".as_bytes().to_vec(), "enjusdt".as_bytes().to_vec(), 2u32, 4, 8),
//         // 		("mana-usdt".as_bytes().to_vec(), "manausdt".as_bytes().to_vec(), 2u32, 4, 8),
//         // 		("yfi-usdt".as_bytes().to_vec(), "yfiusdt".as_bytes().to_vec(), 2u32, 4, 8),
//         // 		("iost-usdt".as_bytes().to_vec(), "iostusdt".as_bytes().to_vec(), 2u32, 4, 8),
//         // 		("qtum-usdt".as_bytes().to_vec(), "qtumusdt".as_bytes().to_vec(), 2u32, 4, 8),
//         // 		("bat-usdt".as_bytes().to_vec(), "batusdt".as_bytes().to_vec(), 2u32, 4, 8),
//         // 		("zil-usdt".as_bytes().to_vec(), "zilusdt".as_bytes().to_vec(), 2u32, 4, 8),
//         // 		("icx-usdt".as_bytes().to_vec(), "icxusdt".as_bytes().to_vec(), 2u32, 4, 8),
//         // 		("grt-usdt".as_bytes().to_vec(), "grtusdt".as_bytes().to_vec(), 2u32, 4, 8),
//         // 		("celo-usdt".as_bytes().to_vec(), "celousdt".as_bytes().to_vec(), 2u32, 4, 8),
//         // 		("zen-usdt".as_bytes().to_vec(), "zenusdt".as_bytes().to_vec(), 2u32, 4, 8),
//         // 		("ren-usdt".as_bytes().to_vec(), "renusdt".as_bytes().to_vec(), 2u32, 4, 8),
//         // 		("sc-usdt".as_bytes().to_vec(), "scusdt".as_bytes().to_vec(), 2u32, 4, 8),
//         // 		("zrx-usdt".as_bytes().to_vec(), "zrxusdt".as_bytes().to_vec(), 2u32, 4, 8),
//         // 		("ont-usdt".as_bytes().to_vec(), "ontusdt".as_bytes().to_vec(), 2u32, 4, 8),
//         // 		("nano-usdt".as_bytes().to_vec(), "nanousdt".as_bytes().to_vec(), 2u32, 4, 8),
//         // 		("crv-usdt".as_bytes().to_vec(), "crvusdt".as_bytes().to_vec(), 2u32, 4, 8),
//         // 		("bnt-usdt".as_bytes().to_vec(), "bntusdt".as_bytes().to_vec(), 2u32, 4, 8),
//         // 		("fet-usdt".as_bytes().to_vec(), "fetusdt".as_bytes().to_vec(), 2u32, 4, 8),
//         // 		("uma-usdt".as_bytes().to_vec(), "umausdt".as_bytes().to_vec(), 2u32, 4, 8),
//         // 		("iotx-usdt".as_bytes().to_vec(), "iotxusdt".as_bytes().to_vec(), 2u32, 4, 8),
//         // 		("lrc-usdt".as_bytes().to_vec(), "lrcusdt".as_bytes().to_vec(), 2u32, 4, 8),
//         // 		("sand-usdt".as_bytes().to_vec(), "sandusdt".as_bytes().to_vec(), 2u32, 4, 8),
//         // 		("srm-usdt".as_bytes().to_vec(), "srmusdt".as_bytes().to_vec(), 2u32, 4, 8),
//         // 		("1inch-usdt".as_bytes().to_vec(), "1inch-usdt".as_bytes().to_vec(), 2u32, 4, 4),
//         // 		("kava-usdt".as_bytes().to_vec(), "kavausdt".as_bytes().to_vec(), 2u32, 4, 8),
//         // 		("knc-usdt".as_bytes().to_vec(), "kncusdt".as_bytes().to_vec(), 2u32, 4, 8),
//         // 	],
//         // },
//     }
// }


/// Helper function to generate a crypto pair from seed
pub fn get_pair_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

/// Generate collator keys from seed.
///
/// This function's return type must always match the session keys of the chain in tuple format.
pub fn get_collator_keys_from_seed(seed: &str) -> AuraId {
    get_pair_from_seed::<AuraId>(seed)
}


