// src/main.rs

mod cache;
mod config;
mod event_listener;
mod fallback;
mod hashing;
mod monitoring;
mod pod_discovery;
mod replication;
mod search_index;
mod security;
mod transaction_manager;

mod proto {
    tonic::include_proto!("cache");
}

use crate::cache::Cache;
use crate::config::Config;
use crate::event_listener::{EventListener, EventType as ListenerEventType};
use crate::fallback::{Fallback, RedisFallback};
use crate::hashing::ConsistentHashing;
use crate::monitoring::Monitoring;
use crate::pod_discovery::PodDiscovery;
use crate::proto::cache_service_client::CacheServiceClient;
use crate::proto::cache_service_server::{CacheService, CacheServiceServer};
use crate::proto::*;
use crate::replication::Replicator;
use crate::search_index::SearchIndex;
use crate::security::Security;
use bytes::Bytes;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tonic::{transport::Server, Request, Response, Status};
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let config = Config::load();

    let event_listener = Arc::new(EventListener::new());
    let event_sender = event_listener.get_sender();

    let cache = Arc::new(Cache::new(config.clone(), event_sender.clone()));

    // Start event listener
    {
        let event_listener_clone = event_listener.clone();
        tokio::spawn(async move {
            let receiver = event_listener_clone.listen();
            while let Ok(event) = receiver.recv() {
                info!("Event received: {:?}", event);
            }
        });
    }

    let hasher = Arc::new(ConsistentHashing::new(100));
    hasher.add_node("localhost".to_string());
    let replicator = Arc::new(Replicator::new(
        hasher.clone(),
        config.replication_factor,
        "localhost".to_string(),
    ));
    let fallback = Arc::new(RedisFallback::new(&config.redis_url).await);
    let search_index = Arc::new(SearchIndex::new());
    let security = Security::new(&config);

    if config.enable_monitoring {
        let monitoring = Monitoring::new();
        monitoring.serve(cache.clone(), hasher.clone(), Arc::new(config.clone()));
    }

    let pod_discovery = PodDiscovery::new(hasher.clone());
    tokio::spawn(async move { pod_discovery.start().await });

    let addr = config.local_address.parse()?;
    let cache_service = MyCacheService {
        cache: cache.clone(),
        replicator,
        hasher: hasher.clone(),
        fallback,
        search_index,
        security,
        config: config.clone(),
        event_listener: event_listener.clone(),
    };

    let mut server_builder = Server::builder();

    if let Some(tls_config) = &cache_service.security.tls_config {
        server_builder = server_builder.tls_config(tls_config.clone())?;
    }

    info!("Starting CacheService on {}", addr);

    server_builder
        .add_service(CacheServiceServer::new(cache_service))
        .serve(addr)
        .await?;

    Ok(())
}

struct MyCacheService {
    cache: Arc<Cache>,
    replicator: Arc<Replicator>,
    hasher: Arc<ConsistentHashing>,
    fallback: Arc<dyn Fallback + Send + Sync>,
    search_index: Arc<SearchIndex>,
    security: Security,
    config: Config,
    event_listener: Arc<EventListener>,
}

#[tonic::async_trait]
impl CacheService for MyCacheService {
    type ListenEventsStream = tokio_stream::wrappers::ReceiverStream<Result<EventResponse, Status>>;

