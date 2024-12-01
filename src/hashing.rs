// src/hashing.rs

use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};
use std::sync::RwLock;

use std::collections::hash_map::DefaultHasher;

pub struct ConsistentHashing {
    ring: RwLock<BTreeMap<u64, String>>,
    replicas: usize,
}

impl ConsistentHashing {
    pub fn new(replicas: usize) -> Self {
        Self {
            ring: RwLock::new(BTreeMap::new()),
            replicas,
        }
    }

    pub fn add_node(&self, node: String) {
        let mut ring = self.ring.write().unwrap();
        for i in 0..self.replicas {
            let virtual_node = format!("{}-{}", node, i);
            let hash = self.hash(&virtual_node);
            ring.insert(hash, node.clone());
        }
    }

    pub fn remove_node(&self, node: &str) {
        let mut ring = self.ring.write().unwrap();
        for i in 0..self.replicas {
            let virtual_node = format!("{}-{}", node, i);
            let hash = self.hash(&virtual_node);
            ring.remove(&hash);
        }
    }

    pub fn get_node(&self, key: &str) -> Option<String> {
        let hash = self.hash(key);
        let ring = self.ring.read().unwrap();
        ring.range(hash..)
            .next()
            .or_else(|| ring.iter().next())
            .map(|(_, node)| node.clone())
    }

    pub fn get_n_nodes(&self, key: &str, n: usize) -> Vec<String> {
        let hash = self.hash(key);
        let ring = self.ring.read().unwrap();
        let mut nodes = Vec::new();
        let mut iter = ring.range(hash..).chain(ring.iter());

        for (_, node) in iter {
            if !nodes.contains(node) {
                nodes.push(node.clone());
            }
            if nodes.len() >= n {
                break;
            }
        }

        nodes
    }

    fn hash<T: Hash + ?Sized>(&self, key: &T) -> u64 {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        hasher.finish()
    }

    pub fn get_all_nodes(&self) -> Vec<String> {
        let ring = self.ring.read().unwrap();
        let mut nodes = ring.values().cloned().collect::<Vec<_>>();
        nodes.sort();
        nodes.dedup();
        nodes
    }
}
