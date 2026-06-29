use std::sync::Arc;
use std::net::SocketAddr;
use sha2::{Digest, Sha256};
use tokio::sync::RwLock;
use tonic::metadata::MetadataMap;
use tonic::service::{interceptor, Interceptor};
use tonic::transport::{Identity, Server, ServerTlsConfig};
use tokio_rustls::rustls::{Certificate, PrivateKey, RootCertStore, ServerConfig as RustlsServerConfig, AllowAnyAuthenticatedClient};
use tonic_rustls::server::TlsConfigExt;
use std::io::Cursor;
use rustls_pemfile::{certs, read_one, Item};
use tonic::{Request, Status};
use tracing::{info, error, warn};

use crate::config::NetworkConfig;
use crate::database::ConnectionPool;
use crate::error::NetworkError;
use crate::signing::SigningService;
use crate::grpc::{
    NetworkServiceImpl, GatewayServiceImpl, HealthServiceImpl, P2PServiceImpl, VaultServiceImpl,
    ServiceRegistryImpl,
    network::network_service_server::NetworkServiceServer,
    network::vault_service_server::VaultServiceServer,
    network::health_service_server::HealthServiceServer,
    network::p2p_service_server::P2PServiceServer,
    network::service_registry_server::ServiceRegistryServer,
    gateway::gateway_service_server::GatewayServiceServer,
};
use crate::service_registry::ServiceDiscoveryRegistry;
use crate::state_trie::StateTrie;
use crate::p2p::P2PManager;
use crate::chain_params::ChainParameterRegistry;

const ADMIN_GRPC_PATHS: [&str; 4] = [
    "/axionvera.network.NetworkService/DistributeRewards",
    "/axionvera.network.NetworkService/ParameterUpgrade",
    "/axionvera.gateway.GatewayService/DistributeRewards",
    "/axionvera.gateway.GatewayService/ParameterUpgrade",
];

fn is_admin_grpc_route(path: &str) -> bool {
    ADMIN_GRPC_PATHS.contains(&path)
}

fn sha256_hex(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn extract_admin_token(metadata: &MetadataMap) -> Option<String> {
    if let Some(raw_auth_header) = metadata
        .get("authorization")
        .and_then(|value| value.to_str().ok())
    {
        let auth_header = raw_auth_header.trim();
        if let Some((scheme, token)) = auth_header.split_once(' ') {
            if scheme.eq_ignore_ascii_case("bearer") {
                let bearer_token = token.trim();
                if !bearer_token.is_empty() {
                    return Some(bearer_token.to_string());
                }
            }
        }
    }

    for key in ["x-api-key", "api-key"] {
        if let Some(raw_api_key) = metadata.get(key).and_then(|value| value.to_str().ok()) {
            let api_key = raw_api_key.trim();
            if !api_key.is_empty() {
                return Some(api_key.to_string());
            }
        }
    }

    None
}

#[derive(Clone, Debug)]
struct AdminAuthInterceptor {
    expected_token_hash: Option<Arc<str>>,
}

impl AdminAuthInterceptor {
    fn from_env() -> Self {
        Self::new(std::env::var("GRPC_ADMIN_AUTH_TOKEN_HASH").ok())
    }

    fn new(expected_token_hash: Option<String>) -> Self {
        let expected_token_hash = expected_token_hash
            .map(|hash| hash.trim().to_ascii_lowercase())
            .filter(|hash| !hash.is_empty())
            .map(Arc::<str>::from);

        Self { expected_token_hash }
    }

    fn is_configured(&self) -> bool {
        self.expected_token_hash.is_some()
    }
}

impl Interceptor for AdminAuthInterceptor {
    fn call(&mut self, request: Request<()>) -> Result<Request<()>, Status> {
        let path = request.uri().path();
        if !is_admin_grpc_route(path) {
            return Ok(request);
        }

        let expected_hash = self.expected_token_hash.as_deref().ok_or_else(|| {
            Status::unauthenticated("admin authentication is not configured")
        })?;

        let provided_token = extract_admin_token(request.metadata()).ok_or_else(|| {
            Status::unauthenticated("missing authorization token")
        })?;

        if sha256_hex(&provided_token) != expected_hash {
            return Err(Status::unauthenticated("invalid authorization token"));
        }

        Ok(request)
    }
}

pub struct GrpcServer {
    config: NetworkConfig,
    connection_pool: Arc<RwLock<ConnectionPool>>,
    state_trie: Arc<RwLock<StateTrie>>,
    p2p_manager: Arc<P2PManager>,
    signing_service: Arc<SigningService>,
    chain_parameters: Arc<RwLock<ChainParameterRegistry>>,
    service_registry: Arc<ServiceDiscoveryRegistry>,
}

impl Clone for GrpcServer {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            connection_pool: self.connection_pool.clone(),
            state_trie: self.state_trie.clone(),
            p2p_manager: self.p2p_manager.clone(),
            signing_service: self.signing_service.clone(),
            chain_parameters: self.chain_parameters.clone(),
            service_registry: self.service_registry.clone(),
        }
    }
}

