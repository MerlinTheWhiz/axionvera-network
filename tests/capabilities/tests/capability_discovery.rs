use axionvera_capabilities::{
    CapabilitiesContract, Capability, CapabilityError, ProtocolMetadata, Version,
};
use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, Events},
    xdr::ToXdr,
    Address, Env, Vec,
};

fn setup(e: &Env) -> (Address, Address) {
    e.mock_all_auths();
    let admin = Address::generate(e);
    let contract_id = e.register_contract(None, CapabilitiesContract);
    e.as_contract(&contract_id, || {
        CapabilitiesContract::initialize(e.clone(), admin.clone(), Version { major: 1, minor: 0, patch: 0 })
    })
    .unwrap();
    (contract_id, admin)
}

fn register_swap(e: &Env, contract_id: &Address, admin: &Address) {
    let cap = Capability {
        id: symbol_short!("swap"),
        version: Version { major: 1, minor: 0, patch: 0 },
        name: symbol_short!("swap"),
        description: symbol_short!("desc"),
        interfaces: Vec::from_array(e, [symbol_short!("ISwap")]),
    };
    e.as_contract(contract_id, || {
        CapabilitiesContract::register_capability(e.clone(), admin.clone(), cap)
    })
    .unwrap();
}

#[test]
fn test_initialize_and_metadata() {
    let e = Env::default();
    let (contract_id, _admin) = setup(&e);
    let meta: ProtocolMetadata = e.as_contract(&contract_id, || {
        CapabilitiesContract::query_metadata(e.clone())
    });
    assert_eq!(meta.name, axionvera_capabilities::PROTOCOL_NAME);
    assert_eq!(meta.protocol_version, Version { major: 1, minor: 0, patch: 0 });
    assert_eq!(meta.capabilities_count, 0);
}

#[test]
fn test_double_initialize_fails() {
    let e = Env::default();
    e.mock_all_auths();
    let admin = Address::generate(&e);
    let contract_id = e.register_contract(None, CapabilitiesContract);
    let r1 = e.as_contract(&contract_id, || {
        CapabilitiesContract::initialize(e.clone(), admin.clone(), Version { major: 1, minor: 0, patch: 0 })
    });
    assert!(r1.is_ok());
    let r2 = e.as_contract(&contract_id, || {
        CapabilitiesContract::initialize(e.clone(), admin.clone(), Version { major: 1, minor: 0, patch: 0 })
    });
    assert_eq!(r2, Err(CapabilityError::AlreadyInitialized));
}

#[test]
fn test_register_and_query_capability() {
    let e = Env::default();
    let (contract_id, admin) = setup(&e);
    register_swap(&e, &contract_id, &admin);

    let ids: Vec<soroban_sdk::Symbol> = e.as_contract(&contract_id, || {
        CapabilitiesContract::query_capabilities(e.clone())
    });
    assert_eq!(ids.len(), 1);
    assert_eq!(ids.first().unwrap(), symbol_short!("swap"));

    let stored: Capability = e.as_contract(&contract_id, || {
        CapabilitiesContract::query_capability(e.clone(), symbol_short!("swap"))
    })
    .unwrap();
    assert_eq!(stored.id, symbol_short!("swap"));
    assert_eq!(stored.version.major, 1);
}

#[test]
fn test_register_duplicate_fails() {
    let e = Env::default();
    let (contract_id, admin) = setup(&e);
    register_swap(&e, &contract_id, &admin);

    let cap = Capability {
        id: symbol_short!("swap"),
        version: Version { major: 1, minor: 0, patch: 0 },
        name: symbol_short!("swap"),
        description: symbol_short!("desc"),
        interfaces: Vec::new(&e),
    };
    let result: Result<(), CapabilityError> = e.as_contract(&contract_id, || {
        CapabilitiesContract::register_capability(e.clone(), admin.clone(), cap.clone())
    });
    assert_eq!(result, Err(CapabilityError::DuplicateCapability));
}

#[test]
fn test_unauthorized_fails() {
    let e = Env::default();
    let admin = Address::generate(&e);
    let contract_id = e.register_contract(None, CapabilitiesContract);
    e.as_contract(&contract_id, || {
        CapabilitiesContract::initialize(e.clone(), admin.clone(), Version { major: 1, minor: 0, patch: 0 })
    })
    .unwrap();

    let stranger = Address::generate(&e);
    let cap = Capability {
        id: symbol_short!("swap"),
        version: Version { major: 1, minor: 0, patch: 0 },
        name: symbol_short!("swap"),
        description: symbol_short!("desc"),
        interfaces: Vec::new(&e),
    };
    let result: Result<(), CapabilityError> = e.as_contract(&contract_id, || {
        CapabilitiesContract::register_capability(e.clone(), stranger.clone(), cap.clone())
    });
    assert_eq!(result, Err(CapabilityError::Unauthorized));
}

