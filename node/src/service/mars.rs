use super::*;

use mars_runtime::{api::dispatch, native_version, RuntimeApi};
// use mars_runtime::part_oracle::LOCAL_STORAGE_PRICE_REQUEST_DOMAIN;
native_executor_instance!(
	pub RuntimeExecutor,
	dispatch,
	native_version,
	frame_benchmarking::benchmarking::HostFunctions,
);
use log;

/// Build the import queue for the rococo parachain runtime.
pub fn parachain_build_import_queue(
	client: Arc<TFullClient<Block, RuntimeApi, RuntimeExecutor>>,
	config: &Configuration,
	telemetry: Option<TelemetryHandle>,
	task_manager: &TaskManager,
) -> Result<sc_consensus::DefaultImportQueue<Block, TFullClient<Block, RuntimeApi, RuntimeExecutor>>, sc_service::Error>
{
	let slot_duration = cumulus_client_consensus_aura::slot_duration(&*client)?;

	cumulus_client_consensus_aura::import_queue::<sp_consensus_aura::sr25519::AuthorityPair, _, _, _, _, _, _>(
		cumulus_client_consensus_aura::ImportQueueParams {
			block_import: client.clone(),
			client: client.clone(),
			create_inherent_data_providers: move |_, _| async move {
				let time = sp_timestamp::InherentDataProvider::from_system_time();

				let slot = sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_duration(
					*time,
					slot_duration.slot_duration(),
				);

				Ok((time, slot))
			},
			registry: config.prometheus_registry().clone(),
			can_author_with: sp_consensus::CanAuthorWithNativeVersion::new(client.executor().clone()),
			spawner: &task_manager.spawn_essential_handle(),
			telemetry,
		},
	)
	.map_err(Into::into)
}

/// Start a parachain node.
pub async fn start_parachain_node(
	parachain_config: Configuration,
	polkadot_config: Configuration,
	id: ParaId,
    ares_params: Vec<(&str, Option<Vec<u8>>)>,
) -> sc_service::error::Result<(TaskManager, Arc<TFullClient<Block, RuntimeApi, RuntimeExecutor>>)> {
	start_node_impl::<RuntimeApi, RuntimeExecutor, _, _, _>(
		parachain_config,
		polkadot_config,
		id,
		|_| Ok(Default::default()),
		parachain_build_import_queue,
		|client,
		 backend,
		 prometheus_registry,
		 telemetry,
		 task_manager,
		 relay_chain_node,
		 transaction_pool,
		 sync_oracle,
		 keystore,
		 force_authoring| {

            // TODO insert ares params
            log::info!("🚅 Setting ares_params :-) {:?}", ares_params);
            let backend_clone = backend.clone();
            let result: Vec<(&str, bool)> = ares_params
                .iter()
                .map(|(order, x)| {
                    match order {
                        &"warehouse" => {
                            match x {
                                None => (*order, false),
                                Some(exe_vecu8) => {
                                    let request_base_str = sp_std::str::from_utf8(exe_vecu8).unwrap();
                                    let store_request_u8 = request_base_str.encode();
                                    log::info!("setting request_domain: {:?}", request_base_str);
                                    if let Some(mut offchain_db) = backend_clone.offchain_storage() {
                                        log::debug!("after setting request_domain: {:?}", request_base_str);
                                        // offchain_db.set(
                                        //     STORAGE_PREFIX,
                                        //     LOCAL_STORAGE_PRICE_REQUEST_DOMAIN,
                                        //     store_request_u8.as_slice(),
                                        // );
                                    }
                                    (*order, true)
                                }
                            }
                        }
                        &_ => ("NONE", false),
                    }
                }).collect();
            log::info!("🚅 Results of Ares settings:{:?}", result);

            let slot_duration = cumulus_client_consensus_aura::slot_duration(&*client)?;
			let proposer_factory = sc_basic_authorship::ProposerFactory::with_proof_recording(
				task_manager.spawn_handle(),
				client.clone(),
				transaction_pool,
				prometheus_registry.clone(),
				telemetry.clone(),
			);

			let relay_chain_backend = relay_chain_node.backend.clone();
			let relay_chain_client = relay_chain_node.client.clone();
			Ok(build_aura_consensus::<
				sp_consensus_aura::sr25519::AuthorityPair,
				_,
				_,
				_,
				_,
				_,
				_,
				_,
				_,
				_,
			>(BuildAuraConsensusParams {
				proposer_factory,
				create_inherent_data_providers: move |_, (relay_parent, validation_data)| {
					let parachain_inherent =
						cumulus_primitives_parachain_inherent::ParachainInherentData::create_at_with_client(
							relay_parent,
							&relay_chain_client,
							&*relay_chain_backend,
							&validation_data,
							id,
						);
					async move {
						let time = sp_timestamp::InherentDataProvider::from_system_time();

						let slot = sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_duration(
							*time,
							slot_duration.slot_duration(),
						);

						let parachain_inherent = parachain_inherent.ok_or_else(|| {
							Box::<dyn std::error::Error + Send + Sync>::from("Failed to create parachain inherent")
						})?;
						Ok((time, slot, parachain_inherent))
					}
				},
				block_import: client.clone(),
				relay_chain_client: relay_chain_node.client.clone(),
				relay_chain_backend: relay_chain_node.backend.clone(),
				para_client: client.clone(),
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
			}))
		},
	)
	.await
}