impl GrpcServer {
    pub fn new(
        config: NetworkConfig,
        connection_pool: Arc<RwLock<ConnectionPool>>,
        state_trie: Arc<RwLock<StateTrie>>,
        p2p_manager: Arc<P2PManager>,
        signing_service: Arc<SigningService>,
        chain_parameters: Arc<RwLock<ChainParameterRegistry>>,
    ) -> Self {
        Self {
            config,
            connection_pool,
            state_trie,
            p2p_manager,
            signing_service,
            chain_parameters,
            service_registry: Arc::new(ServiceDiscoveryRegistry::new()),
        }
    }
    
    /// Get a reference to the signing service
    pub fn signing_service(&self) -> &Arc<SigningService> {
        &self.signing_service
    }

    pub async fn start(&self, shutdown_token: CancellationToken) -> Result<(), NetworkError> {
        let addr: SocketAddr = self.config.grpc_bind_address
            .parse()
            .map_err(|e| NetworkError::Config(format!("Invalid gRPC bind address: {}", e)))?;

        info!("Starting gRPC server on {}", addr);

        let admin_auth_interceptor = AdminAuthInterceptor::from_env();
        if !admin_auth_interceptor.is_configured() {
            warn!(
                "GRPC_ADMIN_AUTH_TOKEN_HASH is not set; administrative gRPC routes will reject requests"
            );
        }

        // Create service implementations
        let chain_cp = self.chain_parameters.clone();
        let network_service = NetworkServiceImpl::new(
            self.connection_pool.clone(),
            self.state_trie.clone(),
            self.p2p_manager.clone(),
            chain_cp.clone(),
        );

        let gateway_service = GatewayServiceImpl::new(
            self.connection_pool.clone(),
            self.state_trie.clone(),
            self.p2p_manager.clone(),
            chain_cp.clone(),
        );

        let health_service = HealthServiceImpl::new(self.connection_pool.clone());
        let p2p_service = P2PServiceImpl::new(self.p2p_manager.clone());
        let vault_service = VaultServiceImpl::new(self.connection_pool.clone());
        let service_registry_svc = ServiceRegistryImpl::new(self.service_registry.clone());

        let network_service = interceptor(
            NetworkServiceServer::new(network_service)
                .max_decoding_message_size(4 * 1024 * 1024), // 4MB max message size
            admin_auth_interceptor.clone(),
        );

        let gateway_service = interceptor(
            GatewayServiceServer::new(gateway_service)
                .max_decoding_message_size(4 * 1024 * 1024),
            admin_auth_interceptor.clone(),
        );

        // Build the gRPC server with middleware
        let mut server = Server::builder()
            .add_service(network_service)
            .add_service(gateway_service)
            .add_service(
                VaultServiceServer::new(vault_service)
                    .max_decoding_message_size(1024 * 1024)
            )
            .add_service(
                HealthServiceServer::new(health_service)
                    .max_decoding_message_size(1024 * 1024) // 1MB for health checks
            )
            .add_service(
                P2PServiceServer::new(p2p_service)
                    .max_decoding_message_size(8 * 1024 * 1024) // 8MB for P2P messages
            )
            .add_service(
                ServiceRegistryServer::new(service_registry_svc)
                    .max_decoding_message_size(1024 * 1024)
            );

        // Add gRPC-Web support for browser clients
        server = server.add_service(
            interceptor(
                GatewayServiceServer::new(GatewayServiceImpl::new(
                    self.connection_pool.clone(),
                    self.state_trie.clone(),
                    self.p2p_manager.clone(),
                    self.chain_parameters.clone(),
                ))
                .accept_compressed(tonic::codec::CompressionEncoding::Gzip)
                .send_compressed(tonic::codec::CompressionEncoding::Gzip),
                admin_auth_interceptor.clone(),
            )
        );

        // Configure TLS (with optional mTLS) if certificates are provided
        if let (Some(cert_path), Some(key_path)) = (&self.config.tls_cert_path, &self.config.tls_key_path) {
            info!("Configuring TLS for gRPC server");

            // Read server cert and key as PEM bytes and parse into DER for rustls
            let cert_pem = std::fs::read(cert_path)
                .map_err(|e| NetworkError::Config(format!("Failed to read TLS certificate: {}", e)))?;
            let key_pem = std::fs::read(key_path)
                .map_err(|e| NetworkError::Config(format!("Failed to read TLS private key: {}", e)))?;

            // Build identity for tonic by converting to strings (preserve previous behavior)
            let cert_str = String::from_utf8(cert_pem.clone())
                .map_err(|e| NetworkError::Config(format!("Invalid cert PEM encoding: {}", e)))?;
            let key_str = String::from_utf8(key_pem.clone())
                .map_err(|e| NetworkError::Config(format!("Invalid key PEM encoding: {}", e)))?;

            let identity = Identity::from_pem(cert_str, key_str);

            // If a client CA is provided, configure rustls to require client authentication
            if let Some(client_ca_path) = &self.config.tls_client_ca_path {
                info!("Configuring mTLS: requiring client certificates");

                let ca_pem = std::fs::read(client_ca_path)
                    .map_err(|e| NetworkError::Config(format!("Failed to read TLS client CA file: {}", e)))?;

                // Parse CA certs and populate a RootCertStore using rustls-pemfile
                let mut roots = RootCertStore::empty();
                let mut cursor = Cursor::new(&ca_pem);
                let parsed = certs(&mut cursor)
                    .map_err(|_| NetworkError::Config("Failed to parse client CA PEM".to_string()))?;

                if parsed.is_empty() {
                    return Err(NetworkError::Config("No CA certificates found in tls_client_ca_path".to_string()));
                }

                for cert in parsed {
                    roots.add(&Certificate(cert)).map_err(|e| NetworkError::Config(format!("Failed to add CA cert to root store: {}", e)))?;
                }

                // Parse server cert PEM to DER vec
                let mut cert_cursor = Cursor::new(&cert_pem);
                let server_certs = certs(&mut cert_cursor)
                    .map_err(|_| NetworkError::Config("Failed to parse server cert PEM".to_string()))?;

                if server_certs.is_empty() {
                    return Err(NetworkError::Config("No server certificates found in TLS cert file".to_string()));
                }

                // Parse private key PEM (support pkcs8 and rsa)
                let mut key_cursor = Cursor::new(&key_pem);
                let mut keys = Vec::new();
                loop {
                    match read_one(&mut key_cursor).map_err(|_| NetworkError::Config("Failed to parse TLS private key PEM".to_string()))? {
                        Some(Item::PKCS8Key(key)) => { keys.push(key); }
                        Some(Item::RSAKey(key)) => { keys.push(key); }
                        Some(_) => {}
                        None => break,
                    }
                }

                if keys.is_empty() {
                    return Err(NetworkError::Config("No private keys found in TLS key file".to_string()));
                }

                let der_certs = server_certs.into_iter().map(Certificate).collect::<Vec<_>>();
                let der_key = PrivateKey(keys.remove(0));

                let mut rustls_cfg = RustlsServerConfig::builder()
                    .with_safe_defaults()
                    .with_client_cert_verifier(AllowAnyAuthenticatedClient::new(roots))
                    .with_single_cert(der_certs.clone(), der_key.clone())
                    .map_err(|e| NetworkError::Config(format!("Failed to create rustls server config: {}", e)))?;

                // If configured to not require client auth, switch to NoClientAuth
                if !self.config.tls_require_client_auth {
                    rustls_cfg = RustlsServerConfig::builder()
                        .with_safe_defaults()
                        .with_no_client_auth()
                        .with_single_cert(der_certs.clone(), der_key.clone())
                        .map_err(|e| NetworkError::Config(format!("Failed to create rustls server config: {}", e)))?;
                }

                // Convert rustls config into a tonic ServerTlsConfig via tonic-rustls helper
                let tls_config = ServerTlsConfig::new().rustls_server_config(Arc::new(rustls_cfg));

                server = server.tls_config(tls_config)
                    .map_err(|e| NetworkError::Config(format!("Failed to configure TLS: {}", e)))?;
            } else {
                // No client CA provided: use one-way TLS with existing Identity path
                let tls_config = ServerTlsConfig::new().identity(identity);
                server = server.tls_config(tls_config)
                    .map_err(|e| NetworkError::Config(format!("Failed to configure TLS: {}", e)))?;
            }
        }

        // Add reflection service for development
        #[cfg(debug_assertions)]
        {
            use tonic_reflection::server::{ServerReflection, ServerReflectionServer};
            let reflection_service = ServerReflectionServer::new(ServerReflection::new());
            server = server.add_service(reflection_service);
            info!("gRPC reflection service enabled");
        }

        // Apply interceptors for logging and metrics
        server = server.intercept_fn(|req| {
            info!("gRPC request: path={}", req.uri().path());
            Ok(req)
        });

        // Start the server
        let server_future = server.serve_with_shutdown(addr, async move {
            shutdown_token.cancelled().await;
            info!("Shutdown signal received, stopping gRPC server");
        });

        info!("gRPC server started successfully on {}", addr);

        match server_future.await {
            Ok(_) => {
                info!("gRPC server stopped gracefully");
                Ok(())
            }
            Err(e) => {
                error!("gRPC server error: {}", e);
                Err(NetworkError::Server(format!("gRPC server failed: {}", e)))
            }
        }
    }

