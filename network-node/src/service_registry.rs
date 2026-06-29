use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

/// Metadata and location of a registered service.
#[derive(Clone, Debug, PartialEq)]
pub struct ServiceEntry {
    pub name: String,
    pub address: String,
    pub version: String,
    pub metadata: HashMap<String, String>,
    pub registered_at: u64,
}

impl ServiceEntry {
    fn new(
        name: String,
        address: String,
        version: String,
        metadata: HashMap<String, String>,
    ) -> Self {
        let registered_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        Self { name, address, version, metadata, registered_at }
    }
}

/// Thread-safe in-memory service discovery registry.
#[derive(Clone, Debug)]
pub struct ServiceDiscoveryRegistry {
    services: Arc<RwLock<HashMap<String, ServiceEntry>>>,
}

impl ServiceDiscoveryRegistry {
    pub fn new() -> Self {
        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a service. Returns `false` if a service with that name is already registered.
    pub async fn register(
        &self,
        name: String,
        address: String,
        version: String,
        metadata: HashMap<String, String>,
    ) -> Result<ServiceEntry, String> {
        let mut map = self.services.write().await;
        if map.contains_key(&name) {
            return Err(format!("service '{}' is already registered", name));
        }
        let entry = ServiceEntry::new(name.clone(), address, version, metadata);
        map.insert(name, entry.clone());
        Ok(entry)
    }

    /// Deregister a service by name. Returns `false` if not found.
    pub async fn deregister(&self, name: &str) -> bool {
        let mut map = self.services.write().await;
        map.remove(name).is_some()
    }

    /// Look up a service by name.
    pub async fn lookup(&self, name: &str) -> Option<ServiceEntry> {
        self.services.read().await.get(name).cloned()
    }

    /// List all registered services.
    pub async fn list(&self) -> Vec<ServiceEntry> {
        self.services.read().await.values().cloned().collect()
    }
}

impl Default for ServiceDiscoveryRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_register_and_lookup() {
        let registry = ServiceDiscoveryRegistry::new();
        let entry = registry
            .register("vault".into(), "localhost:9000".into(), "1.0.0".into(), HashMap::new())
            .await
            .unwrap();
        assert_eq!(entry.name, "vault");
        assert_eq!(entry.address, "localhost:9000");

        let found = registry.lookup("vault").await.unwrap();
        assert_eq!(found.name, "vault");
    }

    #[tokio::test]
    async fn test_register_duplicate_fails() {
        let registry = ServiceDiscoveryRegistry::new();
        registry
            .register("vault".into(), "localhost:9000".into(), "1.0.0".into(), HashMap::new())
            .await
            .unwrap();
        let result = registry
            .register("vault".into(), "localhost:9001".into(), "1.0.0".into(), HashMap::new())
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_deregister() {
        let registry = ServiceDiscoveryRegistry::new();
        registry
            .register("vault".into(), "localhost:9000".into(), "1.0.0".into(), HashMap::new())
            .await
            .unwrap();
        assert!(registry.deregister("vault").await);
        assert!(registry.lookup("vault").await.is_none());
    }

    #[tokio::test]
    async fn test_deregister_nonexistent_returns_false() {
        let registry = ServiceDiscoveryRegistry::new();
        assert!(!registry.deregister("nonexistent").await);
    }

    #[tokio::test]
    async fn test_list_services() {
        let registry = ServiceDiscoveryRegistry::new();
        registry
            .register("svc_a".into(), "a:1".into(), "1.0".into(), HashMap::new())
            .await
            .unwrap();
        registry
            .register("svc_b".into(), "b:2".into(), "2.0".into(), HashMap::new())
            .await
            .unwrap();
        let mut services = registry.list().await;
        services.sort_by(|a, b| a.name.cmp(&b.name));
        assert_eq!(services.len(), 2);
        assert_eq!(services[0].name, "svc_a");
        assert_eq!(services[1].name, "svc_b");
    }

    #[tokio::test]
    async fn test_lookup_nonexistent_returns_none() {
        let registry = ServiceDiscoveryRegistry::new();
        assert!(registry.lookup("missing").await.is_none());
    }
}
