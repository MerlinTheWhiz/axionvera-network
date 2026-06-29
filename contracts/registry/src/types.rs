use soroban_sdk::{contracttype, Address, Symbol, Vec};

/// Full metadata stored per indexed protocol contract.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractInfo {
    /// On-chain address of the deployed contract.
    pub address: Address,
    /// Human-readable name (e.g. `VaultV2`).
    pub name: Symbol,
    /// Implementation version string (e.g. `v1_0_0`).
    pub version: Symbol,
    /// Owner / deployer of the contract.
    pub owner: Address,
    /// List of interface identifiers this contract implements.
    pub interfaces: Vec<Symbol>,
    /// Ledger timestamp at registration time.
    pub registered_at: u64,
    /// Whether the contract is currently considered active.
    pub is_active: bool,
}
