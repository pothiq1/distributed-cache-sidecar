//replication.rs

use bytes::Bytes;
use std::sync::Arc;

use crate::hashing::ConsistentHashing;
use crate::proto::cache_service_client::CacheServiceClient;
use crate::proto::CacheEntry;
use tonic::transport::Channel;
use tracing::error;

pub struct Replicator {
    hasher: Arc<ConsistentHashing>,
    pub replication_factor: usize,
    local_node_address: String,
}

impl Replicator {
    pub fn new(
        hasher: Arc<ConsistentHashing>,
        replication_factor: usize,
        local_node_address: String,
    ) -> Self {
        Self {
            hasher,
            replication_factor,
            local_node_address,
        }
    }

    pub async fn replicate(&self, key: String, value: Bytes, ttl: i64) {
        let nodes = self.hasher.get_n_nodes(&key, self.replication_factor + 1); // +1 to include local node

        for node in nodes {
            if node == self.local_node_address {
                continue; // Skip local node
            }
            let addr = format!("http://{}:50051", node);
            let mut client = match CacheServiceClient::connect(addr.clone()).await {
                Ok(client) => client,
                Err(e) => {
                    error!("Failed to connect to node {}: {}", addr, e);
                    continue;
                }
            };
            let entry = CacheEntry {
                key: key.clone(),
                value: value.clone().to_vec(),
                ttl,
            };
            if let Err(e) = client.put(tonic::Request::new(entry)).await {
                error!("Failed to replicate to node {}: {}", addr, e);
            }
        }
    }
}
