//transaction_manager.rs

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use bytes::Bytes;
use uuid::Uuid;

#[derive(Debug)]
pub enum Operation {
    Put {
        key: String,
        value: Bytes,
        ttl: Option<Duration>,
    },
    Evict {
        key: String,
        value: Bytes,
        ttl: Option<Duration>,
    }, // Include `value` and `ttl`
}

pub struct Transaction {
    pub id: String,
    pub operations: Vec<Operation>,
    pub expires_at: Instant,
}

pub struct TransactionManager {
    transactions: Mutex<HashMap<String, Transaction>>,
    timeout: Duration,
    enabled: bool,
}

impl TransactionManager {
    pub fn new(timeout: Duration) -> Self {
        Self {
            transactions: Mutex::new(HashMap::new()),
            timeout,
            enabled: true,
        }
    }

    pub fn disabled() -> Self {
        Self {
            transactions: Mutex::new(HashMap::new()),
            timeout: Duration::from_secs(0),
            enabled: false,
        }
    }

    pub fn begin_transaction(&self) -> String {
        if !self.enabled {
            return "".to_string();
        }
        let id = Uuid::new_v4().to_string();
        let transaction = Transaction {
            id: id.clone(),
            operations: Vec::new(),
            expires_at: Instant::now() + self.timeout,
        };
        self.transactions
            .lock()
            .unwrap()
            .insert(id.clone(), transaction);
        id
    }

    pub fn add_operation(&self, transaction_id: &str, operation: Operation) {
        if !self.enabled {
            return;
        }
        if let Some(transaction) = self.transactions.lock().unwrap().get_mut(transaction_id) {
            transaction.operations.push(operation);
        }
    }

    pub fn commit(&self, transaction_id: &str) -> Option<Vec<Operation>> {
        if !self.enabled {
            return None;
        }
        self.transactions
            .lock()
            .unwrap()
            .remove(transaction_id)
            .map(|t| t.operations)
    }

    pub fn rollback(&self, transaction_id: &str) -> Option<Vec<Operation>> {
        self.transactions
            .lock()
            .unwrap()
            .remove(transaction_id)
            .map(|transaction| transaction.operations) // Extract operations from the transaction
    }

    pub fn cleanup_expired(&self) {
        let now = Instant::now();
        self.transactions
            .lock()
            .unwrap()
            .retain(|_, t| t.expires_at > now);
    }
}
