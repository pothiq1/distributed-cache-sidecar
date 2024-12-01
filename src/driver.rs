//driver.rs

use crate::proto::cache_service_client::CacheServiceClient;
use crate::proto::*;
use bytes::Bytes;
use std::collections::HashMap;
use tonic::transport::Channel;
use tonic::Request;

pub struct CacheDriver {
    client: CacheServiceClient<Channel>,
}

impl CacheDriver {
    pub async fn new(sidecar_address: String) -> Self {
        let client = CacheServiceClient::connect(sidecar_address).await.unwrap();
        Self { client }
    }

    pub async fn get(&self, key: String) -> Option<Bytes> {
        let request = Request::new(CacheKey { key });
        if let Ok(response) = self.client.clone().get(request).await {
            let cache_value = response.into_inner();
            if cache_value.found {
                return Some(Bytes::from(cache_value.value));
            }
        }
        None
    }

    pub async fn put(&self, key: String, value: Bytes, ttl: Option<i64>) -> bool {
        let entry = CacheEntry {
            key,
            value: value.to_vec(),
            ttl: ttl.unwrap_or(0),
        };
        let request = Request::new(entry);
        if let Ok(response) = self.client.clone().put(request).await {
            return response.into_inner().success;
        }
        false
    }

    pub async fn evict(&self, key: String) -> bool {
        let request = Request::new(CacheKey { key });
        if let Ok(response) = self.client.clone().evict(request).await {
            return response.into_inner().success;
        }
        false
    }

    pub async fn refresh(&self, key: String) -> Option<Bytes> {
        let request = Request::new(CacheKey { key });
        if let Ok(response) = self.client.clone().refresh(request).await {
            let cache_value = response.into_inner();
            if cache_value.found {
                return Some(Bytes::from(cache_value.value));
            }
        }
        None
    }

    pub async fn batch_get(&self, keys: Vec<String>) -> HashMap<String, Option<Bytes>> {
        let request = Request::new(BatchKeys { keys });
        let mut result = HashMap::new();

        if let Ok(response) = self.client.clone().batch_get(request).await {
            let batch_values = response.into_inner().values;
            for (key, cache_value) in batch_values {
                if cache_value.found {
                    result.insert(key, Some(Bytes::from(cache_value.value)));
                } else {
                    result.insert(key, None);
                }
            }
        }

        result
    }

    pub async fn batch_put(&self, entries: Vec<(String, Bytes, Option<i64>)>) -> bool {
        let cache_entries = entries
            .into_iter()
            .map(|(key, value, ttl)| CacheEntry {
                key,
                value: value.to_vec(),
                ttl: ttl.unwrap_or(0),
            })
            .collect();

        let request = Request::new(BatchEntries {
            entries: cache_entries,
        });

        if let Ok(response) = self.client.clone().batch_put(request).await {
            return response.into_inner().success;
        }
        false
    }

    pub async fn query(&self, query_str: String) -> HashMap<String, Bytes> {
        let request = Request::new(QueryRequest { query: query_str });
        let mut result = HashMap::new();

        if let Ok(response) = self.client.clone().query(request).await {
            let query_response = response.into_inner().results;
            for (key, cache_value) in query_response {
                if cache_value.found {
                    result.insert(key, Bytes::from(cache_value.value));
                }
            }
        }

        result
    }

    pub async fn begin_transaction(&self) -> Option<String> {
        let request = Request::new(TransactionRequest {
            transaction_id: String::new(),
        });
        if let Ok(response) = self.client.clone().begin_transaction(request).await {
            let transaction_response = response.into_inner();
            if transaction_response.success {
                return Some(transaction_response.message);
            }
        }
        None
    }

    pub async fn commit_transaction(&self, transaction_id: String) -> bool {
        let request = Request::new(TransactionRequest { transaction_id });
        if let Ok(response) = self.client.clone().commit_transaction(request).await {
            return response.into_inner().success;
        }
        false
    }

    pub async fn rollback_transaction(&self, transaction_id: String) -> bool {
        let request = Request::new(TransactionRequest { transaction_id });
        if let Ok(response) = self.client.clone().rollback_transaction(request).await {
            return response.into_inner().success;
        }
        false
    }

    pub async fn listen_events(
        &self,
        event_types: Vec<EventType>,
    ) -> Result<tonic::Streaming<EventResponse>, tonic::Status> {
        let request = Request::new(EventRequest { event_types });
        let response = self.client.clone().listen_events(request).await?;
        Ok(response.into_inner())
    }
}
