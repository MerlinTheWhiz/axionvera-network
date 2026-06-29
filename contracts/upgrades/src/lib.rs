pub mod compat;
pub mod report;
pub mod validator;

pub use compat::{
    CompatibilityResult, MigrationPhase, MigrationStep, StorageKeySpec, StorageScope,
};
pub use report::CompatibilityReport;
pub use validator::UpgradeValidator;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compat::CompatStatus;

    #[test]
    fn removed_required_storage_key_blocks_upgrade() {
        let old = [StorageKeySpec::new(
            "Admin",
            StorageScope::Instance,
            "Address",
            true,
        )];
        let new = [];
        let mut result = CompatibilityResult::new("v1", "v2");

        UpgradeValidator::validate_storage_layout(&mut result, &old, &new);

        assert!(!result.is_fully_compatible);
        assert_eq!(result.breaking_count(), 1);
    }

    #[test]
    fn additive_storage_key_is_warning_not_blocking() {
        let old = [StorageKeySpec::new(
            "Admin",
            StorageScope::Instance,
            "Address",
            true,
        )];
        let new = [
            StorageKeySpec::new("Admin", StorageScope::Instance, "Address", true),
            StorageKeySpec::new("UpgradeNonce", StorageScope::Instance, "u64", false),
        ];
        let mut result = CompatibilityResult::new("v1", "v2");

        UpgradeValidator::validate_storage_layout(&mut result, &old, &new);

        assert!(result.is_fully_compatible);
        assert_eq!(result.warning_count(), 1);
    }

    #[test]
    fn storage_scope_change_blocks_upgrade() {
        let old = [StorageKeySpec::new(
            "UserBalance",
            StorageScope::Persistent,
            "i128",
            true,
        )];
        let new = [StorageKeySpec::new(
            "UserBalance",
            StorageScope::Instance,
            "i128",
            true,
        )];
        let mut result = CompatibilityResult::new("v1", "v2");

        UpgradeValidator::validate_storage_layout(&mut result, &old, &new);

        assert!(matches!(
            result.storage[0].status,
            CompatStatus::Breaking(_)
        ));
    }

    #[test]
    fn missing_stored_admin_authorization_blocks_upgrade() {
        let mut result = CompatibilityResult::new("v1", "v2");

        UpgradeValidator::validate_authorization_rules(
            &mut result,
            &["require_auth"],
            &["require_auth", "stored_admin_match"],
        );

        assert!(!result.is_fully_compatible);
        assert_eq!(result.breaking_count(), 1);
    }

    #[test]
    fn standard_workflow_records_validation_and_verification() {
        let mut result = CompatibilityResult::new("v1", "v2");

        UpgradeValidator::attach_standard_migration_workflow(&mut result);

        assert_eq!(result.migration.len(), 5);
        assert_eq!(result.migration[1].phase, MigrationPhase::Validated);
        assert_eq!(result.migration[4].phase, MigrationPhase::Verified);
    }

    #[test]
    fn report_includes_warning_and_migration_counts() {
        let mut result = CompatibilityResult::new("abc", "def");
        UpgradeValidator::validate_events(&mut result, &["deposit"], &["deposit", "upgrade"]);
        UpgradeValidator::attach_standard_migration_workflow(&mut result);
        let report = CompatibilityReport::new(result);
        let summary = report.summary();

        assert!(summary.contains("COMPATIBLE"));
        assert!(summary.contains("Warnings: 1"));
        assert!(summary.contains("Migration steps: 5"));
    }
}
