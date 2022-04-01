use super::*;
use mars_runtime;
// use mars_runtime::constants;
use ares_para_common::constants;
use mars_runtime::Balance as MarsBalance;
use mars_runtime::{AccountId, SS58Prefix, SessionKeys, Signature, StakerStatus};
use ares_para_common::{AresId, AuraId};
use mars_runtime::{
	AresOracleConfig, BalancesConfig, CollatorSelectionConfig, CouncilConfig, DemocracyConfig, ElectionsConfig,
	GenesisConfig, ParachainInfoConfig, SessionConfig, /*StakingConfig,*/ SudoConfig, SystemConfig,
	TechnicalCommitteeConfig, VestingConfig, WASM_BINARY,
};
use sc_chain_spec::ChainType;
use polkadot_service::ParaId;

const AMAS_ED: MarsBalance = ares_para_common::constants::currency::EXISTENTIAL_DEPOSIT;

pub type ChainSpec = sc_service::GenericChainSpec<mars_runtime::GenesisConfig, Extensions>;
pub const PARA_ID_NUM: u32 = 2008;
pub const PARA_ID: ParaId = ParaId::new(PARA_ID_NUM);

pub fn mars_session_keys(aura: AuraId, ares: AresId) -> SessionKeys {
	SessionKeys { aura, ares }
}

pub fn mars_development_config() -> ChainSpec {
	// Give your base currency a unit name and decimal places
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), "AMAS".into());
	properties.insert("tokenDecimals".into(), 12.into());
	properties.insert("SS58Prefix".into(), SS58Prefix::get().into());

	let initial_authorities: Vec<(
		AccountId, // stash
		AccountId, // controller
		AuraId,
		AresId,
	)> = vec![
		(
			hex!["70214e02fb2ec155a4c7bb8c122864b3b03f58c4ac59e8d83af7dc29851df657"].into(),
			hex!["aaf0c45982a423036601dcacc67854b38b854690d8e15bf1543e9a00e660e019"].into(),
			hex!["08ecdc14e2dd427724c60c6879a1aeade21d9708c30c4477f679dde971cb1378"].unchecked_into(),
			hex!["08ecdc14e2dd427724c60c6879a1aeade21d9708c30c4477f679dde971cb1378"].unchecked_into(),
		),
		(
			hex!["c82c3780d981812be804345618d27228680f61bb06a22689dcacf32b9be8815a"].into(),
			hex!["74a173a22757ddc9790ed388953a1ed8a5933a421858533411b36ebd41d74165"].into(),
			hex!["46bd24b721b0252e4c5b933b3c1b53b5179799511594695bf03f06d17b91154e"].unchecked_into(),
			hex!["46bd24b721b0252e4c5b933b3c1b53b5179799511594695bf03f06d17b91154e"].unchecked_into(),
		),
	];
	let endowed_accounts: Vec<AccountId> = vec![
		hex!["70214e02fb2ec155a4c7bb8c122864b3b03f58c4ac59e8d83af7dc29851df657"].into(),
		hex!["aaf0c45982a423036601dcacc67854b38b854690d8e15bf1543e9a00e660e019"].into(),
		hex!["c82c3780d981812be804345618d27228680f61bb06a22689dcacf32b9be8815a"].into(),
		hex!["74a173a22757ddc9790ed388953a1ed8a5933a421858533411b36ebd41d74165"].into(),
	];

	let council_members = endowed_accounts.clone();
	ChainSpec::from_genesis(
		"Mars",
		"mars_testnet",
		ChainType::Live,
		move || {
			mars_genesis(
				initial_authorities.clone(),
				vec![],
				hex!["aaf0c45982a423036601dcacc67854b38b854690d8e15bf1543e9a00e660e019"].into(),
				endowed_accounts.clone(),
				council_members.clone(),
				PARA_ID,
			)
		},
		Vec::new(),
		None,
		None,
		None,
		Some(properties),
		Extensions {
			relay_chain: "rococo-local".into(), // You MUST set this to the correct network!
			para_id: PARA_ID_NUM,
		},
	)
}