    pub async fn start_with_health_check(&self, shutdown_token: CancellationToken) -> Result<(), NetworkError> {
        // Start health check service in a separate task
        let health_service = HealthServiceImpl::new(self.connection_pool.clone());
        let health_addr: SocketAddr = "0.0.0.0:50051"
            .parse()
            .map_err(|e| NetworkError::Config(format!("Invalid health check address: {}", e)))?;

        let health_shutdown_token = shutdown_token.clone();
        tokio::spawn(async move {
            info!("Starting gRPC health check service on {}", health_addr);
            
            let health_server = Server::builder()
                .add_service(HealthServiceServer::new(health_service))
                .serve_with_shutdown(health_addr, async move {
                    health_shutdown_token.cancelled().await;
                    info!("Shutdown signal received, stopping health check service");
                });

            if let Err(e) = health_server.await {
                error!("Health check service error: {}", e);
            }
        });

        // Start main gRPC server
        self.start(shutdown_token).await
    }
}

// gRPC Gateway for HTTP/JSON-RPC interface
pub struct GrpcGateway {
    config: NetworkConfig,
    grpc_address: String,
}

impl GrpcGateway {
    pub fn new(config: NetworkConfig, grpc_address: String) -> Self {
        Self {
            config,
            grpc_address,
        }
    }

