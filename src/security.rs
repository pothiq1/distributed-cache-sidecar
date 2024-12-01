//security.rs

use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use tonic::transport::{Certificate, Identity, ServerTlsConfig};
use tonic::Request;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

pub struct Security {
    pub tls_config: Option<ServerTlsConfig>,
    pub jwt_secret: Option<String>,
}

impl Security {
    pub fn new(config: &crate::config::Config) -> Self {
        let tls_config = if let (Some(cert_path), Some(key_path)) =
            (&config.tls_cert_path, &config.tls_key_path)
        {
            let cert = std::fs::read(cert_path).expect("Failed to read TLS certificate");
            let key = std::fs::read(key_path).expect("Failed to read TLS key");
            let identity = Identity::from_pem(cert, key);
            Some(ServerTlsConfig::new().identity(identity))
        } else {
            None
        };

        let jwt_secret = config.jwt_secret.clone();

        Self {
            tls_config,
            jwt_secret,
        }
    }

    pub fn authenticate<T>(&self, request: &Request<T>) -> Result<(), tonic::Status> {
        if let Some(secret) = &self.jwt_secret {
            let token = request
                .metadata()
                .get("authorization")
                .and_then(|t| t.to_str().ok())
                .and_then(|s| s.strip_prefix("Bearer "))
                .ok_or_else(|| tonic::Status::unauthenticated("No valid auth token"))?;

            let decoding_key = DecodingKey::from_secret(secret.as_bytes());
            let validation = Validation::default();

            decode::<Claims>(token, &decoding_key, &validation)
                .map_err(|_| tonic::Status::unauthenticated("Invalid token"))?;
        }
        Ok(())
    }
}
