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

// pub mod template;
pub mod mars;
pub mod odyssey;
use codec::Codec;
use cumulus_client_consensus_aura::{
    // build_aura_consensus,
    BuildAuraConsensusParams,
    SlotProportion,
};
use cumulus_client_consensus_common::{
    ParachainBlockImport, ParachainCandidate, ParachainConsensus,
};
// use cumulus_client_network::build_block_announce_validator;
use cumulus_client_network::BlockAnnounceValidator;
use cumulus_client_service::{
    start_collator, start_full_node, StartCollatorParams, StartFullNodeParams,
};
use cumulus_relay_chain_interface::{RelayChainError, RelayChainInterface, RelayChainResult};
use cumulus_relay_chain_rpc_interface::{create_client_and_start_worker, RelayChainRpcInterface};
use cumulus_relay_chain_inprocess_interface::build_inprocess_relay_chain;
use polkadot_primitives::v2::CollatorPair;
use jsonrpsee::RpcModule;
use sp_core::Pair;
use sc_executor::WasmExecutor;

#[cfg(not(feature = "runtime-benchmarks"))]
type HostFunctions = sp_io::SubstrateHostFunctions;

#[cfg(feature = "runtime-benchmarks")]
type HostFunctions =
(sp_io::SubstrateHostFunctions, frame_benchmarking::benchmarking::HostFunctions);


// use cumulus_primitives_core::{
//     relay_chain::v1::{Hash as PHash, PersistedValidationData},
//     ParaId,
// };

use cumulus_primitives_core::{
    relay_chain::Hash as PHash, CollationInfo, CollectCollationInfo, ParachainBlockData,
    PersistedValidationData,
    ParaId,
};
use cumulus_client_consensus_relay_chain::Verifier as RelayChainVerifier;
use futures::lock::Mutex;
use sc_client_api::{Backend, ExecutorProvider};
use sc_consensus::{
    import_queue::{BasicQueue, Verifier as VerifierT},
    BlockImportParams,
};

// use sc_executor::native_executor_instance;
use sc_network::{NetworkBlock, NetworkService};
use sc_service::{Configuration, PartialComponents, Role, TFullBackend, TFullClient, TaskManager};
use sc_telemetry::{Telemetry, TelemetryHandle, TelemetryWorker, TelemetryWorkerHandle};
use sp_api::{ApiExt, ConstructRuntimeApi};
// use sp_consensus::{CacheKeyId, SlotData};
use sp_consensus::{CacheKeyId};
use sp_consensus_aura::{sr25519::AuthorityId as AuraId, AuraApi};
use sp_keystore::SyncCryptoStorePtr;
use sp_runtime::{
    app_crypto::AppKey,
    generic::BlockId,
    sp_std,
    traits::{BlakeTwo256, Header as HeaderT},
};
use std::sync::Arc;
use cumulus_client_cli::CollatorOptions;
use substrate_prometheus_endpoint::Registry;

pub type Block = sp_runtime::generic::Block<Header, sp_runtime::OpaqueExtrinsic>;
use frame_support::pallet_prelude::{Encode, PhantomData};
// pub use sc_executor::NativeExecutor;
use sp_core::offchain::{OffchainStorage, STORAGE_PREFIX};
use sc_executor::NativeElseWasmExecutor;
use polkadot_primitives::v2::{Nonce, Balance, AccountId};
use sp_runtime::sp_std::time::Duration;
use crate::rpc;
use core::convert::TryFrom;

type BlockNumber = u32;
type Header = sp_runtime::generic::Header<BlockNumber, sp_runtime::traits::BlakeTwo256>;
type Hash = sp_core::H256;

/// Build the import queue for Statemint and other Aura-based runtimes.
pub fn aura_build_import_queue<RuntimeApi, AuraId: AppKey>(
    client: Arc<TFullClient<Block, RuntimeApi, WasmExecutor<HostFunctions>>>,
    config: &Configuration,
    telemetry_handle: Option<TelemetryHandle>,
    task_manager: &TaskManager,
) -> Result<
    sc_consensus::DefaultImportQueue<
        Block,
        TFullClient<Block, RuntimeApi, WasmExecutor<HostFunctions>>,
    >,
    sc_service::Error,