    async fn get(&self, request: Request<CacheKey>) -> Result<Response<CacheValue>, Status> {
        self.security.authenticate(&request)?;

        let key = request.into_inner().key;

        if let Some(value) = self.cache.get(&key) {
            info!("Cache hit for key: {}", key);
            return Ok(Response::new(CacheValue {
                value: value.to_vec(),
                found: true,
            }));
        }

        let nodes = self
            .hasher
            .get_n_nodes(&key, self.replicator.replication_factor + 1);

        for node in nodes {
            if node == "localhost" {
                continue;
            }
            let addr = format!("http://{}:50051", node);
            let mut client = match CacheServiceClient::connect(addr.clone()).await {
                Ok(client) => client,
                Err(e) => {
                    warn!("Failed to connect to node {}: {}", node, e);
                    continue;
                }
            };
            let request = tonic::Request::new(CacheKey { key: key.clone() });
            match client.get(request).await {
                Ok(response) => {
                    let cache_value = response.into_inner();
                    if cache_value.found {
                        info!("Cache hit from node {} for key: {}", node, key);
                        self.cache
                            .put(key.clone(), Bytes::from(cache_value.value.clone()), None);
                        return Ok(Response::new(CacheValue {
                            value: cache_value.value,
                            found: true,
                        }));
                    }
                }
                Err(e) => {
                    warn!("Failed to get key {} from node {}: {}", key, node, e);
                    continue;
                }
            }
        }

        if let Some(value) = self.fallback.get(&key).await {
            info!("Cache miss for key: {}. Fetched from fallback.", key);
            self.cache.put(key.clone(), value.clone(), None);
            let _ = self
                .replicator
                .replicate(
                    key.clone(),
                    value.clone(),
                    self.cache.default_ttl.map_or(0, |d| d.as_secs() as i64),
                )
                .await;
            return Ok(Response::new(CacheValue {
                value: value.to_vec(),
                found: true,
            }));
        } else {
            info!("Cache miss for key: {}. No data found in fallback.", key);
            return Ok(Response::new(CacheValue {
                value: vec![],
                found: false,
            }));
        }
    }

    async fn put(&self, request: Request<CacheEntry>) -> Result<Response<PutResponse>, Status> {
        self.security.authenticate(&request)?;

        let entry = request.into_inner();
        let ttl = if entry.ttl > 0 {
            Some(Duration::from_secs(entry.ttl as u64))
        } else {
            None
        };
        let value = Bytes::from(entry.value.clone());

        self.cache.put(entry.key.clone(), value.clone(), ttl);

        if let Ok(value_str) = String::from_utf8(entry.value.clone()) {
            self.search_index.add_document(&entry.key, &value_str);
        }

        let _ = self
            .replicator
            .replicate(entry.key.clone(), value.clone(), entry.ttl)
            .await;

        info!("Stored key: {} in cache and replicated.", entry.key);

        Ok(Response::new(PutResponse { success: true }))
    }

    async fn begin_transaction(
        &self,
        request: Request<TransactionRequest>,
    ) -> Result<Response<TransactionResponse>, Status> {
        self.security.authenticate(&request)?;

        if !self.config.enable_transactions {
            warn!("Attempted to begin transaction, but transactions are disabled");
            return Err(Status::unimplemented("Transactions are disabled"));
        }

        let transaction_id = self.cache.transaction_manager.begin_transaction();

        info!("Started transaction with ID: {}", transaction_id);

        Ok(Response::new(TransactionResponse {
            success: true,
            message: transaction_id,
        }))
    }

    async fn commit_transaction(
        &self,
        request: Request<TransactionRequest>,
    ) -> Result<Response<TransactionResponse>, Status> {
        self.security.authenticate(&request)?;

        let transaction_id = request.into_inner().transaction_id;

        self.cache.commit_transaction(&transaction_id);

        info!("Committed transaction ID: {}", transaction_id);

        Ok(Response::new(TransactionResponse {
            success: true,
            message: "Transaction committed".to_string(),
        }))
    }

    async fn rollback_transaction(
        &self,
        request: Request<TransactionRequest>,
    ) -> Result<Response<TransactionResponse>, Status> {
        self.security.authenticate(&request)?;

        let transaction_id = request.into_inner().transaction_id;

        self.cache.rollback_transaction(&transaction_id);

        info!("Rolled back transaction ID: {}", transaction_id);

        Ok(Response::new(TransactionResponse {
            success: true,
            message: "Transaction rolled back".to_string(),
        }))
    }

