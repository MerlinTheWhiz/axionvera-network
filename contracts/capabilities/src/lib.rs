#![no_std]

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, Map, Symbol, Vec};

use axionvera_events as events;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

pub const PROTOCOL_NAME: Symbol = symbol_short!("AxionVera");

/// Semantic version.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

/// A single capability offered by the protocol.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Capability {
    pub id: Symbol,
    pub version: Version,
    pub name: Symbol,
    pub description: Symbol,
    pub interfaces: Vec<Symbol>,
}

/// Protocol-level metadata.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProtocolMetadata {
    pub name: Symbol,
    pub protocol_version: Version,
    pub capabilities_count: u32,
    pub interfaces_count: u32,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum CapabilityError {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    CapabilityNotFound = 3,
    Unauthorized = 4,
    DuplicateCapability = 5,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Initialized,
    ProtocolVersion,
    Capability(Symbol),
    CapabilityList,
    Interface(Symbol),
    InterfaceList,
    Admins,
}

// ---------------------------------------------------------------------------
// Contract
// ---------------------------------------------------------------------------

#[contract]
pub struct CapabilitiesContract;

#[contractimpl]
impl CapabilitiesContract {
    pub fn initialize(e: Env, admin: Address, protocol_version: Version) -> Result<(), CapabilityError> {
        if e.storage().instance().has(&DataKey::Initialized) {
            return Err(CapabilityError::AlreadyInitialized);
        }
        e.storage().instance().set(&DataKey::Initialized, &true);
        e.storage().instance().set(&DataKey::ProtocolVersion, &protocol_version);
        let mut admins: Map<Address, bool> = Map::new(&e);
        admins.set(admin, true);
        e.storage().instance().set(&DataKey::Admins, &admins);
        e.storage().instance().extend_ttl(518400, 518400);
        Ok(())
    }

    pub fn register_capability(e: Env, admin: Address, cap: Capability) -> Result<(), CapabilityError> {
        Self::require_admin(&e, &admin)?;
        let mut list: Vec<Symbol> = e.storage().instance().get(&DataKey::CapabilityList).unwrap_or_else(|| Vec::new(&e));
        if list.contains(&cap.id) {
            return Err(CapabilityError::DuplicateCapability);
        }
        list.push_back(cap.id.clone());
        e.storage().instance().set(&DataKey::CapabilityList, &list);
        e.storage().instance().set(&DataKey::Capability(cap.id.clone()), &cap);
        for iface in cap.interfaces.iter() {
            let mut iface_list: Vec<Symbol> = e.storage().instance().get(&DataKey::InterfaceList).unwrap_or_else(|| Vec::new(&e));
            if !iface_list.contains(&iface) {
                iface_list.push_back(iface.clone());
                e.storage().instance().set(&DataKey::InterfaceList, &iface_list);
            }
        }
        e.storage().instance().extend_ttl(518400, 518400);
        e.events().publish((
            events::PROTOCOL_CAPABILITIES,
            events::ACT_CAP_REGISTERED,
        ), (
            admin,
            cap.id,
            cap.version.major,
            cap.version.minor,
            cap.version.patch,
            e.ledger().timestamp(),
        ));
        Ok(())
    }

    pub fn update_capability_version(e: Env, admin: Address, id: Symbol, new_version: Version) -> Result<(), CapabilityError> {
        Self::require_admin(&e, &admin)?;
        let mut cap: Capability = e.storage().instance().get(&DataKey::Capability(id.clone())).ok_or(CapabilityError::CapabilityNotFound)?;
        let old_version = cap.version.clone();
        cap.version = new_version.clone();
        e.storage().instance().set(&DataKey::Capability(id.clone()), &cap);
        e.storage().instance().extend_ttl(518400, 518400);
        e.events().publish((
            events::PROTOCOL_CAPABILITIES,
            events::ACT_CAP_UPDATED,
        ), (
            admin,
            id,
            old_version.major,
            old_version.minor,
            old_version.patch,
            new_version.major,
            new_version.minor,
            new_version.patch,
            e.ledger().timestamp(),
        ));
        Ok(())
    }

    pub fn remove_capability(e: Env, admin: Address, id: Symbol) -> Result<(), CapabilityError> {
        Self::require_admin(&e, &admin)?;
        if !e.storage().instance().has(&DataKey::Capability(id.clone())) {
            return Err(CapabilityError::CapabilityNotFound);
        }
        e.storage().instance().remove(&DataKey::Capability(id.clone()));
        let mut list: Vec<Symbol> = e.storage().instance().get(&DataKey::CapabilityList).unwrap_or_else(|| Vec::new(&e));
        if let Some(pos) = list.first_index_of(&id) {
            list.remove(pos as u32);
            e.storage().instance().set(&DataKey::CapabilityList, &list);
        }
        e.storage().instance().extend_ttl(518400, 518400);
        e.events().publish((
            events::PROTOCOL_CAPABILITIES,
            events::ACT_CAP_REMOVED,
        ), (
            admin,
            id,
            e.ledger().timestamp(),
        ));
        Ok(())
    }

    pub fn query_capabilities(e: Env) -> Vec<Symbol> {
        e.storage().instance().get(&DataKey::CapabilityList).unwrap_or_else(|| Vec::new(&e))
    }

    pub fn query_capability(e: Env, id: Symbol) -> Option<Capability> {
        e.storage().instance().get(&DataKey::Capability(id))
    }

    pub fn supports_capability(e: Env, id: Symbol, major: u32, minor: u32, patch: u32) -> bool {
        let cap = match e.storage().instance().get::<_, Capability>(&DataKey::Capability(id)) {
            Some(c) => c,
            None => return false,
        };
        if cap.version.major != major { return false; }
        if cap.version.minor < minor { return false; }
        if cap.version.minor == minor && cap.version.patch < patch { return false; }
        true
    }

    pub fn query_interfaces(e: Env) -> Vec<Symbol> {
        e.storage().instance().get(&DataKey::InterfaceList).unwrap_or_else(|| Vec::new(&e))
    }

    pub fn query_metadata(e: Env) -> ProtocolMetadata {
        let version: Version = e.storage().instance().get(&DataKey::ProtocolVersion).unwrap_or(Version { major: 0, minor: 0, patch: 0 });
        let caps_count = e.storage().instance().get::<_, Vec<Symbol>>(&DataKey::CapabilityList).map(|l| l.len()).unwrap_or(0);
        let ifaces_count = e.storage().instance().get::<_, Vec<Symbol>>(&DataKey::InterfaceList).map(|l| l.len()).unwrap_or(0);
        ProtocolMetadata {
            name: PROTOCOL_NAME,
            protocol_version: version,
            capabilities_count: caps_count,
            interfaces_count: ifaces_count,
        }
    }

    pub fn protocol_version(e: Env) -> Version {
        e.storage().instance().get(&DataKey::ProtocolVersion).unwrap_or(Version { major: 0, minor: 0, patch: 0 })
    }

    pub fn supports_protocol_version(e: Env, major: u32, minor: u32) -> bool {
        let v: Version = e.storage().instance().get(&DataKey::ProtocolVersion).unwrap_or(Version { major: 0, minor: 0, patch: 0 });
        if v.major != major { return false; }
        v.minor >= minor
    }

    fn require_admin(e: &Env, addr: &Address) -> Result<(), CapabilityError> {
        let admins: Map<Address, bool> = e.storage().instance().get(&DataKey::Admins).unwrap_or_else(|| Map::new(e));
        if !admins.get(addr.clone()).unwrap_or(false) {
            return Err(CapabilityError::Unauthorized);
        }
        Ok(())
    }
}
