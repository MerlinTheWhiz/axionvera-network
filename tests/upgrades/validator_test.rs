#[cfg(test)]
mod tests {
    use axionvera_upgrades::compat::CompatibilityResult;
    use axionvera_upgrades::validator::UpgradeValidator;
    use axionvera_upgrades::report::CompatibilityReport;

    #[test]
    fn test_fully_compatible() {
        let mut r = CompatibilityResult::new("v1", "v2");
        UpgradeValidator::validate_storage_keys(&mut r, &["Admin"], &["Admin"]);
        assert!(r.is_fully_compatible);
    }

    #[test]
    fn test_removed_key_breaking() {
        let mut r = CompatibilityResult::new("v1", "v2");
        UpgradeValidator::validate_storage_keys(&mut r, &["Admin", "Old"], &["Admin"]);
        assert!(!r.is_fully_compatible);
    }

    #[test]
    fn test_new_event_warning() {
        let mut r = CompatibilityResult::new("v1", "v2");
        UpgradeValidator::validate_events(&mut r, &["deposit"], &["deposit", "new"]);
        assert!(r.is_fully_compatible);
    }

    #[test]
    fn test_report() {
        let mut r = CompatibilityResult::new("abc", "def");
        UpgradeValidator::validate_storage_keys(&mut r, &["Admin"], &["Admin"]);
        let report = CompatibilityReport::new(r);
        assert!(report.summary().contains("COMPATIBLE"));
    }
}
