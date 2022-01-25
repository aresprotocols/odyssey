use super::*;

use template_runtime::{
    native_version,
    api::dispatch,
    RuntimeApi
};
use sc_executor::NativeElseWasmExecutor;

// native_executor_instance!(
//     pub RuntimeExecutor,
//     dispatch,
//     native_version,
//     frame_benchmarking::benchmarking::HostFunctions,
// );

pub struct TemplateRuntimeExecutor;
impl sc_executor::NativeExecutionDispatch for TemplateRuntimeExecutor {
    type ExtendHostFunctions = frame_benchmarking::benchmarking::HostFunctions;
    fn dispatch(method: &str, data: &[u8]) -> Option<Vec<u8>> {
        template_runtime::api::dispatch(method, data)
    }
    fn native_version() -> sc_executor::NativeVersion {
        template_runtime::native_version()
    }
}

/// Build the import queue for the rococo parachain runtime.
pub fn parachain_build_import_queue(
    client: Arc<TFullClient<Block, RuntimeApi, NativeElseWasmExecutor<TemplateRuntimeExecutor>>>,
    config: &Configuration,
    telemetry: Option<TelemetryHandle>,
    task_manager: &TaskManager,
) -> Result<
    sc_consensus::DefaultImportQueue<
        Block,
        TFullClient<Block, RuntimeApi, NativeElseWasmExecutor<TemplateRuntimeExecutor>>,
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
        client: client.clone(),
        create_inherent_data_providers: move |_, _| async move {
            let time = sp_timestamp::InherentDataProvider::from_system_time();

            let slot =
                sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_duration(
                    *time,
                    slot_duration.slot_duration(),
                );

            Ok((time, slot))
        },
        registry: config.prometheus_registry().clone(),
        can_author_with: sp_consensus::CanAuthorWithNativeVersion::new(client.executor().clone()),
        spawner: &task_manager.spawn_essential_handle(),
        telemetry,
    })
        .map_err(Into::into)
}

/// Start a rococo parachain node.
pub async fn start_parachain_node(
    parachain_config: Configuration,
    polkadot_config: Configuration,
    id: ParaId,
) -> sc_service::error::Result<(
    TaskManager,
    Arc<TFullClient<Block, RuntimeApi, NativeElseWasmExecutor<TemplateRuntimeExecutor>>>,
)> {
    start_node_impl::<RuntimeApi, TemplateRuntimeExecutor, _, _, _>(
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

                        let slot =
                            sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_duration(
                                *time,
                                slot_duration.slot_duration(),
                            );

                        let parachain_inherent = parachain_inherent.ok_or_else(|| {
                            Box::<dyn std::error::Error + Send + Sync>::from(
                                "Failed to create parachain inherent",
                            )
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