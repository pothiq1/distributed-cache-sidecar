# **Distributed Cache Sidecar**

**Distributed Cache Sidecar** is a modern, high-performance distributed key-value caching system designed for scalable microservices and distributed architectures. Built with **Rust**, it ensures low-latency data retrieval, fault tolerance, and real-time monitoring. This system is tailored for applications requiring **high availability** and **fast caching** mechanisms with advanced features like replication, transactional support, and full-text search.

---

## **Table of Contents**

- [Features](#features)
- [Architecture Overview](#architecture-overview)
- [Installation](#installation)
- [Usage and API Endpoints](#usage-and-api-endpoints)
- [Configuration](#configuration)
- [Deployment](#deployment)
- [Monitoring](#monitoring)
- [Development](#development)
- [Contributing](#contributing)
- [Future Roadmap](#future-roadmap)
- [License](#license)
- [Acknowledgments](#acknowledgments)

---

## **Features**

### **Core Functionalities**

- **In-Memory Caching**: Ultra-fast key-value storage with LZ4 compression.
- **TTL (Time-to-Live)**: Expiration mechanism for stale entries.
- **Eviction Policy**: Uses LRU (Least Recently Used) for managing memory constraints.
- **Transaction Support**: Atomic, rollback-enabled transactional operations.

### **Advanced Capabilities**

- **Data Replication**: Ensures fault tolerance through consistent hashing and configurable replication factors.
- **Real-Time Monitoring**: Exposes Prometheus-compatible metrics for system insights.
- **Full-Text Search**: Seamlessly search through cached data using Tantivy-powered indexing.

### **Security**

- **JWT Authentication**: Secures endpoints with token-based authentication.
- **TLS Encryption**: Provides secure communication between nodes using TLS.

### **Scalability**

- **Dynamic Cluster Management**: Easily add or remove nodes with automatic load redistribution.
- **Configurable Parameters**: Adjust settings like memory limits, replication factors, and TTL dynamically.

---

## **Architecture Overview**

### **System Components**

| Component               | Description                                                               |
| ----------------------- | ------------------------------------------------------------------------- |
| **Cache Manager**       | Handles key-value storage, eviction policies, and TTL-based expiration.   |
| **Transaction Manager** | Provides atomic operations, rollback support, and timeout handling.       |
| **Replicator**          | Synchronizes data across nodes, ensuring consistency and fault tolerance. |
| **Monitoring Service**  | Tracks metrics like cache hits, misses, memory usage, and node health.    |
| **Search Index**        | Enables full-text search capabilities on cached data using Tantivy.       |
| **Security Layer**      | Secures API endpoints with JWT and optional TLS encryption.               |

---

## **Installation**

### **Prerequisites**

- **Rust**: Install via [Rustup](https://rustup.rs/).
- **Docker** (Optional): For containerized deployments.
- **Prometheus** (Optional): For monitoring and metrics collection.

### **Steps**

1. **Clone the Repository**

   ```bash
   git clone git@github.com:pothiq1/distributed-cache-sidecar.git
   cd distributed-cache-sidecar
   ```

2. **Build the Application**

   ```bash
   cargo build --release
   ```

3. **Run the Application**
   ```bash
   cargo run --release
   ```

---

## **Usage and API Endpoints**

### **Cache Operations**

- **`GET /cache?key=<key>`**

  - Retrieves the value associated with a key.
  - **Response**:
    ```json
    {
      "value": "exampleValue",
      "found": true
    }
    ```

- **`POST /cache`**
  - Stores a key-value pair in the cache.
  - **Request Body**:
    ```json
    {
      "key": "exampleKey",
      "value": "exampleValue",
      "ttl": 300
    }
    ```
  - **Response**:
    ```json
    {
      "success": true
    }
    ```

### **Transaction Management**

- **`POST /transaction/start`**

  - Starts a new transaction and returns a transaction ID.
  - **Response**:
    ```json
    {
      "transaction_id": "unique_transaction_id"
    }
    ```

- **`POST /transaction/commit`**

  - Commits changes made within a transaction.
  - **Request Body**:
    ```json
    {
      "transaction_id": "unique_transaction_id"
    }
    ```

- **`POST /transaction/rollback`**
  - Rolls back all changes within a transaction.

### **Monitoring and Metrics**

- **`GET /metrics`**
  - Prometheus-compatible endpoint exposing system metrics.
- **`GET /stats`**
  - Provides high-level system stats, including cache memory usage and hit/miss ratios.
- **`GET /nodes`**
  - Returns a list of active nodes in the cluster.

---

## **Configuration**

Create a configuration file (`config.toml`) with the following template:

```toml
# Cache Configuration
max_memory = 1073741824  # Max memory in bytes (1GB)
default_ttl = 300        # Default TTL in seconds
replication_factor = 2   # Number of replicas per key

# Transactions
enable_transactions = true
transaction_timeout = 30 # Transaction timeout in seconds

# Security
jwt_secret = "your_jwt_secret"
tls_cert_path = "/path/to/cert.pem"
tls_key_path = "/path/to/key.pem"

# Monitoring
metrics_port = 9898
```

---

## **Deployment**

### **Using Docker**

1. Build the Docker image:

   ```bash
   docker build -t distributed-cache-sidecar .
   ```

2. Run the Docker container:
   ```bash
   docker run -p 50051:50051 -p 9898:9898 distributed-cache-sidecar
   ```

### **Using Kubernetes**

1. Create a `k8s.yaml` file with your deployment and service configuration.
2. Deploy using `kubectl`:
   ```bash
   kubectl apply -f k8s.yaml
   ```

---

## **Monitoring**

### **Prometheus Integration**

Add the following job to your Prometheus configuration to start scraping metrics:

```yaml
scrape_configs:
  - job_name: "distributed-cache-sidecar"
    static_configs:
      - targets: ["localhost:9898"]
```

Access the metrics at `http://<host>:9898/metrics`.

---

## **Development**

### **Run Tests**

Execute unit tests to validate functionality:

```bash
cargo test
```

### **Debug Mode**

Run the application in debug mode for development:

```bash
cargo run
```

---

## **Contributing**

We welcome contributions to enhance the project. To contribute:

1. Fork this repository.
2. Create a feature branch.
3. Implement your changes.
4. Submit a pull request with detailed information about the changes.

---

## **Future Roadmap**

- **Cluster Autoscaling**: Automatic detection and management of new nodes.
- **Web Dashboard**: Intuitive UI for monitoring cache performance and managing nodes.
- **Advanced Search Features**: Query planners for distributed search across clusters.
- **Improved Security**: Role-based access control (RBAC) and OAuth2 support.

---

## **License**

Distributed Cache Sidecar is licensed under the **[MIT License](LICENSE)**. See the `LICENSE` file for more details.

---

## **Acknowledgments**

Special thanks to:

- **Rust Community**: For providing an exceptional ecosystem.
- **Prometheus**: For monitoring and observability support.
- **Tantivy**: A fast, full-text search engine in Rust.

```

```
