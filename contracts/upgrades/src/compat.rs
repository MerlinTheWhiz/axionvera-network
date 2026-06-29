/// Severity assigned to an upgrade compatibility check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompatStatus {
    /// The checked item is unchanged or explicitly compatible.
    Compatible,
    /// The checked item is additive or operationally risky, but not blocking.
    Warning(String),
    /// The checked item can strand state or break callers and must block upgrade.
    Breaking(String),
}

impl CompatStatus {
    pub fn is_breaking(&self) -> bool {
        matches!(self, Self::Breaking(_))
    }
}

/// Expected storage location for a persisted key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageScope {
    Instance,
    Persistent,
}

/// A versioned storage key declaration used by pre-upgrade validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageKeySpec {
    pub name: String,
    pub scope: StorageScope,
    pub type_hash: String,
    pub required: bool,
}

impl StorageKeySpec {
    pub fn new(name: &str, scope: StorageScope, type_hash: &str, required: bool) -> Self {
        Self {
            name: name.to_string(),
            scope,
            type_hash: type_hash.to_string(),
            required,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageFieldCheck {
    pub name: String,
    pub status: CompatStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventCheck {
    pub name: String,
    pub status: CompatStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterfaceCheck {
    pub function: String,
    pub status: CompatStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorizationCheck {
    pub rule: String,
    pub status: CompatStatus,
}

/// Migration phase tracked by upgrade runbooks and tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MigrationPhase {
    Proposed,
    Validated,
    Authorized,
    Executed,
    Verified,
    RolledBack,
}

/// One migration step with enough metadata for audit trails and dry-run tests.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationStep {
    pub name: String,
    pub phase: MigrationPhase,
    pub reversible: bool,
}

impl MigrationStep {
    pub fn new(name: &str, phase: MigrationPhase, reversible: bool) -> Self {
        Self {
            name: name.to_string(),
            phase,
            reversible,
        }
    }
}

/// Aggregate result for an upgrade compatibility and migration readiness check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompatibilityResult {
    pub v1_hash: String,
    pub v2_hash: String,
    pub storage: Vec<StorageFieldCheck>,
    pub events: Vec<EventCheck>,
    pub interfaces: Vec<InterfaceCheck>,
    pub authorization: Vec<AuthorizationCheck>,
    pub migration: Vec<MigrationStep>,
    pub is_fully_compatible: bool,
}

impl CompatibilityResult {
    pub fn new(v1: &str, v2: &str) -> Self {
        Self {
            v1_hash: v1.to_string(),
            v2_hash: v2.to_string(),
            storage: vec![],
            events: vec![],
            interfaces: vec![],
            authorization: vec![],
            migration: vec![],
            is_fully_compatible: true,
        }
    }

    pub fn add_storage(&mut self, name: &str, status: CompatStatus) {
        self.record_status(&status);
        self.storage.push(StorageFieldCheck {
            name: name.to_string(),
            status,
        });
    }

    pub fn add_event(&mut self, name: &str, status: CompatStatus) {
        self.record_status(&status);
        self.events.push(EventCheck {
            name: name.to_string(),
            status,
        });
    }

    pub fn add_interface(&mut self, func: &str, status: CompatStatus) {
        self.record_status(&status);
        self.interfaces.push(InterfaceCheck {
            function: func.to_string(),
            status,
        });
    }

    pub fn add_authorization(&mut self, rule: &str, status: CompatStatus) {
        self.record_status(&status);
        self.authorization.push(AuthorizationCheck {
            rule: rule.to_string(),
            status,
        });
    }

    pub fn add_migration_step(&mut self, step: MigrationStep) {
        self.migration.push(step);
    }

    pub fn breaking_count(&self) -> usize {
        self.storage
            .iter()
            .filter(|s| s.status.is_breaking())
            .count()
            + self
                .events
                .iter()
                .filter(|e| e.status.is_breaking())
                .count()
            + self
                .interfaces
                .iter()
                .filter(|i| i.status.is_breaking())
                .count()
            + self
                .authorization
                .iter()
                .filter(|a| a.status.is_breaking())
                .count()
    }

    pub fn warning_count(&self) -> usize {
        self.storage
            .iter()
            .filter(|s| matches!(s.status, CompatStatus::Warning(_)))
            .count()
            + self
                .events
                .iter()
                .filter(|e| matches!(e.status, CompatStatus::Warning(_)))
                .count()
            + self
                .interfaces
                .iter()
                .filter(|i| matches!(i.status, CompatStatus::Warning(_)))
                .count()
            + self
                .authorization
                .iter()
                .filter(|a| matches!(a.status, CompatStatus::Warning(_)))
                .count()
    }

    fn record_status(&mut self, status: &CompatStatus) {
        if status.is_breaking() {
            self.is_fully_compatible = false;
        }
    }
}