>
    where
        RuntimeApi: ConstructRuntimeApi<Block, TFullClient<Block, RuntimeApi, WasmExecutor<HostFunctions>>>
        + Send
        + Sync
        + 'static,
        RuntimeApi::RuntimeApi: sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block>
        + sp_api::Metadata<Block>
        + sp_session::SessionKeys<Block>
        + sp_api::ApiExt<
            Block,
            StateBackend = sc_client_api::StateBackendFor<TFullBackend<Block>, Block>,
        > + sp_offchain::OffchainWorkerApi<Block>
        + sp_block_builder::BlockBuilder<Block>
        + sp_consensus_aura::AuraApi<Block, <<AuraId as AppKey>::Pair as Pair>::Public>,
        sc_client_api::StateBackendFor<TFullBackend<Block>, Block>: sp_api::StateBackend<BlakeTwo256>,
        <<AuraId as AppKey>::Pair as Pair>::Signature:
        TryFrom<Vec<u8>> + std::hash::Hash + sp_runtime::traits::Member + Codec,
{
    let client2 = client.clone();

    let aura_verifier = move || {
        let slot_duration = cumulus_client_consensus_aura::slot_duration(&*client2).unwrap();

        Box::new(
            cumulus_client_consensus_aura::build_verifier::<<AuraId as AppKey>::Pair, _, _, _>(
                cumulus_client_consensus_aura::BuildVerifierParams {
                    client: client2.clone(),
                    create_inherent_data_providers: move |_, _| async move {
                        let timestamp = sp_timestamp::InherentDataProvider::from_system_time();

                        let slot =
                            sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
                                *timestamp,
                                slot_duration,
                            );

                        Ok((timestamp, slot))
                    },
                    can_author_with: sp_consensus::AlwaysCanAuthor,
                    telemetry: telemetry_handle,
                },
            ),
        ) as Box<_>
    };

    let relay_chain_verifier =
        Box::new(RelayChainVerifier::new(client.clone(), |_, _| async { Ok(()) })) as Box<_>;

    let verifier = Verifier {
        client: client.clone(),
        relay_chain_verifier,
        aura_verifier: BuildOnAccess::Uninitialized(Some(Box::new(aura_verifier))),
        _phantom: PhantomData,
    };

    let registry = config.prometheus_registry();
    let spawner = task_manager.spawn_essential_handle();

    Ok(BasicQueue::new(
        verifier,
        Box::new(ParachainBlockImport::new(client)),
        None,
        &spawner,
        registry,
    ))
}

