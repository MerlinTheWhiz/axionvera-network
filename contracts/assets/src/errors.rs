use soroban_sdk::contracterror;

/// All errors that can be returned by the asset registry contract.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum AssetRegistryError {
    /// `initialize` was called on an already-initialised contract.
    AlreadyInitialized = 1,
    /// A state-reading function was called before `initialize`.
    NotInitialized = 2,
    /// Caller does not have admin authority.
    Unauthorized = 3,
    /// The asset address has already been registered.
    AssetAlreadyRegistered = 4,
    /// The requested asset is not in the registry.
    AssetNotFound = 5,
    /// Asset name is empty or exceeds 32 bytes.
    InvalidAssetName = 6,
    /// Asset symbol is empty or exceeds 12 bytes.
    InvalidAssetSymbol = 7,
    /// Decimal places value exceeds 18.
    InvalidDecimals = 8,
    /// `accept_admin` called when no transfer is pending.
    NoPendingAdmin = 9,
    /// A write function was called while the contract is paused.
    ContractPaused = 10,
}
