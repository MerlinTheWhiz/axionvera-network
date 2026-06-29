use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use prost_types::Timestamp;
use tonic::{Request, Response, Status};
use tracing::info;

use crate::grpc::network::{
    service_registry_server::ServiceRegistry, DeregisterServiceRequest, DeregisterServiceResponse,
    ListServicesResponse, LookupServiceRequest, LookupServiceResponse, RegisterServiceRequest,
    RegisterServiceResponse, ServiceInfo,
};
use crate::service_registry::ServiceDiscoveryRegistry;

pub struct ServiceRegistryImpl {
    registry: Arc<ServiceDiscoveryRegistry>,
}

impl ServiceRegistryImpl {
    pub fn new(registry: Arc<ServiceDiscoveryRegistry>) -> Self {
        Self { registry }
    }
}

fn now_timestamp() -> Option<Timestamp> {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    Some(Timestamp { seconds: secs, nanos: 0 })
}

fn entry_to_proto(entry: crate::service_registry::ServiceEntry) -> ServiceInfo {
    ServiceInfo {
        service_name: entry.name,
        service_address: entry.address,
        version: entry.version,
        metadata: entry.metadata,
        registered_at: Some(Timestamp {
            seconds: entry.registered_at as i64,
            nanos: 0,
        }),
    }
}

#[tonic::async_trait]
impl ServiceRegistry for ServiceRegistryImpl {
    async fn register_service(
        &self,
        request: Request<RegisterServiceRequest>,
    ) -> Result<Response<RegisterServiceResponse>, Status> {
        let req = request.into_inner();
        info!("RegisterService: name={}", req.service_name);

        match self
            .registry
            .register(
                req.service_name,
                req.service_address,
                req.version,
                req.metadata,
            )
            .await
        {
            Ok(entry) => Ok(Response::new(RegisterServiceResponse {
                success: true,
                error_message: String::new(),
                service: Some(entry_to_proto(entry)),
            })),
            Err(e) => Ok(Response::new(RegisterServiceResponse {
                success: false,
                error_message: e,
                service: None,
            })),
        }
    }

    async fn deregister_service(
        &self,
        request: Request<DeregisterServiceRequest>,
    ) -> Result<Response<DeregisterServiceResponse>, Status> {
        let req = request.into_inner();
        info!("DeregisterService: name={}", req.service_name);

        let removed = self.registry.deregister(&req.service_name).await;
        Ok(Response::new(DeregisterServiceResponse {
            success: removed,
            error_message: if removed {
                String::new()
            } else {
                format!("service '{}' not found", req.service_name)
            },
        }))
    }

    async fn lookup_service(
        &self,
        request: Request<LookupServiceRequest>,
    ) -> Result<Response<LookupServiceResponse>, Status> {
        let req = request.into_inner();
        info!("LookupService: name={}", req.service_name);

        let entry = self.registry.lookup(&req.service_name).await;
        Ok(Response::new(LookupServiceResponse {
            found: entry.is_some(),
            service: entry.map(entry_to_proto),
        }))
    }

    async fn list_services(
        &self,
        _request: Request<()>,
    ) -> Result<Response<ListServicesResponse>, Status> {
        info!("ListServices");

        let services: Vec<ServiceInfo> = self
            .registry
            .list()
            .await
            .into_iter()
            .map(entry_to_proto)
            .collect();

        let total = services.len() as u64;
        Ok(Response::new(ListServicesResponse {
            services,
            total_count: total,
        }))
    }
}
