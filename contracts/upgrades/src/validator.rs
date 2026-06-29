use crate::compat::{
    CompatStatus, CompatibilityResult, MigrationPhase, MigrationStep, StorageKeySpec,
};

/// Stateless validator for pre-upgrade compatibility gates.
pub struct UpgradeValidator;

impl Default for UpgradeValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl UpgradeValidator {
    pub fn new() -> Self {
        Self
    }

    /// Legacy convenience check: removed storage keys are breaking, additions warn.
    pub fn validate_storage_keys(r: &mut CompatibilityResult, v1: &[&str], v2: &[&str]) {
        for k in v1 {
            if !v2.contains(k) {
                r.add_storage(k, CompatStatus::Breaking("removed storage key".into()));
            } else {
                r.add_storage(k, CompatStatus::Compatible);
            }
        }
        for k in v2 {
            if !v1.contains(k) {
                r.add_storage(
                    k,
                    CompatStatus::Warning("new storage key; verify default value and TTL".into()),
                );
            }
        }
    }

    /// Validate key presence plus scope/type compatibility for state preservation.
    pub fn validate_storage_layout(
        r: &mut CompatibilityResult,
        previous: &[StorageKeySpec],
        next: &[StorageKeySpec],
    ) {
        for old in previous {
            match next.iter().find(|new| new.name == old.name) {
                None if old.required => r.add_storage(
                    &old.name,
                    CompatStatus::Breaking("required storage key removed".into()),
                ),
                None => r.add_storage(
                    &old.name,
                    CompatStatus::Warning(
                        "optional storage key removed; migration must clear or ignore legacy data"
                            .into(),
                    ),
                ),
                Some(new) if new.scope != old.scope => r.add_storage(
                    &old.name,
                    CompatStatus::Breaking(
                        "storage scope changed between instance and persistent".into(),
                    ),
                ),
                Some(new) if new.type_hash != old.type_hash => r.add_storage(
                    &old.name,
                    CompatStatus::Breaking(
                        "stored value type changed without migration adapter".into(),
                    ),
                ),
                Some(_) => r.add_storage(&old.name, CompatStatus::Compatible),
            }
        }

        for new in next {
            if !previous.iter().any(|old| old.name == new.name) {
                r.add_storage(
                    &new.name,
                    CompatStatus::Warning(
                        "additive storage key; must be lazily initialized or migrated".into(),
                    ),
                );
            }
        }
    }

    pub fn validate_events(r: &mut CompatibilityResult, v1: &[&str], v2: &[&str]) {
        for e in v1 {
            if !v2.contains(e) {
                r.add_event(
                    e,
                    CompatStatus::Breaking("removed event observed by indexers".into()),
                );
            } else {
                r.add_event(e, CompatStatus::Compatible);
            }
        }
        for e in v2 {
            if !v1.contains(e) {
                r.add_event(
                    e,
                    CompatStatus::Warning("new event; update indexers and monitors".into()),
                );
            }
        }
    }

    pub fn validate_interfaces(r: &mut CompatibilityResult, v1: &[&str], v2: &[&str]) {
        for f in v1 {
            if !v2.contains(f) {
                r.add_interface(f, CompatStatus::Breaking("removed public function".into()));
            } else {
                r.add_interface(f, CompatStatus::Compatible);
            }
        }
        for f in v2 {
            if !v1.contains(f) {
                r.add_interface(
                    f,
                    CompatStatus::Warning(
                        "new public function; document auth and pause behavior".into(),
                    ),
                );
            }
        }
    }

    /// Enforce baseline authorization rules required by the upgrade framework.
    pub fn validate_authorization_rules(
        r: &mut CompatibilityResult,
        rules: &[&str],
        required_rules: &[&str],
    ) {
        for required in required_rules {
            if rules.contains(required) {
                r.add_authorization(required, CompatStatus::Compatible);
            } else {
                r.add_authorization(
                    required,
                    CompatStatus::Breaking("missing mandatory upgrade authorization rule".into()),
                );
            }
        }
    }

    /// Record the canonical migration workflow expected by runbooks and tests.
    pub fn attach_standard_migration_workflow(r: &mut CompatibilityResult) {
        r.add_migration_step(MigrationStep::new(
            "proposal-recorded",
            MigrationPhase::Proposed,
            true,
        ));
        r.add_migration_step(MigrationStep::new(
            "compatibility-report-approved",
            MigrationPhase::Validated,
            true,
        ));
        r.add_migration_step(MigrationStep::new(
            "stored-admin-authorized",
            MigrationPhase::Authorized,
            false,
        ));
        r.add_migration_step(MigrationStep::new(
            "wasm-updated-in-place",
            MigrationPhase::Executed,
            false,
        ));
        r.add_migration_step(MigrationStep::new(
            "post-upgrade-state-verified",
            MigrationPhase::Verified,
            false,
        ));
    }
}