    async fn listen_events(
        &self,
        request: Request<EventRequest>,
    ) -> Result<Response<Self::ListenEventsStream>, Status> {
        self.security.authenticate(&request)?;

        let _event_types = request.into_inner().event_types;

        let (tx, rx) = mpsc::channel(100);

        let receiver = self.event_listener.listen();

        let event_sender = tx.clone();
        tokio::spawn(async move {
            while let Ok(event) = receiver.recv() {
                let event_type = match event.event_type {
                    ListenerEventType::Put => EventType::Put as i32,
                    ListenerEventType::Evict => EventType::Evict as i32,
                    ListenerEventType::Expire => EventType::Expire as i32,
                };

                let cache_entry = Some(CacheEntry {
                    key: event.key.clone(),
                    value: vec![],
                    ttl: 0,
                });

                let event_response = EventResponse {
                    event_type,
                    entry: cache_entry,
                };

                if event_sender.send(Ok(event_response)).await.is_err() {
                    break;
                }
            }
        });

        Ok(Response::new(tokio_stream::wrappers::ReceiverStream::new(
            rx,
        )))
    }

    async fn batch_get(
        &self,
        request: Request<BatchKeys>,
    ) -> Result<Response<BatchValues>, Status> {
        self.security.authenticate(&request)?;

        let keys = request.into_inner().keys;
        let mut values = std::collections::HashMap::new();

        for key in keys {
            if let Some(value) = self.cache.get(&key) {
                values.insert(
                    key.clone(),
                    CacheValue {
                        value: value.to_vec(),
                        found: true,
                    },
                );
            } else {
                values.insert(
                    key.clone(),
                    CacheValue {
                        value: vec![],
                        found: false,
                    },
                );
            }
        }

        Ok(Response::new(BatchValues { values }))
    }

    async fn batch_put(
        &self,
        request: Request<BatchEntries>,
    ) -> Result<Response<BatchPutResponse>, Status> {
        self.security.authenticate(&request)?;

        let entries = request.into_inner().entries;
        for entry in entries {
            let ttl = if entry.ttl > 0 {
                Some(Duration::from_secs(entry.ttl as u64))
            } else {
                None
            };
            let value = Bytes::from(entry.value.clone());

            self.cache.put(entry.key.clone(), value.clone(), ttl);

            if let Ok(value_str) = String::from_utf8(entry.value.clone()) {
                self.search_index.add_document(&entry.key, &value_str);
            }

            let _ = self
                .replicator
                .replicate(entry.key.clone(), value.clone(), entry.ttl)
                .await;

            info!("Stored key: {} in cache and replicated.", entry.key);
        }

        Ok(Response::new(BatchPutResponse { success: true }))
    }

    async fn evict(&self, request: Request<CacheKey>) -> Result<Response<EvictResponse>, Status> {
        self.security.authenticate(&request)?;

        let key = request.into_inner().key;

        self.cache.evict(&key);

        info!("Evicted key: {} from cache.", key);

        Ok(Response::new(EvictResponse { success: true }))
    }

    async fn refresh(&self, request: Request<CacheKey>) -> Result<Response<CacheValue>, Status> {
        self.security.authenticate(&request)?;

        let key = request.into_inner().key;

        if let Some(value) = self.fallback.get(&key).await {
            self.cache.put(key.clone(), value.clone(), None);
            let _ = self
                .replicator
                .replicate(
                    key.clone(),
                    value.clone(),
                    self.cache.default_ttl.map_or(0, |d| d.as_secs() as i64),
                )
                .await;
            info!("Refreshed key: {} from fallback.", key);
            return Ok(Response::new(CacheValue {
                value: value.to_vec(),
                found: true,
            }));
        } else {
            info!("Failed to refresh key: {}. No data found in fallback.", key);
            return Ok(Response::new(CacheValue {
                value: vec![],
                found: false,
            }));
        }
    }
}
