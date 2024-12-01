// src/pod_discovery.rs

use futures_util::StreamExt;
use kube::api::{WatchEvent, WatchParams};
use kube::{Api, Client};
use std::sync::Arc;
use tracing::{error, info};

use crate::hashing::ConsistentHashing;

pub struct PodDiscovery {
    namespace: String,
    app_label: String,
    hasher: Arc<ConsistentHashing>,
}

impl PodDiscovery {
    pub fn new(hasher: Arc<ConsistentHashing>) -> Self {
        let namespace = std::env::var("NAMESPACE").unwrap_or_else(|_| "default".to_string());
        let app_label =
            std::env::var("APP_LABEL").unwrap_or_else(|_| "distributed-cache".to_string());

        Self {
            namespace,
            app_label,
            hasher,
        }
    }

    pub async fn start(self) {
        let client = match Client::try_default().await {
            Ok(client) => client,
            Err(e) => {
                error!("Failed to create Kubernetes client: {}", e);
                return;
            }
        };

        let pods: Api<k8s_openapi::api::core::v1::Pod> = Api::namespaced(client, &self.namespace);
        let wp = WatchParams::default().labels(&format!("app={}", self.app_label));

        let mut stream = match pods.watch(&wp, "0").await {
            Ok(watcher) => watcher.boxed(),
            Err(e) => {
                error!("Failed to watch pods: {}", e);
                return;
            }
        };

        while let Some(status) = stream.next().await {
            match status {
                Ok(WatchEvent::Added(pod)) | Ok(WatchEvent::Modified(pod)) => {
                    if let Some(pod_ip) =
                        pod.status.as_ref().and_then(|status| status.pod_ip.clone())
                    {
                        self.hasher.add_node(pod_ip.clone());
                        info!("Pod added/modified: {}", pod_ip);
                    }
                }
                Ok(WatchEvent::Deleted(pod)) => {
                    if let Some(pod_ip) =
                        pod.status.as_ref().and_then(|status| status.pod_ip.clone())
                    {
                        self.hasher.remove_node(&pod_ip);
                        info!("Pod deleted: {}", pod_ip);
                    }
                }
                Err(e) => {
                    error!("Watch event error: {}", e);
                }
                _ => {}
            }
        }
    }
}
