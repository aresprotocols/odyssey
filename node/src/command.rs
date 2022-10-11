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

use crate::{
	chain_spec,
	cli::{Cli, RelayChainCli, Subcommand},
	service,
	service::{new_partial, Block},
};
use codec::Encode;
// use cumulus_client_service::genesis::generate_genesis_block;
use cumulus_client_cli::generate_genesis_block;
use cumulus_primitives_core::ParaId;
use log::info;
// use polkadot_parachain::primitives::AccountIdConversion;
use sp_runtime::traits::{AccountIdConversion, Block as BlockT};
use polkadot_service::IdentifyVariant;
use sc_cli::{
	ChainSpec, CliConfiguration, DefaultConfigurationValues, ImportParams, KeystoreParams, NetworkParams, Result,
	RuntimeVersion, SharedParams, SubstrateCli,
};
use frame_benchmarking_cli::{BenchmarkCmd, SUBSTRATE_REFERENCE_HARDWARE};
use sc_service::{
	config::{BasePath, PrometheusConfig},
	TaskManager,
};
use ares_para_common::AuraId;
use sp_core::hexdisplay::HexDisplay;
// use sp_runtime::traits::Block as BlockT;
use std::{io::Write, net::SocketAddr};
use crate::service::odyssey::OdysseyRuntimeExecutor;

// default to the Statemint/Statemine/Westmint id
const DEFAULT_PARA_ID: u32 = 1000;

trait IdentifyChain {
	fn is_odyssey(&self) -> bool;
	fn is_mars(&self) -> bool;
	fn is_dev(&self) -> bool;
}

impl IdentifyChain for dyn sc_service::ChainSpec {
	fn is_odyssey(&self) -> bool {
		self.id().starts_with("odyssey")
	}
	fn is_mars(&self) -> bool {
		self.id().starts_with("mars")
	}
	fn is_dev(&self) -> bool {
		self.id().starts_with("dev")
	}
}

impl<T: sc_service::ChainSpec + 'static> IdentifyChain for T {
	fn is_odyssey(&self) -> bool {
		<dyn sc_service::ChainSpec>::is_odyssey(self)
	}
	fn is_mars(&self) -> bool {
		<dyn sc_service::ChainSpec>::is_mars(self)
	}
	fn is_dev(&self) -> bool {
		<dyn sc_service::ChainSpec>::is_dev(self)
	}
}

fn load_spec(id: &str) -> std::result::Result<Box<dyn sc_service::ChainSpec>, String> {
	Ok(match id {
		"" | "mars-dev" => Box::new(chain_spec::mars::mars_development_config()),
		"dev" => Box::new(chain_spec::mars::mars_development_config()),
		"mars" => {
			Box::new(chain_spec::mars::mars_development_config())
		},
		"" | "odyssey" => {
			Box::new(chain_spec::odyssey::odyssey_development_config())
		},
		path => {
			let chain_spec = chain_spec::odyssey::ChainSpec::from_json_file(path.into())?;
			if chain_spec.is_mars() {
				Box::new(chain_spec::mars::ChainSpec::from_json_file(path.into())?)
			} else if chain_spec.is_dev() {
				Box::new(chain_spec::mars::ChainSpec::from_json_file(path.into())?)
			} else if chain_spec.is_odyssey() {
				Box::new(chain_spec::odyssey::ChainSpec::from_json_file(path.into())?)
			} else {
				Box::new(chain_spec)
			}

		}
	})
}

impl SubstrateCli for Cli {
	fn impl_name() -> String {
		"Ares collator".into()
	}

	fn impl_version() -> String {
		env!("SUBSTRATE_CLI_IMPL_VERSION").into()
	}

	fn description() -> String {
		format!(
			"Polkadot collator\n\nThe command-line arguments provided first will be \
		passed to the parachain node, while the arguments provided after -- will be passed \
		to the relaychain node.\n\n\
		{} [parachain-args] -- [relaychain-args]",
			Self::executable_name()
		)
	}

	fn author() -> String {
		env!("CARGO_PKG_AUTHORS").into()
	}

	fn support_url() -> String {
		"https://github.com/paritytech/cumulus/issues/new".into()
	}

