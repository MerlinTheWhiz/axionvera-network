#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompatStatus {
    Compatible,
    Warning(String),
    Breaking(String),
}

#[derive(Debug, Clone)]
pub struct StorageFieldCheck { pub name: String, pub status: CompatStatus }

#[derive(Debug, Clone)]
pub struct EventCheck { pub name: String, pub status: CompatStatus }

#[derive(Debug, Clone)]
pub struct InterfaceCheck { pub function: String, pub status: CompatStatus }

#[derive(Debug, Clone)]
pub struct CompatibilityResult {
    pub v1_hash: String,
    pub v2_hash: String,
    pub storage: Vec<StorageFieldCheck>,
    pub events: Vec<EventCheck>,
    pub interfaces: Vec<InterfaceCheck>,
    pub is_fully_compatible: bool,
}

impl CompatibilityResult {
    pub fn new(v1: &str, v2: &str) -> Self {
        Self { v1_hash: v1.to_string(), v2_hash: v2.to_string(), storage: vec![], events: vec![], interfaces: vec![], is_fully_compatible: true }
    }
    pub fn add_storage(&mut self, name: &str, status: CompatStatus) {
        if matches!(status, CompatStatus::Breaking(_)) { self.is_fully_compatible = false; }
        self.storage.push(StorageFieldCheck { name: name.to_string(), status });
    }
    pub fn add_event(&mut self, name: &str, status: CompatStatus) {
        if matches!(status, CompatStatus::Breaking(_)) { self.is_fully_compatible = false; }
        self.events.push(EventCheck { name: name.to_string(), status });
    }
    pub fn add_interface(&mut self, func: &str, status: CompatStatus) {
        if matches!(status, CompatStatus::Breaking(_)) { self.is_fully_compatible = false; }
        self.interfaces.push(InterfaceCheck { function: func.to_string(), status });
    }
    pub fn breaking_count(&self) -> usize {
        self.storage.iter().filter(|s| matches!(s.status, CompatStatus::Breaking(_))).count()
            + self.events.iter().filter(|e| matches!(e.status, CompatStatus::Breaking(_))).count()
            + self.interfaces.iter().filter(|i| matches!(i.status, CompatStatus::Breaking(_))).count()
    }
}