async fn build_relay_chain_interface(
    polkadot_config: Configuration,
    parachain_config: &Configuration,
    telemetry_worker_handle: Option<TelemetryWorkerHandle>,
    task_manager: &mut TaskManager,
    collator_options: CollatorOptions,
    hwbench: Option<sc_sysinfo::HwBench>,
) -> RelayChainResult<(Arc<(dyn RelayChainInterface + 'static)>, Option<CollatorPair>)> {
    match collator_options.relay_chain_rpc_url {
        Some(relay_chain_url) => {
            let client = create_client_and_start_worker(relay_chain_url, task_manager).await?;
            Ok((Arc::new(RelayChainRpcInterface::new(client)) as Arc<_>, None))
        },
        None => build_inprocess_relay_chain(
            polkadot_config,
            parachain_config,
            telemetry_worker_handle,
            task_manager,
            hwbench,
        ),
    }
}

/// Starts a `ServiceBuilder` for a full service.
///
/// Use this macro if you don't actually need the full service, but just the builder in order to
/// be able to perform chain operations.
pub fn new_partial<RuntimeApi, BIQ>(
    config: &Configuration,
    build_import_queue: BIQ,
) -> Result<
    PartialComponents<
        TFullClient<Block, RuntimeApi, WasmExecutor<HostFunctions>>,
        TFullBackend<Block>,
        (),
        sc_consensus::DefaultImportQueue<
            Block,
            TFullClient<Block, RuntimeApi, WasmExecutor<HostFunctions>>,
        >,
        sc_transaction_pool::FullPool<
            Block,
            TFullClient<Block, RuntimeApi, WasmExecutor<HostFunctions>>,
        >,
        (Option<Telemetry>, Option<TelemetryWorkerHandle>),
    >,
    sc_service::Error,
>
    where
        RuntimeApi: ConstructRuntimeApi<Block, TFullClient<Block, RuntimeApi, WasmExecutor<HostFunctions>>>
        + Send
        + Sync
        + 'static,
        RuntimeApi::RuntimeApi: sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block>
        + sp_api::Metadata<Block>
        + sp_session::SessionKeys<Block>
        + sp_api::ApiExt<
            Block,
            StateBackend = sc_client_api::StateBackendFor<TFullBackend<Block>, Block>,
        > + sp_offchain::OffchainWorkerApi<Block>
        + sp_block_builder::BlockBuilder<Block>,
        sc_client_api::StateBackendFor<TFullBackend<Block>, Block>: sp_api::StateBackend<BlakeTwo256>,
        BIQ: FnOnce(
            Arc<TFullClient<Block, RuntimeApi, WasmExecutor<HostFunctions>>>,
            &Configuration,
            Option<TelemetryHandle>,
            &TaskManager,
        ) -> Result<
            sc_consensus::DefaultImportQueue<
                Block,
                TFullClient<Block, RuntimeApi, WasmExecutor<HostFunctions>>,
            >,
            sc_service::Error,
        >,
{
    let telemetry = config
        .telemetry_endpoints
        .clone()
        .filter(|x| !x.is_empty())
        .map(|endpoints| -> Result<_, sc_telemetry::Error> {
            let worker = TelemetryWorker::new(16)?;
            let telemetry = worker.handle().new_telemetry(endpoints);
            Ok((worker, telemetry))
        })
        .transpose()?;

    let executor = sc_executor::WasmExecutor::<HostFunctions>::new(
        config.wasm_method,
        config.default_heap_pages,
        config.max_runtime_instances,
        None,
        config.runtime_cache_size,
    );

    let (client, backend, keystore_container, task_manager) =
        sc_service::new_full_parts::<Block, RuntimeApi, _>(
            config,
            telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
            executor,
        )?;
    let client = Arc::new(client);

    let telemetry_worker_handle = telemetry.as_ref().map(|(worker, _)| worker.handle());

    let telemetry = telemetry.map(|(worker, telemetry)| {
        task_manager.spawn_handle().spawn("telemetry", None, worker.run());
        telemetry
    });

    let transaction_pool = sc_transaction_pool::BasicPool::new_full(
        config.transaction_pool.clone(),
        config.role.is_authority().into(),
        config.prometheus_registry(),
        task_manager.spawn_essential_handle(),
        client.clone(),
    );

    let import_queue = build_import_queue(
        client.clone(),
        config,
        telemetry.as_ref().map(|telemetry| telemetry.handle()),
        &task_manager,
    )?;

    let params = PartialComponents {
        backend,
        client,
        import_queue,
        keystore_container,
        task_manager,
        transaction_pool,
        select_chain: (),
        other: (telemetry, telemetry_worker_handle),
    };
    Ok(params)
}

/// Prepare the parachain's node configuration
///
/// This function will disable the default announcement of Substrate for the parachain in favor
/// of the one of Cumulus.
pub fn prepare_node_config(mut parachain_config: Configuration) -> Configuration {
    parachain_config.announce_block = false;
    parachain_config
}

/// Start a node with the given parachain `Configuration` and relay chain `Configuration`.
///
/// This is the actual implementation that is abstract over the executor and the runtime api.
#[sc_tracing::logging::prefix_logs_with("Parachain")]
async fn start_node_impl<RuntimeApi, RB, BIQ, BIC>(
    parachain_config: Configuration,
    polkadot_config: Configuration,
    collator_options: CollatorOptions,
    id: ParaId,
    _rpc_ext_builder: RB,
    build_import_queue: BIQ,
    build_consensus: BIC,
    hwbench: Option<sc_sysinfo::HwBench>,
) -> sc_service::error::Result<(
    TaskManager,
    Arc<TFullClient<Block, RuntimeApi, WasmExecutor<HostFunctions>>>,
)>
    where
        RuntimeApi: ConstructRuntimeApi<Block, TFullClient<Block, RuntimeApi, WasmExecutor<HostFunctions>>>
        + Send
        + Sync
        + 'static,
        RuntimeApi::RuntimeApi: sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block>
        + sp_api::Metadata<Block>
        + sp_session::SessionKeys<Block>
        + sp_api::ApiExt<
            Block,
            StateBackend = sc_client_api::StateBackendFor<TFullBackend<Block>, Block>,
        > + sp_offchain::OffchainWorkerApi<Block>
        + sp_block_builder::BlockBuilder<Block>
        + cumulus_primitives_core::CollectCollationInfo<Block>
        + pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>
        + frame_rpc_system::AccountNonceApi<Block, AccountId, Nonce>,
        sc_client_api::StateBackendFor<TFullBackend<Block>, Block>: sp_api::StateBackend<BlakeTwo256>,
        RB: Fn(
            Arc<TFullClient<Block, RuntimeApi, WasmExecutor<HostFunctions>>>,
        ) -> Result<jsonrpsee::RpcModule<()>, sc_service::Error>
        + Send
        + 'static,
        BIQ: FnOnce(
            Arc<TFullClient<Block, RuntimeApi, WasmExecutor<HostFunctions>>>,
            &Configuration,
            Option<TelemetryHandle>,
            &TaskManager,
        ) -> Result<
            sc_consensus::DefaultImportQueue<
                Block,
                TFullClient<Block, RuntimeApi, WasmExecutor<HostFunctions>>,
            >,
            sc_service::Error,
        > + 'static,
        BIC: FnOnce(
            Arc<TFullClient<Block, RuntimeApi, WasmExecutor<HostFunctions>>>,
            Arc<TFullBackend<Block>>,
            Option<&Registry>,
            Option<TelemetryHandle>,
            &TaskManager,
            Arc<dyn RelayChainInterface>,
            Arc<
                sc_transaction_pool::FullPool<
                    Block,
                    TFullClient<Block, RuntimeApi, WasmExecutor<HostFunctions>>,
                >,
            >,
            Arc<NetworkService<Block, Hash>>,
            SyncCryptoStorePtr,
            bool,
        ) -> Result<Box<dyn ParachainConsensus<Block>>, sc_service::Error>,
{
    let parachain_config = prepare_node_config(parachain_config);

    let params = new_partial::<RuntimeApi, BIQ>(&parachain_config, build_import_queue)?;
    let (mut telemetry, telemetry_worker_handle) = params.other;

    let client = params.client.clone();
    let backend = params.backend.clone();

    let mut task_manager = params.task_manager;
    let (relay_chain_interface, collator_key) = build_relay_chain_interface(
        polkadot_config,
        &parachain_config,
        telemetry_worker_handle,
        &mut task_manager,
        collator_options.clone(),
        hwbench.clone(),
    )
        .await
        .map_err(|e| match e {
            RelayChainError::ServiceError(polkadot_service::Error::Sub(x)) => x,
            s => s.to_string().into(),
        })?;

    let block_announce_validator = BlockAnnounceValidator::new(relay_chain_interface.clone(), id);

    let force_authoring = parachain_config.force_authoring;
    let validator = parachain_config.role.is_authority();
    let prometheus_registry = parachain_config.prometheus_registry().cloned();
    let transaction_pool = params.transaction_pool.clone();
    let import_queue = cumulus_client_service::SharedImportQueue::new(params.import_queue);
    let (network, system_rpc_tx, start_network) =
        sc_service::build_network(sc_service::BuildNetworkParams {
            config: &parachain_config,
            client: client.clone(),
            transaction_pool: transaction_pool.clone(),
            spawn_handle: task_manager.spawn_handle(),
            import_queue: import_queue.clone(),
            block_announce_validator_builder: Some(Box::new(|_| {
                Box::new(block_announce_validator)
            })),
            warp_sync: None,
        })?;

    if parachain_config.offchain_worker.enabled {
        sc_service::build_offchain_workers(
            &parachain_config,
            task_manager.spawn_handle(),
            client.clone(),
            network.clone(),
        );
    }

    let rpc_builder = {
        let client = client.clone();
        let transaction_pool = transaction_pool.clone();

        Box::new(move |deny_unsafe, _| {
            let deps = rpc::FullDeps {
                client: client.clone(),
                pool: transaction_pool.clone(),
                deny_unsafe,
            };

            rpc::create_full(deps).map_err(Into::into)
        })
    };

    sc_service::spawn_tasks(sc_service::SpawnTasksParams {
        rpc_builder,
        client: client.clone(),
        transaction_pool: transaction_pool.clone(),
        task_manager: &mut task_manager,
        config: parachain_config,
        keystore: params.keystore_container.sync_keystore(),
        backend: backend.clone(),
        network: network.clone(),
        system_rpc_tx,
        telemetry: telemetry.as_mut(),
    })?;

    if let Some(hwbench) = hwbench {
        sc_sysinfo::print_hwbench(&hwbench);

        if let Some(ref mut telemetry) = telemetry {
            let telemetry_handle = telemetry.handle();
            task_manager.spawn_handle().spawn(
                "telemetry_hwbench",
                None,
                sc_sysinfo::initialize_hwbench_telemetry(telemetry_handle, hwbench),
            );
        }
    }

    let announce_block = {
        let network = network.clone();
        Arc::new(move |hash, data| network.announce_block(hash, data))
    };

    let relay_chain_slot_duration = Duration::from_secs(6);

    if validator {
        let parachain_consensus = build_consensus(
            client.clone(),
            backend.clone(),
            prometheus_registry.as_ref(),
            telemetry.as_ref().map(|t| t.handle()),
            &task_manager,
            relay_chain_interface.clone(),
            transaction_pool,
            network,
            params.keystore_container.sync_keystore(),
            force_authoring,
        )?;

        let spawner = task_manager.spawn_handle();

        let params = StartCollatorParams {
            para_id: id,
            block_status: client.clone(),
            announce_block,
            client: client.clone(),
            task_manager: &mut task_manager,
            relay_chain_interface: relay_chain_interface.clone(),
            spawner,
            parachain_consensus,
            import_queue,
            collator_key: collator_key.expect("Command line arguments do not allow this. qed"),
            relay_chain_slot_duration,
        };

        start_collator(params).await?;
    } else {
        let params = StartFullNodeParams {
            client: client.clone(),
            announce_block,
            task_manager: &mut task_manager,
            para_id: id,
            relay_chain_interface,
            relay_chain_slot_duration,
            import_queue,
            collator_options,
        };

        start_full_node(params)?;
    }

    start_network.start_network();

    Ok((task_manager, client))
}


enum BuildOnAccess<R> {
    Uninitialized(Option<Box<dyn FnOnce() -> R + Send + Sync>>),
    Initialized(R),
}

impl<R> BuildOnAccess<R> {
    fn get_mut(&mut self) -> &mut R {
        loop {
            match self {
                Self::Uninitialized(f) => {
                    *self = Self::Initialized((f.take().unwrap())());
                }
                Self::Initialized(ref mut r) => return r,
            }
        }
    }
}

/// Special [`ParachainConsensus`] implementation that waits for the upgrade from
/// shell to a parachain runtime that implements Aura.
struct WaitForAuraConsensus<Client, AuraId> {
    client: Arc<Client>,
    aura_consensus: Arc<Mutex<BuildOnAccess<Box<dyn ParachainConsensus<Block>>>>>,
    relay_chain_consensus: Arc<Mutex<Box<dyn ParachainConsensus<Block>>>>,
    _phantom: PhantomData<AuraId>,
}

impl<Client, AuraId> Clone for WaitForAuraConsensus<Client, AuraId> {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            aura_consensus: self.aura_consensus.clone(),
            relay_chain_consensus: self.relay_chain_consensus.clone(),
            _phantom: PhantomData,
        }
    }
}