	fn copyright_start_year() -> i32 {
		2017
	}

	fn load_spec(&self, id: &str) -> std::result::Result<Box<dyn sc_service::ChainSpec>, String> {
		load_spec(id)
	}

	fn native_runtime_version(chain_spec: &Box<dyn ChainSpec>) -> &'static RuntimeVersion {
		if chain_spec.is_mars() {
			&mars_runtime::VERSION
		} else if chain_spec.is_dev() {
			&mars_runtime::VERSION
		} else if chain_spec.is_odyssey() {
			// TODO fix
			&odyssey_runtime::VERSION
			// &mars_runtime::VERSION
		} else {
			&mars_runtime::VERSION
		}
	}
}

impl SubstrateCli for RelayChainCli {
	fn impl_name() -> String {
		"Polkadot collator".into()
	}

	fn impl_version() -> String {
		env!("SUBSTRATE_CLI_IMPL_VERSION").into()
	}

	fn description() -> String {
		format!(
			"Polkadot collator\n\nThe command-line arguments provided first will be \
		passed to the parachain node, while the arguments provided after -- will be passed \
		to the relaychain node.\n\n\
		{} [parachain-args] -- [relaychain-args]",
			Self::executable_name()
		)
	}

	fn author() -> String {
		env!("CARGO_PKG_AUTHORS").into()
	}

	fn support_url() -> String {
		"https://github.com/paritytech/cumulus/issues/new".into()
	}

	fn copyright_start_year() -> i32 {
		2017
	}

	fn load_spec(&self, id: &str) -> std::result::Result<Box<dyn sc_service::ChainSpec>, String> {
		polkadot_cli::Cli::from_iter([RelayChainCli::executable_name().to_string()].iter()).load_spec(id)
	}

	fn native_runtime_version(chain_spec: &Box<dyn ChainSpec>) -> &'static RuntimeVersion {
		polkadot_cli::Cli::native_runtime_version(chain_spec)
	}
}

fn extract_genesis_wasm(chain_spec: &Box<dyn sc_service::ChainSpec>) -> Result<Vec<u8>> {
	let mut storage = chain_spec.build_storage()?;

	storage
		.top
		.remove(sp_core::storage::well_known_keys::CODE)
		.ok_or_else(|| "Could not find wasm file in genesis state!".into())
}

/// Creates partial components for the runtimes that are supported by the benchmarks.
macro_rules! construct_benchmark_partials {
	($config:expr, |$partials:ident| $code:expr) => {

		if $config.chain_spec.is_odyssey() {
			let $partials = new_partial::<odyssey_runtime::RuntimeApi, _>(
				&$config,
				crate::service::aura_build_import_queue::<_, AuraId>,
			)?;
			$code
		} else {
			let $partials = new_partial::<mars_runtime::RuntimeApi, _>(
				&$config,
				crate::service::aura_build_import_queue::<_, AuraId>,
			)?;
			$code
		}
	};
}

macro_rules! construct_async_run {
	(|$components:ident, $cli:ident, $cmd:ident, $config:ident| $( $code:tt )* ) => {{
		let runner = $cli.create_runner($cmd)?;

		if runner.config().chain_spec.is_mars() {
			runner.async_run(|$config| {
				let $components = new_partial::<mars_runtime::RuntimeApi, _>(
					&$config,
					crate::service::mars::parachain_build_import_queue,
				)?;
				let task_manager = $components.task_manager;
				{ $( $code )* }.map(|v| (v, task_manager))
			})
		} else  if runner.config().chain_spec.is_odyssey() {
			runner.async_run(|$config| {
					let $components = new_partial::<odyssey_runtime::RuntimeApi, _>(
					&$config,
					crate::service::odyssey::parachain_build_import_queue,
				)?;
				let task_manager = $components.task_manager;
				{ $( $code )* }.map(|v| (v, task_manager))
			})
		} else {
			runner.async_run(|$config| {
					let $components = new_partial::<mars_runtime::RuntimeApi, _>(
					&$config,
					crate::service::mars::parachain_build_import_queue,
				)?;
				let task_manager = $components.task_manager;
				{ $( $code )* }.map(|v| (v, task_manager))
			})
		}

	}}
}

