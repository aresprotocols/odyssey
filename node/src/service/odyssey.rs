
use super::*;

use odyssey_runtime::{api::dispatch, native_version, RuntimeApi};
use ares_oracle_provider_support::LOCAL_STORAGE_PRICE_REQUEST_DOMAIN;
use log;
use sc_executor::NativeElseWasmExecutor;
use cumulus_client_consensus_aura::AuraConsensus;
use sp_runtime::app_crypto::AppKey;

// native_executor_instance!(
// 	pub RuntimeExecutor,
// 	dispatch,
// 	native_version,
// 	frame_benchmarking::benchmarking::HostFunctions,
// );

pub struct OdysseyRuntimeExecutor;
impl sc_executor::NativeExecutionDispatch for OdysseyRuntimeExecutor {
	type ExtendHostFunctions = frame_benchmarking::benchmarking::HostFunctions;
	fn dispatch(method: &str, data: &[u8]) -> Option<Vec<u8>> {
		odyssey_runtime::api::dispatch(method, data)
	}
	fn native_version() -> sc_executor::NativeVersion {
		odyssey_runtime::native_version()
	}
}

/// Build the import queue for the rococo parachain runtime.
pub fn parachain_build_import_queue(
	client: Arc<
		TFullClient<Block, RuntimeApi, WasmExecutor<HostFunctions>>,
	>,
	config: &Configuration,
	telemetry: Option<TelemetryHandle>,
	task_manager: &TaskManager,
) -> Result<
	sc_consensus::DefaultImportQueue<
		Block,
		TFullClient<Block, RuntimeApi, WasmExecutor<HostFunctions>>,
	>,
	sc_service::Error,
> {
	let slot_duration = cumulus_client_consensus_aura::slot_duration(&*client)?;

	cumulus_client_consensus_aura::import_queue::<
		sp_consensus_aura::sr25519::AuthorityPair,
		_,
		_,
		_,
		_,
		_,
		_,
	>(cumulus_client_consensus_aura::ImportQueueParams {
		block_import: client.clone(),
		client,
		create_inherent_data_providers: move |_, _| async move {
			let timestamp = sp_timestamp::InherentDataProvider::from_system_time();

			let slot =
				sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
					*timestamp,
					slot_duration,
				);

			Ok((timestamp, slot))
		},
		registry: config.prometheus_registry(),
		can_author_with: sp_consensus::AlwaysCanAuthor,
		spawner: &task_manager.spawn_essential_handle(),
		telemetry,
	}).map_err(Into::into)
}

// /// Build the import queue for the rococo parachain runtime.
// pub fn parachain_build_import_queue(
// 	client: Arc<TFullClient<Block, RuntimeApi, NativeElseWasmExecutor<OdysseyRuntimeExecutor>>>,
// 	config: &Configuration,
// 	telemetry: Option<TelemetryHandle>,
// 	task_manager: &TaskManager,
// ) -> Result<
// 	sc_consensus::DefaultImportQueue<
// 		Block,
// 		TFullClient<Block, RuntimeApi, NativeElseWasmExecutor<OdysseyRuntimeExecutor>>
// 	>, sc_service::Error
// >
// {
// 	let slot_duration = cumulus_client_consensus_aura::slot_duration(&*client)?;
//
// 	cumulus_client_consensus_aura::import_queue::<sp_consensus_aura::sr25519::AuthorityPair, _, _, _, _, _, _>(
// 		cumulus_client_consensus_aura::ImportQueueParams {
// 			block_import: client.clone(),
// 			client: client.clone(),
// 			create_inherent_data_providers: move |_, _| async move {
// 				let time = sp_timestamp::InherentDataProvider::from_system_time();
//
// 				let slot = sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
// 					*time,
// 					slot_duration,
// 				);
//
// 				Ok((time, slot))
// 			},
// 			registry: config.prometheus_registry().clone(),
// 			can_author_with: sp_consensus::CanAuthorWithNativeVersion::new(client.executor().clone()),
// 			spawner: &task_manager.spawn_essential_handle(),
// 			telemetry,
// 		},
// 	)
// 	.map_err(Into::into)
// }

/// Start a parachain node.
pub async fn start_parachain_node(
	parachain_config: Configuration,
	polkadot_config: Configuration,
	collator_options: CollatorOptions,
	id: ParaId,
	hwbench: Option<sc_sysinfo::HwBench>,
) -> sc_service::error::Result<(
	TaskManager,
	Arc<TFullClient<Block, RuntimeApi, WasmExecutor<HostFunctions>>>,
)> {
	start_node_impl::<RuntimeApi, _, _, _>(
		parachain_config,
		polkadot_config,
		collator_options,
		id,
		|_| Ok(RpcModule::new(())),
		parachain_build_import_queue,
		|client,
		 prometheus_registry,
		 telemetry,
		 task_manager,
		 relay_chain_interface,
		 transaction_pool,
		 sync_oracle,
		 keystore,
		 force_authoring| {
			let slot_duration = cumulus_client_consensus_aura::slot_duration(&*client)?;

			let proposer_factory = sc_basic_authorship::ProposerFactory::with_proof_recording(
				task_manager.spawn_handle(),
				client.clone(),
				transaction_pool,
				prometheus_registry,
				telemetry.clone(),
			);

			Ok(AuraConsensus::build::<sp_consensus_aura::sr25519::AuthorityPair, _, _, _, _, _, _>(
				BuildAuraConsensusParams {
					proposer_factory,
					create_inherent_data_providers: move |_, (relay_parent, validation_data)| {
						let relay_chain_interface = relay_chain_interface.clone();

						async move {
							let parachain_inherent =
								cumulus_primitives_parachain_inherent::ParachainInherentData::create_at(
									relay_parent,
									&relay_chain_interface,
									&validation_data,
									id,
								).await;

							let timestamp = sp_timestamp::InherentDataProvider::from_system_time();

							let slot =
								sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
									*timestamp,
									slot_duration,
								);

							let parachain_inherent = parachain_inherent.ok_or_else(|| {
								Box::<dyn std::error::Error + Send + Sync>::from(
									"Failed to create parachain inherent",
								)
							})?;

							Ok((timestamp, slot, parachain_inherent))
						}
					},
					block_import: client.clone(),
					para_client: client,
					backoff_authoring_blocks: Option::<()>::None,
					sync_oracle,
					keystore,
					force_authoring,
					slot_duration,
					// We got around 500ms for proposing
					block_proposal_slot_portion: SlotProportion::new(1f32 / 24f32),
					// And a maximum of 750ms if slots are skipped
					max_block_proposal_slot_portion: Some(SlotProportion::new(1f32 / 16f32)),
					telemetry,
				},
			))
		},
		hwbench,
	).await
}
