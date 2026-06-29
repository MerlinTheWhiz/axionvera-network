use crate::compat::{CompatStatus, CompatibilityResult};

pub struct CompatibilityReport { pub result: CompatibilityResult }

impl CompatibilityReport {
    pub fn new(result: CompatibilityResult) -> Self { Self { result } }
    pub fn summary(&self) -> String {
        let mut s = String::new();
        s.push_str("=== Upgrade Compatibility Report ===\n\n");
        s.push_str(&format!("V1: {}\nV2: {}\n\n", self.result.v1_hash, self.result.v2_hash));
        if self.result.is_fully_compatible { s.push_str("Result: COMPATIBLE\n"); }
        else { s.push_str(&format!("Breaking: {}\n", self.result.breaking_count())); }
        s
    }
}