/* pub(crate) */
fn mars_genesis(
	initial_authorities: Vec<(AccountId, AccountId, AuraId, AresId)>,
	initial_nominators: Vec<AccountId>,
	root_key: AccountId,
	endowed_accounts: Vec<AccountId>,
	council_members: Vec<AccountId>,
	id: ParaId,
) -> GenesisConfig {
	const TOTAL_ISSUANCE: MarsBalance = constants::currency::AMAS_UNITS * 1_000_000_000; // one billion
	let endowment: MarsBalance = TOTAL_ISSUANCE / endowed_accounts.len() as u128;
	let elections_stash: MarsBalance = endowment / 1000;

	GenesisConfig {
		system: SystemConfig {
			code: WASM_BINARY
				.expect("WASM binary was not build, please build it!")
				.to_vec(),
			// changes_trie_config: Default::default(),
		},
		balances: BalancesConfig {
			balances: endowed_accounts.iter().cloned().map(|k| (k, endowment)).collect(),
		},

		parachain_info: ParachainInfoConfig { parachain_id: id },
		collator_selection: CollatorSelectionConfig {
			invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
			candidacy_bond: AMAS_ED,
			desired_candidates: 10u32,
		},
		parachain_system: Default::default(),
		session: SessionConfig {
			keys: initial_authorities
				.iter()
				.map(|(stash, controller, aura, ares)| {
					(
						stash.clone(),
						stash.clone(),
						mars_session_keys(aura.clone(), ares.clone()),
					)
				})
				.collect::<Vec<_>>(),
		},
		// no need to pass anything to aura, in fact it will panic if we do. Session will take care
		// of this.
		aura: Default::default(),
		aura_ext: Default::default(),
		sudo: SudoConfig { key: Some(root_key.clone()) },
		// staking: StakingConfig {
		// 	validator_count: initial_authorities.len() as u32,
		// 	minimum_validator_count: initial_authorities.len() as u32,
		// 	invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
		// 	slash_reward_fraction: Perbill::from_percent(10),
		// 	stakers,
		// 	..Default::default()
		// },
		council: CouncilConfig::default(),
		technical_committee: TechnicalCommitteeConfig {
			phantom: Default::default(),
			members: council_members.clone(),
		},
		vesting: VestingConfig { vesting: vec![] },
		treasury: Default::default(),
		democracy: DemocracyConfig::default(),
		elections: ElectionsConfig {
			members: council_members
				.clone()
				.iter()
				.map(|member| (member.clone(), elections_stash))
				.collect(),
		},
		ares_oracle: AresOracleConfig {
			_phantom: Default::default(),
			request_base: Vec::new(),
			price_pool_depth: 3u32,
			price_allowable_offset: 10u8,
			authorities: vec![],
			price_requests: vec![
				// price_key, request_uri, parse_version, fraction_num, request interval
				("btc-usdt".as_bytes().to_vec(), "btc".as_bytes().to_vec(), 2u32, 4, 2),
				("eth-usdt".as_bytes().to_vec(), "eth".as_bytes().to_vec(), 2u32, 4, 2),
				("dot-usdt".as_bytes().to_vec(), "dot".as_bytes().to_vec(), 2u32, 4, 2),
				("link-usdt".as_bytes().to_vec(), "link".as_bytes().to_vec(), 2u32, 4, 2),
				("ada-usdt".as_bytes().to_vec(), "ada".as_bytes().to_vec(), 2u32, 4, 4),
				("xrp-usdt".as_bytes().to_vec(), "xrp".as_bytes().to_vec(), 2u32, 4, 4),
				("sol-usdt".as_bytes().to_vec(), "sol".as_bytes().to_vec(), 2u32, 4, 4),
				("uni-usdt".as_bytes().to_vec(), "uni".as_bytes().to_vec(), 2u32, 4, 4),
				("bnb-usdt".as_bytes().to_vec(), "bnb".as_bytes().to_vec(), 2u32, 4, 4),
				(
					"1inch-usdt".as_bytes().to_vec(),
					"1inch".as_bytes().to_vec(),
					2u32,
					4,
					4,
				),
				("atom-usdt".as_bytes().to_vec(), "atom".as_bytes().to_vec(), 2u32, 4, 4),
				("trx-usdt".as_bytes().to_vec(), "trx".as_bytes().to_vec(), 2u32, 4, 4),
				("aave-usdt".as_bytes().to_vec(), "aave".as_bytes().to_vec(), 2u32, 4, 4),
				("snx-usdt".as_bytes().to_vec(), "snx".as_bytes().to_vec(), 2u32, 4, 4),
				("avax-usdt".as_bytes().to_vec(), "avax".as_bytes().to_vec(), 2u32, 4, 5),
				("ltc-usdt".as_bytes().to_vec(), "ltc".as_bytes().to_vec(), 2u32, 4, 5),
				("bch-usdt".as_bytes().to_vec(), "bch".as_bytes().to_vec(), 2u32, 4, 5),
				("fil-usdt".as_bytes().to_vec(), "fil".as_bytes().to_vec(), 2u32, 4, 5),
				("etc-usdt".as_bytes().to_vec(), "etc".as_bytes().to_vec(), 2u32, 4, 5),
				("eos-usdt".as_bytes().to_vec(), "eos".as_bytes().to_vec(), 2u32, 4, 5),
				("dash-usdt".as_bytes().to_vec(), "dash".as_bytes().to_vec(), 2u32, 4, 5),
				("comp-usdt".as_bytes().to_vec(), "comp".as_bytes().to_vec(), 2u32, 4, 5),
				(
					"matic-usdt".as_bytes().to_vec(),
					"matic".as_bytes().to_vec(),
					2u32,
					4,
					5,
				),
				("doge-usdt".as_bytes().to_vec(), "doge".as_bytes().to_vec(), 2u32, 4, 8),
				("luna-usdt".as_bytes().to_vec(), "luna".as_bytes().to_vec(), 2u32, 4, 8),
				("ftt-usdt".as_bytes().to_vec(), "ftt".as_bytes().to_vec(), 2u32, 4, 8),
				("xlm-usdt".as_bytes().to_vec(), "xlm".as_bytes().to_vec(), 2u32, 4, 8),
				("vet-usdt".as_bytes().to_vec(), "vet".as_bytes().to_vec(), 2u32, 4, 8),
				("icp-usdt".as_bytes().to_vec(), "icp".as_bytes().to_vec(), 2u32, 4, 8),
				(
					"theta-usdt".as_bytes().to_vec(),
					"theta".as_bytes().to_vec(),
					2u32,
					4,
					8,
				),
				("algo-usdt".as_bytes().to_vec(), "algo".as_bytes().to_vec(), 2u32, 4, 8),
				("xmr-usdt".as_bytes().to_vec(), "xmr".as_bytes().to_vec(), 2u32, 4, 8),
				("xtz-usdt".as_bytes().to_vec(), "xtz".as_bytes().to_vec(), 2u32, 4, 8),
				("egld-usdt".as_bytes().to_vec(), "egld".as_bytes().to_vec(), 2u32, 4, 8),
				("axs-usdt".as_bytes().to_vec(), "axs".as_bytes().to_vec(), 2u32, 4, 8),
				("iota-usdt".as_bytes().to_vec(), "iota".as_bytes().to_vec(), 2u32, 4, 8),
				("ftm-usdt".as_bytes().to_vec(), "ftm".as_bytes().to_vec(), 2u32, 4, 8),
				("ksm-usdt".as_bytes().to_vec(), "ksm".as_bytes().to_vec(), 2u32, 4, 4),
				("hbar-usdt".as_bytes().to_vec(), "hbar".as_bytes().to_vec(), 2u32, 4, 8),
				("neo-usdt".as_bytes().to_vec(), "neo".as_bytes().to_vec(), 2u32, 4, 8),
				(
					"waves-usdt".as_bytes().to_vec(),
					"waves".as_bytes().to_vec(),
					2u32,
					4,
					8,
				),
				("mkr-usdt".as_bytes().to_vec(), "mkr".as_bytes().to_vec(), 2u32, 4, 8),
				("near-usdt".as_bytes().to_vec(), "near".as_bytes().to_vec(), 2u32, 4, 8),
				("btt-usdt".as_bytes().to_vec(), "btt".as_bytes().to_vec(), 2u32, 4, 8),
				("chz-usdt".as_bytes().to_vec(), "chz".as_bytes().to_vec(), 2u32, 4, 8),
				("stx-usdt".as_bytes().to_vec(), "stx".as_bytes().to_vec(), 2u32, 4, 8),
				("dcr-usdt".as_bytes().to_vec(), "dcr".as_bytes().to_vec(), 2u32, 4, 8),
				("xem-usdt".as_bytes().to_vec(), "xem".as_bytes().to_vec(), 2u32, 4, 8),
				("omg-usdt".as_bytes().to_vec(), "omg".as_bytes().to_vec(), 2u32, 4, 8),
				("zec-usdt".as_bytes().to_vec(), "zec".as_bytes().to_vec(), 2u32, 4, 8),
				(
					"sushi-usdt".as_bytes().to_vec(),
					"sushi".as_bytes().to_vec(),
					2u32,
					4,
					8,
				),
				("enj-usdt".as_bytes().to_vec(), "enj".as_bytes().to_vec(), 2u32, 4, 8),
				("mana-usdt".as_bytes().to_vec(), "mana".as_bytes().to_vec(), 2u32, 4, 8),
				("yfi-usdt".as_bytes().to_vec(), "yfi".as_bytes().to_vec(), 2u32, 4, 8),
				("iost-usdt".as_bytes().to_vec(), "iost".as_bytes().to_vec(), 2u32, 4, 8),
				("qtum-usdt".as_bytes().to_vec(), "qtum".as_bytes().to_vec(), 2u32, 4, 8),
				("bat-usdt".as_bytes().to_vec(), "bat".as_bytes().to_vec(), 2u32, 4, 8),
				("zil-usdt".as_bytes().to_vec(), "zil".as_bytes().to_vec(), 2u32, 4, 8),
				("icx-usdt".as_bytes().to_vec(), "icx".as_bytes().to_vec(), 2u32, 4, 8),
				("grt-usdt".as_bytes().to_vec(), "grt".as_bytes().to_vec(), 2u32, 4, 8),
				("celo-usdt".as_bytes().to_vec(), "celo".as_bytes().to_vec(), 2u32, 4, 8),
				("zen-usdt".as_bytes().to_vec(), "zen".as_bytes().to_vec(), 2u32, 4, 8),
				("ren-usdt".as_bytes().to_vec(), "ren".as_bytes().to_vec(), 2u32, 4, 8),
				("sc-usdt".as_bytes().to_vec(), "sc".as_bytes().to_vec(), 2u32, 4, 8),
				("zrx-usdt".as_bytes().to_vec(), "zrx".as_bytes().to_vec(), 2u32, 4, 8),
				("ont-usdt".as_bytes().to_vec(), "ont".as_bytes().to_vec(), 2u32, 4, 8),
				("nano-usdt".as_bytes().to_vec(), "nano".as_bytes().to_vec(), 2u32, 4, 8),
				("crv-usdt".as_bytes().to_vec(), "crv".as_bytes().to_vec(), 2u32, 4, 8),
				("bnt-usdt".as_bytes().to_vec(), "bnt".as_bytes().to_vec(), 2u32, 4, 8),
				("fet-usdt".as_bytes().to_vec(), "fet".as_bytes().to_vec(), 2u32, 4, 8),
				("uma-usdt".as_bytes().to_vec(), "uma".as_bytes().to_vec(), 2u32, 4, 8),
				("iotx-usdt".as_bytes().to_vec(), "iotx".as_bytes().to_vec(), 2u32, 4, 8),
				("lrc-usdt".as_bytes().to_vec(), "lrc".as_bytes().to_vec(), 2u32, 4, 8),
				("sand-usdt".as_bytes().to_vec(), "sand".as_bytes().to_vec(), 2u32, 4, 8),
				("srm-usdt".as_bytes().to_vec(), "srm".as_bytes().to_vec(), 2u32, 4, 8),
				("kava-usdt".as_bytes().to_vec(), "kava".as_bytes().to_vec(), 2u32, 4, 8),
				("knc-usdt".as_bytes().to_vec(), "knc".as_bytes().to_vec(), 2u32, 4, 8),
			],
		},
	}
}
