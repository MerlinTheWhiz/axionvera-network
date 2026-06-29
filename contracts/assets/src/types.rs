use soroban_sdk::{contracttype, Bytes};

// ---------------------------------------------------------------------------
// Validation bounds
// ---------------------------------------------------------------------------

/// Maximum byte length of an asset name.
pub const MAX_NAME_LEN: u32 = 32;

/// Maximum byte length of an asset symbol (e.g. "USDC", "XLM").
pub const MAX_SYMBOL_LEN: u32 = 12;

/// Maximum number of decimal places for a whitelisted asset.
pub const MAX_DECIMALS: u32 = 18;

// ---------------------------------------------------------------------------
// AssetInfo — metadata stored per whitelisted asset
// ---------------------------------------------------------------------------

/// On-chain metadata for a single whitelisted asset.
///
/// Stored in persistent ledger storage, keyed by the asset's contract address.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssetInfo {
    /// Human-readable asset name (e.g. "USD Coin"). Max 32 bytes.
    pub name: Bytes,

    /// Short ticker symbol (e.g. "USDC"). Max 12 bytes.
    pub symbol: Bytes,

    /// Number of decimal places used by the asset (0–18).
    pub decimals: u32,

    /// Whether the asset is currently active. Inactive assets fail whitelist checks.
    pub is_active: bool,

    /// Ledger timestamp at the time of initial registration.
    pub registered_at: u64,
}
