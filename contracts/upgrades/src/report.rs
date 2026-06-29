use crate::compat::{CompatStatus, CompatibilityResult};

pub struct CompatibilityReport {
    pub result: CompatibilityResult,
}

impl CompatibilityReport {
    pub fn new(result: CompatibilityResult) -> Self {
        Self { result }
    }

    pub fn summary(&self) -> String {
        let mut s = String::new();
        s.push_str("=== Upgrade Compatibility Report ===\n\n");
        s.push_str(&format!(
            "V1: {}\nV2: {}\n\n",
            self.result.v1_hash, self.result.v2_hash
        ));
        if self.result.is_fully_compatible {
            s.push_str("Result: COMPATIBLE\n");
        } else {
            s.push_str(&format!(
                "Result: BLOCKED ({} breaking checks)\n",
                self.result.breaking_count()
            ));
        }
        s.push_str(&format!("Warnings: {}\n", self.result.warning_count()));
        s.push_str(&format!(
            "Migration steps: {}\n",
            self.result.migration.len()
        ));
        s
    }

    pub fn security_notes(&self) -> Vec<String> {
        let mut notes = Vec::new();
        for check in &self.result.authorization {
            match &check.status {
                CompatStatus::Compatible => notes.push(format!("auth ok: {}", check.rule)),
                CompatStatus::Warning(reason) => {
                    notes.push(format!("auth warning: {} ({})", check.rule, reason))
                }
                CompatStatus::Breaking(reason) => {
                    notes.push(format!("auth blocking: {} ({})", check.rule, reason))
                }
            }
        }
        notes
    }
}
