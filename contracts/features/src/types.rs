use soroban_sdk::{contracttype, Symbol};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FeatureFlag {
    pub name: Symbol,
    pub enabled: bool,
    pub rollout_pct: u32,
    pub registered_at: u64,
    pub enabled_at: Option<u64>,
}

pub const MAX_ROLLOUT_PCT: u32 = 100;
pub const MAX_FEATURES: u32 = 64;
