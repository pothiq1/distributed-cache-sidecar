use crate::event_listener::{CacheEvent, EventType};
use bytes::Bytes;
use crossbeam::channel::Sender;
use dashmap::DashMap;
use lz4::block::{compress, decompress};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

pub struct CacheEntry {
    pub value: Bytes,
    pub expires_at: Option<Instant>,
    pub frequency: u64,
}

pub struct Cache {
    max_memory: usize,
    pub current_memory: AtomicUsize,
    pub data: DashMap<String, CacheEntry>,
    pub default_ttl: Option<Duration>,
    frequency_threshold: u64,
    event_sender: Sender<CacheEvent>,
    pub transaction_manager: Arc<crate::transaction_manager::TransactionManager>,
}

impl Cache {
    pub fn new(config: crate::config::Config, event_sender: Sender<CacheEvent>) -> Self {
        let transaction_manager = if config.enable_transactions {
            Arc::new(crate::transaction_manager::TransactionManager::new(
                Duration::from_secs(config.transaction_timeout),
            ))
        } else {
            Arc::new(crate::transaction_manager::TransactionManager::disabled())
        };

        Self {
            max_memory: config.max_memory,
            current_memory: AtomicUsize::new(0),
            data: DashMap::new(),
            default_ttl: Some(Duration::from_secs(config.default_ttl)),
            frequency_threshold: config.frequency_threshold,
            event_sender,
            transaction_manager,
        }
    }

    /// Returns the current memory usage of the cache
    pub fn get_current_memory(&self) -> usize {
        self.current_memory
            .load(std::sync::atomic::Ordering::SeqCst)
    }

    /// Rollbacks the transaction with the given transaction ID.
    pub fn rollback_transaction(&self, transaction_id: &str) {
        if let Some(operations) = self.transaction_manager.rollback(transaction_id) {
            for operation in operations {
                match operation {
                    crate::transaction_manager::Operation::Put { key, .. } => {
                        // Remove the key if it was added during the transaction
                        self.evict(&key);
                    }
                    crate::transaction_manager::Operation::Evict { key, value, ttl } => {
                        // Restore the key if it was evicted during the transaction
                        self.put(key, value, ttl);
                    }
                }
            }
        } else {
            tracing::warn!("Transaction ID {} not found for rollback.", transaction_id);
        }
    }

    pub fn commit_transaction(&self, transaction_id: &str) {
        if let Some(operations) = self.transaction_manager.commit(transaction_id) {
            for operation in operations {
                match operation {
                    crate::transaction_manager::Operation::Put { key, value, ttl } => {
                        self.put(key, value, ttl);
                    }
                    crate::transaction_manager::Operation::Evict { key, .. } => {
                        self.evict(&key);
                    }
                    _ => {
                        // Ignore other fields or log unexpected cases
                        tracing::warn!("Unhandled transaction operation: {:?}", operation);
                    }
                }
            }
        }
    }

    pub fn get(&self, key: &str) -> Option<Bytes> {
        if let Some(mut entry) = self.data.get_mut(key) {
            if let Some(expires_at) = entry.expires_at {
                if Instant::now() > expires_at {
                    let size = entry.value.len();
                    self.data.remove(key);
                    self.current_memory.fetch_sub(size, Ordering::SeqCst);
                    let _ = self.event_sender.send(CacheEvent {
                        event_type: EventType::Expire,
                        key: key.to_string(),
                    });
                    return None;
                }
            }
            entry.frequency += 1;
            decompress(&entry.value, None).ok().map(Bytes::from)
        } else {
            None
        }
    }

    pub fn put(&self, key: String, value: Bytes, ttl: Option<Duration>) {
        let expires_at = ttl.map(|t| Instant::now() + t);
        let compressed_value = compress(&value, None, false).unwrap();
        let size = compressed_value.len();

        while self.current_memory.load(Ordering::SeqCst) + size > self.max_memory {
            if let Some(item) = self.data.iter().min_by_key(|entry| entry.value().frequency) {
                let key = item.key().clone();
                let entry = self.data.remove(&key).unwrap().1;
                self.current_memory
                    .fetch_sub(entry.value.len(), Ordering::SeqCst);
                let _ = self.event_sender.send(CacheEvent {
                    event_type: EventType::Evict,
                    key,
                });
            } else {
                break;
            }
        }

        self.data.insert(
            key.clone(),
            CacheEntry {
                value: Bytes::from(compressed_value),
                expires_at,
                frequency: 1,
            },
        );
        self.current_memory.fetch_add(size, Ordering::SeqCst);
        let _ = self.event_sender.send(CacheEvent {
            event_type: EventType::Put,
            key,
        });
    }

    pub fn evict(&self, key: &str) {
        if let Some(entry) = self.data.remove(key) {
            self.current_memory
                .fetch_sub(entry.1.value.len(), Ordering::SeqCst);
            let _ = self.event_sender.send(CacheEvent {
                event_type: EventType::Evict,
                key: key.to_string(),
            });
        }
    }
}
