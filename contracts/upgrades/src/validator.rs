use crate::compat::{CompatStatus, CompatibilityResult};

pub struct UpgradeValidator;

impl UpgradeValidator {
    pub fn new() -> Self { Self }
    pub fn validate_storage_keys(r: &mut CompatibilityResult, v1: &[&str], v2: &[&str]) {
        for k in v1 { if !v2.contains(k) { r.add_storage(k, CompatStatus::Breaking("removed".into())); } else { r.add_storage(k, CompatStatus::Compatible); } }
        for k in v2 { if !v1.contains(k) { r.add_storage(k, CompatStatus::Warning("new".into())); } }
    }
    pub fn validate_events(r: &mut CompatibilityResult, v1: &[&str], v2: &[&str]) {
        for e in v1 { if !v2.contains(e) { r.add_event(e, CompatStatus::Breaking("removed".into())); } else { r.add_event(e, CompatStatus::Compatible); } }
        for e in v2 { if !v1.contains(e) { r.add_event(e, CompatStatus::Warning("new".into())); } }
    }
    pub fn validate_interfaces(r: &mut CompatibilityResult, v1: &[&str], v2: &[&str]) {
        for f in v1 { if !v2.contains(f) { r.add_interface(f, CompatStatus::Breaking("removed".into())); } else { r.add_interface(f, CompatStatus::Compatible); } }
        for f in v2 { if !v1.contains(f) { r.add_interface(f, CompatStatus::Warning("new".into())); } }
    }
}
