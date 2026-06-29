#![no_std]

mod storage;
pub mod types;
#[cfg(test)]
mod test;

use soroban_sdk::{contract, contractimpl, Address, Env, Symbol, Vec};
use axionvera_events::{
    EVENT_VERSION, PROTOCOL_REGISTRY,
    ACT_MOD_REGISTER, ACT_MOD_STATUS_UPDATE, ACT_CTRT_INDEX, ACT_CTRT_META,
    ModuleRegisteredEvent, ModuleStatusChangedEvent,
    ContractIndexedEvent, ContractMetadataUpdatedEvent,
};

use crate::types::ContractInfo;

#[contract]
pub struct RegistryContract;

#[contractimpl]
impl RegistryContract {
    // -----------------------------------------------------------------------
    // Lifecycle
    // -----------------------------------------------------------------------

    pub fn initialize(e: Env, admin: Address) {
        if storage::is_initialized(&e) {
            panic!("already initialized");
        }
        admin.require_auth();
        storage::set_admin(&e, &admin);
        e.storage().instance().set(&storage::DataKey::Initialized, &true);
    }

    // -----------------------------------------------------------------------
    // Module registry (name → address mapping, simple active/inactive)
    // -----------------------------------------------------------------------

    pub fn register_module(e: Env, name: Symbol, module_address: Address) {
        let admin = storage::get_admin(&e);
        admin.require_auth();

        if storage::has_module_name(&e, name.clone()) {
            panic!("module name already registered");
        }

        if storage::has_module_address(&e, &module_address) {
            panic!("module address already registered");
        }

        storage::set_module_address(&e, name.clone(), &module_address);
        storage::set_module_status(&e, &module_address, true);
        storage::add_to_all_modules(&e, &module_address);

        let timestamp = e.ledger().timestamp();
        e.events().publish(
            (PROTOCOL_REGISTRY, ACT_MOD_REGISTER),
            ModuleRegisteredEvent {
                event_version: EVENT_VERSION,
                admin,
                name,
                module_address,
                timestamp,
            },
        );
    }

    pub fn set_module_status(e: Env, module_address: Address, is_active: bool) {
        let admin = storage::get_admin(&e);
        admin.require_auth();

        storage::set_module_status(&e, &module_address, is_active);

        let timestamp = e.ledger().timestamp();
        e.events().publish(
            (PROTOCOL_REGISTRY, ACT_MOD_STATUS_UPDATE),
            ModuleStatusChangedEvent {
                event_version: EVENT_VERSION,
                admin,
                module_address,
                is_active,
                timestamp,
            },
        );
    }

    pub fn get_module_address(e: Env, name: Symbol) -> Option<Address> {
        storage::get_module_address(&e, name)
    }

    pub fn get_module_status(e: Env, module_address: Address) -> Option<bool> {
        storage::get_module_status(&e, &module_address)
    }

    pub fn is_module_active(e: Env, module_address: Address) -> bool {
        storage::get_module_status(&e, &module_address).unwrap_or(false)
    }

    pub fn list_modules(e: Env) -> Vec<Address> {
        storage::get_all_modules(&e)
    }

    // -----------------------------------------------------------------------
    // Contract index (full metadata: version, interfaces, owner, deployment)
    // -----------------------------------------------------------------------

    /// Registers a deployed protocol contract with full metadata.
    ///
    /// - `contract_address`: The deployed contract's on-chain address.
    /// - `name`: Human-readable identifier (e.g. `VaultV2`).
    /// - `version`: Implementation version (e.g. `v1_0_0`).
    /// - `owner`: Deploying / owning address.
    /// - `interfaces`: List of interface symbols this contract satisfies.
    ///
    /// Panics if the address is already indexed. Emits `ContractIndexedEvent`.
    pub fn index_contract(
        e: Env,
        contract_address: Address,
        name: Symbol,
        version: Symbol,
        owner: Address,
        interfaces: Vec<Symbol>,
    ) {
        let admin = storage::get_admin(&e);
        admin.require_auth();

        if storage::has_contract(&e, &contract_address) {
            panic!("contract already indexed");
        }

        let registered_at = e.ledger().timestamp();
        let info = ContractInfo {
            address: contract_address.clone(),
            name: name.clone(),
            version: version.clone(),
            owner,
            interfaces,
            registered_at,
            is_active: true,
        };

        storage::set_contract_info(&e, &contract_address, &info);
        storage::add_to_all_contracts(&e, &contract_address);

        e.events().publish(
            (PROTOCOL_REGISTRY, ACT_CTRT_INDEX),
            ContractIndexedEvent {
                event_version: EVENT_VERSION,
                registered_by: admin,
                contract_address,
                name,
                version,
                timestamp: registered_at,
            },
        );
    }

    /// Returns full metadata for an indexed contract, or `None` if not found.
    pub fn get_contract_info(e: Env, contract_address: Address) -> Option<ContractInfo> {
        storage::get_contract_info(&e, &contract_address)
    }

    /// Updates the version and interface list for an already-indexed contract.
    ///
    /// Panics if the contract has not been indexed. Emits `ContractMetadataUpdatedEvent`.
    pub fn update_contract_metadata(
        e: Env,
        contract_address: Address,
        version: Symbol,
        interfaces: Vec<Symbol>,
    ) {
        let admin = storage::get_admin(&e);
        admin.require_auth();

        let mut info = storage::get_contract_info(&e, &contract_address)
            .unwrap_or_else(|| panic!("contract not indexed"));

        info.version = version.clone();
        info.interfaces = interfaces;
        storage::set_contract_info(&e, &contract_address, &info);

        let timestamp = e.ledger().timestamp();
        e.events().publish(
            (PROTOCOL_REGISTRY, ACT_CTRT_META),
            ContractMetadataUpdatedEvent {
                event_version: EVENT_VERSION,
                updated_by: admin,
                contract_address,
                version,
                timestamp,
            },
        );
    }

    /// Sets the active status of an indexed contract.
    pub fn set_contract_status(e: Env, contract_address: Address, is_active: bool) {
        let admin = storage::get_admin(&e);
        admin.require_auth();

        let mut info = storage::get_contract_info(&e, &contract_address)
            .unwrap_or_else(|| panic!("contract not indexed"));

        info.is_active = is_active;
        storage::set_contract_info(&e, &contract_address, &info);
    }

    /// Returns all indexed contract addresses in registration order.
    pub fn list_contracts(e: Env) -> Vec<Address> {
        storage::get_all_contracts(&e)
    }

    /// Returns the addresses of all indexed contracts that implement the given interface.
    pub fn list_contracts_by_interface(e: Env, interface: Symbol) -> Vec<Address> {
        let all = storage::get_all_contracts(&e);
        let mut matches: Vec<Address> = Vec::new(&e);
        for addr in all.iter() {
            if let Some(info) = storage::get_contract_info(&e, &addr) {
                if info.interfaces.contains(&interface) {
                    matches.push_back(addr);
                }
            }
        }
        matches
    }

    /// Returns `true` if the contract is indexed and currently active.
    pub fn is_contract_active(e: Env, contract_address: Address) -> bool {
        storage::get_contract_info(&e, &contract_address)
            .map(|i| i.is_active)
            .unwrap_or(false)
    }
}