#[async_trait::async_trait]
impl<Client, AuraId> ParachainConsensus<Block> for WaitForAuraConsensus<Client, AuraId>
    where
        Client: sp_api::ProvideRuntimeApi<Block> + Send + Sync,
        Client::Api: AuraApi<Block, AuraId>,
        AuraId: Send + Codec + Sync,
{
    async fn produce_candidate(
        &mut self,
        parent: &Header,
        relay_parent: PHash,
        validation_data: &PersistedValidationData,
    ) -> Option<ParachainCandidate<Block>> {
        let block_id = BlockId::hash(parent.hash());
        if self
            .client
            .runtime_api()
            .has_api::<dyn AuraApi<Block, AuraId>>(&block_id)
            .unwrap_or(false)
        {
            self.aura_consensus
                .lock()
                .await
                .get_mut()
                .produce_candidate(parent, relay_parent, validation_data)
                .await
        } else {
            self.relay_chain_consensus
                .lock()
                .await
                .produce_candidate(parent, relay_parent, validation_data)
                .await
        }
    }
}

// struct Verifier<Client> {
//     client: Arc<Client>,
//     aura_verifier: BuildOnAccess<Box<dyn VerifierT<Block>>>,
//     relay_chain_verifier: Box<dyn VerifierT<Block>>,
// }
struct Verifier<Client, AuraId> {
    client: Arc<Client>,
    aura_verifier: BuildOnAccess<Box<dyn VerifierT<Block>>>,
    relay_chain_verifier: Box<dyn VerifierT<Block>>,
    _phantom: PhantomData<AuraId>,
}

#[async_trait::async_trait]
impl<Client, AuraId> VerifierT<Block> for Verifier<Client, AuraId>
    where
        Client: sp_api::ProvideRuntimeApi<Block> + Send + Sync,
        Client::Api: AuraApi<Block, AuraId>,
        AuraId: Send + Sync + Codec,
{
    async fn verify(
        &mut self,
        block_import: BlockImportParams<Block, ()>,
    ) -> Result<(BlockImportParams<Block, ()>, Option<Vec<(CacheKeyId, Vec<u8>)>>), String> {
        let block_id = BlockId::hash(*block_import.header.parent_hash());

        if self
            .client
            .runtime_api()
            .has_api::<dyn AuraApi<Block, AuraId>>(&block_id)
            .unwrap_or(false)
        {
            self.aura_verifier.get_mut().verify(block_import).await
        } else {
            self.relay_chain_verifier.verify(block_import).await
        }
    }
}