#[test]
fn test_supports_capability_version_check() {
    let e = Env::default();
    let (contract_id, admin) = setup(&e);

    let cap = Capability {
        id: symbol_short!("swap"),
        version: Version { major: 2, minor: 1, patch: 3 },
        name: symbol_short!("swap"),
        description: symbol_short!("desc"),
        interfaces: Vec::new(&e),
    };
    e.as_contract(&contract_id, || {
        CapabilitiesContract::register_capability(e.clone(), admin.clone(), cap)
    }).unwrap();

    let r1: bool = e.as_contract(&contract_id, || CapabilitiesContract::supports_capability(e.clone(), symbol_short!("swap"), 2, 1, 0));
    assert!(r1);
    let r2: bool = e.as_contract(&contract_id, || CapabilitiesContract::supports_capability(e.clone(), symbol_short!("swap"), 2, 0, 0));
    assert!(r2);
    let r3: bool = e.as_contract(&contract_id, || CapabilitiesContract::supports_capability(e.clone(), symbol_short!("swap"), 3, 0, 0));
    assert!(!r3);
    let r4: bool = e.as_contract(&contract_id, || CapabilitiesContract::supports_capability(e.clone(), symbol_short!("swap"), 2, 2, 0));
    assert!(!r4);
    let r5: bool = e.as_contract(&contract_id, || CapabilitiesContract::supports_capability(e.clone(), symbol_short!("missing"), 1, 0, 0));
    assert!(!r5);
}

#[test]
fn test_supports_protocol_version() {
    let e = Env::default();
    let (contract_id, _admin) = setup(&e);

    let r1: bool = e.as_contract(&contract_id, || CapabilitiesContract::supports_protocol_version(e.clone(), 1, 0));
    assert!(r1);
    let r2: bool = e.as_contract(&contract_id, || CapabilitiesContract::supports_protocol_version(e.clone(), 1, 1));
    assert!(!r2);
    let r3: bool = e.as_contract(&contract_id, || CapabilitiesContract::supports_protocol_version(e.clone(), 2, 0));
    assert!(!r3);
}

#[test]
fn test_metadata_after_registration() {
    let e = Env::default();
    let (contract_id, admin) = setup(&e);
    register_swap(&e, &contract_id, &admin);

    let meta: ProtocolMetadata = e.as_contract(&contract_id, || {
        CapabilitiesContract::query_metadata(e.clone())
    });
    assert_eq!(meta.capabilities_count, 1);
    assert_eq!(meta.interfaces_count, 1);
}

#[test]
fn test_events_emitted_on_registration() {
    let e = Env::default();
    let (contract_id, admin) = setup(&e);

    let cap = Capability {
        id: symbol_short!("stake"),
        version: Version { major: 1, minor: 0, patch: 0 },
        name: symbol_short!("stake"),
        description: symbol_short!("desc"),
        interfaces: Vec::new(&e),
    };
    e.as_contract(&contract_id, || {
        CapabilitiesContract::register_capability(e.clone(), admin.clone(), cap)
    }).unwrap();

    let events = e.events().all();
    let last = events.last().unwrap();
    assert_eq!(last.1.len(), 2);
    assert_eq!(
        last.1.get(0).unwrap().clone().to_xdr(&e),
        axionvera_events::PROTOCOL_CAPABILITIES.to_xdr(&e),
    );
    assert_eq!(
        last.1.get(1).unwrap().clone().to_xdr(&e),
        axionvera_events::ACT_CAP_REGISTERED.to_xdr(&e),
    );
}

#[test]
fn test_update_and_remove_capability() {
    let e = Env::default();
    let (contract_id, admin) = setup(&e);
    register_swap(&e, &contract_id, &admin);

    e.as_contract(&contract_id, || {
        CapabilitiesContract::update_capability_version(e.clone(), admin.clone(), symbol_short!("swap"), Version { major: 2, minor: 0, patch: 0 })
    }).unwrap();

    let cap: Capability = e.as_contract(&contract_id, || {
        CapabilitiesContract::query_capability(e.clone(), symbol_short!("swap"))
    }).unwrap();
    assert_eq!(cap.version.major, 2);

    e.as_contract(&contract_id, || {
        CapabilitiesContract::remove_capability(e.clone(), admin.clone(), symbol_short!("swap"))
    }).unwrap();

    let is_none: bool = e.as_contract(&contract_id, || {
        CapabilitiesContract::query_capability(e.clone(), symbol_short!("swap")).is_none()
    });
    assert!(is_none);

    let ids: Vec<soroban_sdk::Symbol> = e.as_contract(&contract_id, || {
        CapabilitiesContract::query_capabilities(e.clone())
    });
    assert_eq!(ids.len(), 0);
}

#[test]
fn test_query_interfaces() {
    let e = Env::default();
    let (contract_id, admin) = setup(&e);

    let cap = Capability {
        id: symbol_short!("vault"),
        version: Version { major: 1, minor: 0, patch: 0 },
        name: symbol_short!("vault"),
        description: symbol_short!("desc"),
        interfaces: Vec::from_array(&e, [symbol_short!("IVault"), symbol_short!("IStaking")]),
    };
    e.as_contract(&contract_id, || {
        CapabilitiesContract::register_capability(e.clone(), admin.clone(), cap)
    }).unwrap();

    let ifaces: Vec<soroban_sdk::Symbol> = e.as_contract(&contract_id, || {
        CapabilitiesContract::query_interfaces(e.clone())
    });
    assert_eq!(ifaces.len(), 2);
    assert!(ifaces.contains(&symbol_short!("IVault")));
    assert!(ifaces.contains(&symbol_short!("IStaking")));
}