    pub async fn start(&self) -> Result<(), NetworkError> {
        info!("Starting gRPC Gateway for HTTP/JSON-RPC interface");

        // TODO: Implement grpc-gateway HTTP reverse proxy
        // This would typically use grpc-gateway or a custom HTTP-to-gRPC proxy
        
        warn!("gRPC Gateway HTTP interface not yet implemented");
        info!("Use the gRPC endpoint directly: {}", self.grpc_address);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tonic::metadata::MetadataValue;

    fn test_request(path: &str) -> Request<()> {
        Request::from_http(
            http::Request::builder()
                .uri(path)
                .body(())
                .expect("failed to build test request"),
        )
    }

    #[test]
    fn admin_route_without_token_is_rejected() {
        let mut interceptor = AdminAuthInterceptor::new(Some(sha256_hex("test-token")));
        let request = test_request("/axionvera.network.NetworkService/DistributeRewards");

        let error = interceptor.call(request).expect_err("request should fail");
        assert_eq!(error.code(), tonic::Code::Unauthenticated);
    }

    #[test]
    fn admin_route_with_valid_bearer_token_is_allowed() {
        let mut interceptor = AdminAuthInterceptor::new(Some(sha256_hex("test-token")));
        let mut request = test_request("/axionvera.network.NetworkService/DistributeRewards");
        request.metadata_mut().insert(
            "authorization",
            MetadataValue::try_from("Bearer test-token").expect("invalid metadata value"),
        );

        assert!(interceptor.call(request).is_ok());
    }

    #[test]
    fn admin_route_with_valid_api_key_is_allowed() {
        let mut interceptor = AdminAuthInterceptor::new(Some(sha256_hex("test-token")));
        let mut request = test_request("/axionvera.gateway.GatewayService/ParameterUpgrade");
        request.metadata_mut().insert(
            "x-api-key",
            MetadataValue::try_from("test-token").expect("invalid metadata value"),
        );

        assert!(interceptor.call(request).is_ok());
    }

    #[test]
    fn admin_route_with_invalid_token_is_rejected() {
        let mut interceptor = AdminAuthInterceptor::new(Some(sha256_hex("valid-token")));
        let mut request = test_request("/axionvera.network.NetworkService/ParameterUpgrade");
        request.metadata_mut().insert(
            "authorization",
            MetadataValue::try_from("Bearer wrong-token").expect("invalid metadata value"),
        );

        let error = interceptor.call(request).expect_err("request should fail");
        assert_eq!(error.code(), tonic::Code::Unauthenticated);
    }

    #[test]
    fn public_route_without_token_is_allowed() {
        let mut interceptor = AdminAuthInterceptor::new(Some(sha256_hex("test-token")));
        let request = test_request("/axionvera.network.NetworkService/GetBalance");

        assert!(interceptor.call(request).is_ok());
    }

    #[test]
    fn admin_route_is_rejected_when_auth_not_configured() {
        let mut interceptor = AdminAuthInterceptor::new(None);
        let mut request = test_request("/axionvera.gateway.GatewayService/DistributeRewards");
        request.metadata_mut().insert(
            "x-api-key",
            MetadataValue::try_from("test-token").expect("invalid metadata value"),
        );

        let error = interceptor.call(request).expect_err("request should fail");
        assert_eq!(error.code(), tonic::Code::Unauthenticated);
    }
}
