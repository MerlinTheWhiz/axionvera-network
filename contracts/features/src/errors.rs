use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum FeatureError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    FeatureAlreadyRegistered = 4,
    FeatureNotFound = 5,
    InvalidRolloutPct = 6,
    FeatureLimitReached = 7,
    NoPendingAdmin = 8,
    ContractPaused = 9,
}
