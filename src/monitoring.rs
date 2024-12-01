use prometheus::{Encoder, IntCounterVec, Opts, Registry, TextEncoder};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::info;
use warp::Filter;

pub struct Monitoring {
    pub registry: Registry,
    pub cache_hits: IntCounterVec,
    pub cache_misses: IntCounterVec,
}

impl Monitoring {
    pub fn new() -> Self {
        let registry = Registry::new();
        let cache_hits =
            IntCounterVec::new(Opts::new("cache_hits", "Number of cache hits"), &["method"])
                .unwrap();
        let cache_misses = IntCounterVec::new(
            Opts::new("cache_misses", "Number of cache misses"),
            &["method"],
        )
        .unwrap();

        registry.register(Box::new(cache_hits.clone())).unwrap();
        registry.register(Box::new(cache_misses.clone())).unwrap();

        Self {
            registry,
            cache_hits,
            cache_misses,
        }
    }

    pub fn serve(
        self,
        cache: Arc<crate::cache::Cache>,
        hasher: Arc<crate::hashing::ConsistentHashing>,
        config: Arc<crate::config::Config>,
    ) {
        let registry = self.registry.clone();
        let cache_clone = cache.clone();
        let hasher_clone = hasher.clone();
        let config_clone = config.clone();

        // Metrics route
        let metrics_registry = registry.clone();
        let metrics_route = warp::path("metrics").map(move || {
            let encoder = TextEncoder::new();
            let metric_families = metrics_registry.gather();
            let mut buffer = Vec::new();
            encoder.encode(&metric_families, &mut buffer).unwrap();
            let response = String::from_utf8(buffer.clone()).unwrap();
            warp::reply::with_header(response, "Content-Type", encoder.format_type().to_string())
        });

        // Stats route
        let cache_clone_stats = cache.clone();
        let stats_route = warp::path("stats").map(move || {
            let current_memory = cache_clone_stats.get_current_memory();
            let entry_count = cache_clone_stats.data.len();
            let response = json!({
                "cache_hits": 0, // Replace with actual metrics
                "cache_misses": 0, // Replace with actual metrics
                "memory_usage": current_memory,
                "entry_count": entry_count,
            });
            warp::reply::json(&response)
        });

        // Nodes route
        let hasher_clone_nodes = hasher.clone();
        let nodes_route = warp::path("nodes").map(move || {
            let nodes = hasher_clone_nodes.get_all_nodes();
            warp::reply::json(&nodes)
        });

        // Memory usage route
        let memory_route = warp::path("memory_usage").map(move || {
            // For demonstration purposes, we'll simulate data
            // In a real implementation, you'd collect this data from each node
            let memory_data = json!([
                {
                    "node": "node1",
                    "main_cache": 50000000,
                    "replicas": 20000000
                },
                {
                    "node": "node2",
                    "main_cache": 40000000,
                    "replicas": 30000000
                }
            ]);
            warp::reply::json(&memory_data)
        });

        // Configuration route
        let config_clone_get = config.clone();
        let config_route = warp::path("config").map(move || {
            // Return current configuration
            // In a real implementation, you should secure this endpoint
            let config_data = json!({
                "max_memory": config_clone_get.max_memory,
                "default_ttl": config_clone_get.default_ttl,
                // Include other configuration parameters as needed
            });
            warp::reply::json(&config_data)
        });

        // Update configuration route
        let config_clone_update = config.clone();
        let config_update_route = warp::path("update_config")
            .and(warp::post())
            .and(warp::body::json())
            .map(move |config_update: serde_json::Value| {
                // Handle configuration updates
                // For security, you should authenticate and validate inputs
                // Here we'll just log the update
                info!("Received config update: {:?}", config_update);
                warp::reply::json(&json!({"status": "success"}))
            });

        // Search cache route
        let cache_clone_search = cache.clone();
        let search_cache_route = warp::path("search_cache")
            .and(warp::get())
            .and(warp::query::<HashMap<String, String>>())
            .map(move |params: HashMap<String, String>| {
                if let Some(key) = params.get("key") {
                    if let Some(value) = cache_clone_search.get(key) {
                        let response = json!({
                            "found": true,
                            "value": String::from_utf8_lossy(&value).to_string(),
                        });
                        warp::reply::json(&response)
                    } else {
                        let response = json!({ "found": false });
                        warp::reply::json(&response)
                    }
                } else {
                    warp::reply::json(&json!({ "error": "Key parameter is missing" }))
                }
            });

        // Combine all routes
        let routes = metrics_route
            .or(stats_route)
            .or(nodes_route)
            .or(memory_route)
            .or(config_route)
            .or(config_update_route)
            .or(search_cache_route)
            .with(
                warp::cors()
                    .allow_any_origin()
                    .allow_methods(vec!["GET", "POST"])
                    .allow_headers(vec!["Content-Type"]),
            );

        // Start the Warp server
        tokio::spawn(warp::serve(routes).run(([0, 0, 0, 0], 9898)));
    }
}