/// Parse command line arguments into service configuration.
pub fn run() -> Result<()> {
	let cli = Cli::from_args();

	match &cli.subcommand {
		Some(Subcommand::Key(cmd)) => cmd.run(&cli),
		Some(Subcommand::BuildSpec(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.sync_run(|config| cmd.run(config.chain_spec, config.network))
		}
		Some(Subcommand::CheckBlock(cmd)) => {
			construct_async_run!(|components, cli, cmd, config| {
				Ok(cmd.run(components.client, components.import_queue))
			})
		}
		Some(Subcommand::ExportBlocks(cmd)) => {
			construct_async_run!(|components, cli, cmd, config| { Ok(cmd.run(components.client, config.database)) })
		}
		Some(Subcommand::ExportState(cmd)) => {
			construct_async_run!(|components, cli, cmd, config| { Ok(cmd.run(components.client, config.chain_spec)) })
		}
		Some(Subcommand::ImportBlocks(cmd)) => {
			construct_async_run!(|components, cli, cmd, config| {
				Ok(cmd.run(components.client, components.import_queue))
			})
		}
		Some(Subcommand::PurgeChain(cmd)) => {
			let runner = cli.create_runner(cmd)?;

			runner.sync_run(|config| {
				let polkadot_cli = RelayChainCli::new(
					&config,
					[RelayChainCli::executable_name().to_string()]
						.iter()
						.chain(cli.relaychain_args.iter()),
				);

				let polkadot_config =
					SubstrateCli::create_configuration(&polkadot_cli, &polkadot_cli, config.tokio_handle.clone())
						.map_err(|err| format!("Relay chain argument error: {}", err))?;

				cmd.run(config, polkadot_config)
			})
		}
		// Some(Subcommand::Revert(cmd)) => {
		// 	construct_async_run!(|components, cli, cmd, config| { Ok(cmd.run(components.client, components.backend)) })
		// }
		Some(Subcommand::Revert(cmd)) => construct_async_run!(|components, cli, cmd, config| {
			Ok(cmd.run(components.client, components.backend, None))
		}),
		// Some(Subcommand::ExportGenesisState(params)) => {
		// 	let mut builder = sc_cli::LoggerBuilder::new("");
		// 	builder.with_profiling(sc_tracing::TracingReceiver::Log, "");
		// 	let _ = builder.init();
		//
		// 	// let block: crate::service::Block = generate_genesis_block(&load_spec(
		// 	// 	&params.chain.clone().unwrap_or_default(),
		// 	// )?)?;
		//
		// 	let spec = load_spec(&params.chain.clone().unwrap_or_default())?;
		// 	let state_version = Cli::native_runtime_version(&spec).state_version();
		// 	let block: crate::service::Block = generate_genesis_block(&spec, state_version)?;
		//
		// 	let raw_header = block.header().encode();
		// 	let output_buf = if params.raw {
		// 		raw_header
		// 	} else {
		// 		format!("0x{:?}", HexDisplay::from(&block.header().encode())).into_bytes()
		// 	};
		//
		// 	if let Some(output) = &params.output {
		// 		std::fs::write(output, output_buf)?;
		// 	} else {
		// 		std::io::stdout().write_all(&output_buf)?;
		// 	}
		//
		// 	Ok(())
		// }
		Some(Subcommand::ExportGenesisState(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.sync_run(|_config| {
				let spec = cli.load_spec(&cmd.shared_params.chain.clone().unwrap_or_default())?;
				let state_version = Cli::native_runtime_version(&spec).state_version();
				cmd.run::<crate::service::Block>(&*spec, state_version)
			})
		},
		// Some(Subcommand::ExportGenesisWasm(cmd)) => {
		// 	let mut builder = sc_cli::LoggerBuilder::new("");
		// 	builder.with_profiling(sc_tracing::TracingReceiver::Log, "");
		// 	let _ = builder.init();
		//
		// 	let raw_wasm_blob = extract_genesis_wasm(&cli.load_spec(&cmd.chain.clone().unwrap_or_default())?)?;
		// 	let output_buf = if cmd.raw {
		// 		raw_wasm_blob
		// 	} else {
		// 		format!("0x{:?}", HexDisplay::from(&raw_wasm_blob)).into_bytes()
		// 	};
		//
		// 	if let Some(output) = &cmd.output {
		// 		std::fs::write(output, output_buf)?;
		// 	} else {
		// 		std::io::stdout().write_all(&output_buf)?;
		// 	}
		//
		// 	Ok(())
		// }
		Some(Subcommand::ExportGenesisWasm(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.sync_run(|_config| {
				let spec = cli.load_spec(&cmd.shared_params.chain.clone().unwrap_or_default())?;
				cmd.run(&*spec)
			})
		},
		// Some(Subcommand::Benchmark(cmd)) => {
		// 	if cfg!(feature = "runtime-benchmarks") {
		// 		let runner = cli.create_runner(cmd)?;
		//
		// 		/*if runner.config().chain_spec.is_statemine() {
		// 			runner.sync_run(|config| cmd.run::<Block, StatemineRuntimeExecutor>(config))
		// 		} else if runner.config().chain_spec.is_westmint() {
		// 			runner.sync_run(|config| cmd.run::<Block, WestmintRuntimeExecutor>(config))
		// 		} else if runner.config().chain_spec.is_statemint() {
		// 			runner.sync_run(|config| cmd.run::<Block, StatemintRuntimeExecutor>(config))
		// 		} else */
		// 		if runner.config().chain_spec.is_mars() {
		// 			todo!("Not implement for mars.")
		// 		// runner.sync_run(|config| cmd.run::<Block, StatemintRuntimeExecutor>(config))
		// 		} else if runner.config().chain_spec.is_odyssey() {
		// 			todo!("Not implement for odyssey.")
		// 		// runner.sync_run(|config| cmd.run::<Block, StatemintRuntimeExecutor>(config))
		// 		} else {
		// 			Err("Chain doesn't support benchmarking".into())
		// 		}
		// 	} else {
		// 		Err("Benchmarking wasn't enabled when building the node. \
		// 		You can enable it with `--features runtime-benchmarks`."
		// 			.into())
		// 	}
		// },
		Some(Subcommand::Benchmark(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			// Switch on the concrete benchmark sub-command-
			match cmd {
				BenchmarkCmd::Pallet(cmd) =>
					if cfg!(feature = "runtime-benchmarks") {
						runner.sync_run(|config| {
							if config.chain_spec.is_mars() {
								todo!("Not implement for odyssey.")
							} else if config.chain_spec.is_odyssey() {
								todo!("Not implement for odyssey.")
							} else {
								Err("Chain doesn't support benchmarking".into())
							}
						})
					} else {
						Err("Benchmarking wasn't enabled when building the node. \
				You can enable it with `--features runtime-benchmarks`."
							.into())
					},
				BenchmarkCmd::Block(cmd) => runner.sync_run(|config| {
					construct_benchmark_partials!(config, |partials| cmd.run(partials.client))
				}),
				BenchmarkCmd::Storage(cmd) => runner.sync_run(|config| {
					construct_benchmark_partials!(config, |partials| {
						let db = partials.backend.expose_db();
						let storage = partials.backend.expose_storage();

						cmd.run(config, partials.client.clone(), db, storage)
					})
				}),
				BenchmarkCmd::Machine(cmd) =>
					runner.sync_run(|config| cmd.run(&config, SUBSTRATE_REFERENCE_HARDWARE.clone())),
				// NOTE: this allows the Client to leniently implement
				// new benchmark commands without requiring a companion MR.
				#[allow(unreachable_patterns)]
				_ => Err("Benchmarking sub-command unsupported".into()),
			}
		},
		Some(Subcommand::TryRuntime(cmd)) => {
			if cfg!(feature = "try-runtime") {
				// grab the task manager.
				let runner = cli.create_runner(cmd)?;
				let registry = &runner.config().prometheus_config.as_ref().map(|cfg| &cfg.registry);
				let task_manager =
					TaskManager::new(runner.config().tokio_handle.clone(), *registry)
						.map_err(|e| format!("Error: {:?}", e))?;

				if runner.config().chain_spec.is_odyssey() {
					runner.async_run(|config| {
						Ok((cmd.run::<Block, OdysseyRuntimeExecutor>(config), task_manager))
					})
				} else {
					Err("Chain doesn't support try-runtime".into())
				}
			} else {
				Err("Try-runtime must be enabled by `--features try-runtime`.".into())
			}
		},
		None => {
			// let runner = cli.create_runner(&cli.run.normalize())?;
			let runner = cli.create_runner(&cli.run.normalize())?;
			let collator_options = cli.run.collator_options();

			runner.run_node_until_exit(|config| async move {

				let hwbench = if !cli.no_hardware_benchmarks {
					config.database.path().map(|database_path| {
						let _ = std::fs::create_dir_all(&database_path);
						sc_sysinfo::gather_hwbench(Some(database_path))
					})
				} else {
					None
				};


				let polkadot_cli = RelayChainCli::new(
					&config,
					[RelayChainCli::executable_name()].iter().chain(cli.relaychain_args.iter()),
				);

				let para_id = chain_spec::Extensions::try_get(&*config.chain_spec)
					.map(|e| e.para_id)
					.ok_or_else(|| "Could not find parachain ID in chain-spec.")?;

				let id = ParaId::from(para_id);

				let parachain_account =
					AccountIdConversion::<polkadot_primitives::v2::AccountId>::into_account_truncating(&id);

				let state_version = Cli::native_runtime_version(&config.chain_spec).state_version();
				let block: Block = generate_genesis_block(&*config.chain_spec, state_version)
					.map_err(|e| format!("{:?}", e))?;
				let genesis_state = format!("0x{:?}", HexDisplay::from(&block.header().encode()));

				let tokio_handle = config.tokio_handle.clone();
				let polkadot_config =
					SubstrateCli::create_configuration(&polkadot_cli, &polkadot_cli, tokio_handle)
						.map_err(|err| format!("Relay chain argument error: {}", err))?;

				info!("Parachain id: {:?}", id);
				info!("Parachain Account: {}", parachain_account);
				info!("Parachain genesis state: {}", genesis_state);
				info!("Is collating: {}", if config.role.is_authority() { "yes" } else { "no" });

				if config.chain_spec.is_mars() {
					// crate::service::mars::start_parachain_node(config, polkadot_config, id, get_warehouse_params(cli))
					// 	.await
					// 	.map(|r| r.0)
					// 	.map_err(Into::into)
					crate::service::mars::start_parachain_node(
						config,
						polkadot_config,
						collator_options,
						id,
						hwbench,
					).await.map(|r| r.0).map_err(Into::into)
				} else if config.chain_spec.is_dev() {
					// crate::service::mars::start_parachain_node(config, polkadot_config, id, get_warehouse_params(cli))
					// 	.await
					// 	.map(|r| r.0)
					// 	.map_err(Into::into)
					crate::service::mars::start_parachain_node(
						config,
						polkadot_config,
						collator_options,
						id,
						hwbench,
					).await.map(|r| r.0).map_err(Into::into)
				} else if config.chain_spec.is_odyssey() {
					// crate::service::odyssey::start_parachain_node(config, polkadot_config, id, get_warehouse_params(cli))
					// 	.await
					// 	.map(|r| r.0)
					// 	.map_err(Into::into)
					crate::service::odyssey::start_parachain_node(
						config,
						polkadot_config,
						collator_options,
						id,
						hwbench,
					).await.map(|r| r.0).map_err(Into::into)
				}  else {
					// crate::service::mars::start_parachain_node(config, polkadot_config, id, get_warehouse_params(cli))
					// 	.await
					// 	.map(|r| r.0)
					// 	.map_err(Into::into)
					crate::service::mars::start_parachain_node(
						config,
						polkadot_config,
						collator_options,
						id,
						hwbench,
					).await.map(|r| r.0).map_err(Into::into)
				}
			})
		}

	}
}

// Make some ares peculiar params with `Cli`, like as warehouse and ares_keys
fn get_warehouse_params(cli: Cli) -> Vec<(&'static str, Option<Vec<u8>>)> {
	// ares params
	let mut ares_params: Vec<(&str, Option<Vec<u8>>)> = Vec::new();
	let request_base = match cli.warehouse {
		None => {
			panic!("⛔ Start parameter `--warehouse` is required!");
		}
		Some(request_url) => request_url.as_str().as_bytes().to_vec(),
	};
	ares_params.push(("warehouse", Some(request_base)));
	ares_params
}

impl DefaultConfigurationValues for RelayChainCli {
	fn p2p_listen_port() -> u16 {
		30334
	}

	fn rpc_ws_listen_port() -> u16 {
		9945
	}

	fn rpc_http_listen_port() -> u16 {
		9934
	}

	fn prometheus_listen_port() -> u16 {
		9616
	}
}

impl CliConfiguration<Self> for RelayChainCli {
	fn shared_params(&self) -> &SharedParams {
		self.base.base.shared_params()
	}

	fn import_params(&self) -> Option<&ImportParams> {
		self.base.base.import_params()
	}

	fn network_params(&self) -> Option<&NetworkParams> {
		self.base.base.network_params()
	}

	fn keystore_params(&self) -> Option<&KeystoreParams> {
		self.base.base.keystore_params()
	}

	fn base_path(&self) -> Result<Option<BasePath>> {
		Ok(self
			.shared_params()
			.base_path()
			.or_else(|| self.base_path.clone().map(Into::into)))
	}

	fn rpc_http(&self, default_listen_port: u16) -> Result<Option<SocketAddr>> {
		self.base.base.rpc_http(default_listen_port)
	}

	fn rpc_ipc(&self) -> Result<Option<String>> {
		self.base.base.rpc_ipc()
	}

	fn rpc_ws(&self, default_listen_port: u16) -> Result<Option<SocketAddr>> {
		self.base.base.rpc_ws(default_listen_port)
	}

	fn prometheus_config(
		&self,
		default_listen_port: u16,
		chain_spec: &Box<dyn ChainSpec>,
	) -> Result<Option<PrometheusConfig>> {
		self.base.base.prometheus_config(default_listen_port, chain_spec)
	}

	fn init<F>(
		&self,
		_support_url: &String,
		_impl_version: &String,
		_logger_hook: F,
		_config: &sc_service::Configuration,
	) -> Result<()>
		where
			F: FnOnce(&mut sc_cli::LoggerBuilder, &sc_service::Configuration),
	{
		unreachable!("PolkadotCli is never initialized; qed");
	}

	fn chain_id(&self, is_dev: bool) -> Result<String> {
		let chain_id = self.base.base.chain_id(is_dev)?;

		Ok(if chain_id.is_empty() { self.chain_id.clone().unwrap_or_default() } else { chain_id })
	}

	fn role(&self, is_dev: bool) -> Result<sc_service::Role> {
		self.base.base.role(is_dev)
	}

	fn transaction_pool(&self, is_dev: bool) -> Result<sc_service::config::TransactionPoolOptions> {
		self.base.base.transaction_pool(is_dev)
	}

	fn state_cache_child_ratio(&self) -> Result<Option<usize>> {
		self.base.base.state_cache_child_ratio()
	}

	fn rpc_methods(&self) -> Result<sc_service::config::RpcMethods> {
		self.base.base.rpc_methods()
	}

	fn rpc_ws_max_connections(&self) -> Result<Option<usize>> {
		self.base.base.rpc_ws_max_connections()
	}

	fn rpc_cors(&self, is_dev: bool) -> Result<Option<Vec<String>>> {
		self.base.base.rpc_cors(is_dev)
	}

	fn default_heap_pages(&self) -> Result<Option<u64>> {
		self.base.base.default_heap_pages()
	}

	fn force_authoring(&self) -> Result<bool> {
		self.base.base.force_authoring()
	}

	fn disable_grandpa(&self) -> Result<bool> {
		self.base.base.disable_grandpa()
	}

	fn max_runtime_instances(&self) -> Result<Option<usize>> {
		self.base.base.max_runtime_instances()
	}

	fn announce_block(&self) -> Result<bool> {
		self.base.base.announce_block()
	}

	fn telemetry_endpoints(
		&self,
		chain_spec: &Box<dyn ChainSpec>,
	) -> Result<Option<sc_telemetry::TelemetryEndpoints>> {
		self.base.base.telemetry_endpoints(chain_spec)
	}

	fn node_name(&self) -> Result<String> {
		self.base.base.node_name()
	}
}
