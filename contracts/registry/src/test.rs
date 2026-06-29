#![cfg(test)]

use crate::{RegistryContract, RegistryContractClient};
use soroban_sdk::{testutils::Address as _, vec, Address, Env, Symbol};

fn setup() -> (Env, RegistryContractClient<'static>, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(RegistryContract, ());
    let client = RegistryContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialize(&admin);
    (env, client, admin)
}

// ---------------------------------------------------------------------------
// Module registry
// ---------------------------------------------------------------------------

#[test]
fn test_register_and_list() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(RegistryContract, ());
    let client = RegistryContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    let module_addr = Address::generate(&env);
    let name = Symbol::new(&env, "VaultV1");

    client.register_module(&name, &module_addr);

    let listed_addr = client.get_module_address(&name);
    assert_eq!(listed_addr, Some(module_addr.clone()));

    let is_active = client.is_module_active(&module_addr);
    assert_eq!(is_active, true);

    let modules = client.list_modules();
    assert_eq!(modules.len(), 1);
    assert_eq!(modules.get(0).unwrap(), module_addr);
}

#[test]
fn test_set_status() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(RegistryContract, ());
    let client = RegistryContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    let module_addr = Address::generate(&env);
    let name = Symbol::new(&env, "VaultV1");

    client.register_module(&name, &module_addr);

    // Pause module
    client.set_module_status(&module_addr, &false);
    let is_active = client.is_module_active(&module_addr);
    assert_eq!(is_active, false);

    let status = client.get_module_status(&module_addr);
    assert_eq!(status, Some(false));
}

// ---------------------------------------------------------------------------
// Contract index — registration
// ---------------------------------------------------------------------------

#[test]
fn test_index_contract_stores_full_metadata() {
    let (env, client, _admin) = setup();

    let addr = Address::generate(&env);
    let name = Symbol::new(&env, "VaultV2");
    let version = Symbol::new(&env, "v1_0_0");
    let owner = Address::generate(&env);
    let iface = Symbol::new(&env, "IVault");
    let interfaces = vec![&env, iface.clone()];

    client.index_contract(&addr, &name, &version, &owner, &interfaces);

    let info = client.get_contract_info(&addr).expect("info must exist");
    assert_eq!(info.address, addr);
    assert_eq!(info.name, name);
    assert_eq!(info.version, version);
    assert_eq!(info.owner, owner);
    assert_eq!(info.interfaces.len(), 1);
    assert_eq!(info.interfaces.get(0).unwrap(), iface);
    assert_eq!(info.is_active, true);
}

#[test]
#[should_panic(expected = "contract already indexed")]
fn test_index_contract_rejects_duplicate() {
    let (env, client, _admin) = setup();

    let addr = Address::generate(&env);
    let name = Symbol::new(&env, "VaultV2");
    let version = Symbol::new(&env, "v1_0_0");
    let owner = Address::generate(&env);

    client.index_contract(&addr, &name, &version, &owner, &vec![&env]);
    // Second call with same address must panic
    client.index_contract(&addr, &name, &version, &owner, &vec![&env]);
}

// ---------------------------------------------------------------------------
// Contract index — list and lookup
// ---------------------------------------------------------------------------

#[test]
fn test_list_contracts_returns_all_indexed() {
    let (env, client, _admin) = setup();

    let addr1 = Address::generate(&env);
    let addr2 = Address::generate(&env);
    let owner = Address::generate(&env);
    let name1 = Symbol::new(&env, "VaultV1");
    let name2 = Symbol::new(&env, "Treasury");
    let ver = Symbol::new(&env, "v1_0_0");

    client.index_contract(&addr1, &name1, &ver, &owner, &vec![&env]);
    client.index_contract(&addr2, &name2, &ver, &owner, &vec![&env]);

    let listed = client.list_contracts();
    assert_eq!(listed.len(), 2);
    assert!(listed.contains(&addr1));
    assert!(listed.contains(&addr2));
}

#[test]
fn test_get_contract_info_returns_none_for_unknown() {
    let (env, client, _admin) = setup();
    let addr = Address::generate(&env);
    assert_eq!(client.get_contract_info(&addr), None);
}

// ---------------------------------------------------------------------------
// Contract index — interface filtering
// ---------------------------------------------------------------------------

#[test]
fn test_list_contracts_by_interface() {
    let (env, client, _admin) = setup();

    let owner = Address::generate(&env);
    let vault_iface = Symbol::new(&env, "IVault");
    let treasury_iface = Symbol::new(&env, "ITreasury");
    let ver = Symbol::new(&env, "v1_0_0");

    let addr1 = Address::generate(&env);
    client.index_contract(
        &addr1,
        &Symbol::new(&env, "VaultA"),
        &ver,
        &owner,
        &vec![&env, vault_iface.clone()],
    );

    let addr2 = Address::generate(&env);
    client.index_contract(
        &addr2,
        &Symbol::new(&env, "Treasury"),
        &ver,
        &owner,
        &vec![&env, treasury_iface.clone()],
    );

    let addr3 = Address::generate(&env);
    client.index_contract(
        &addr3,
        &Symbol::new(&env, "VaultB"),
        &ver,
        &owner,
        &vec![&env, vault_iface.clone(), treasury_iface.clone()],
    );

    // IVault should match addr1 and addr3
    let vault_matches = client.list_contracts_by_interface(&vault_iface);
    assert_eq!(vault_matches.len(), 2);
    assert!(vault_matches.contains(&addr1));
    assert!(vault_matches.contains(&addr3));

    // ITreasury should match addr2 and addr3
    let treasury_matches = client.list_contracts_by_interface(&treasury_iface);
    assert_eq!(treasury_matches.len(), 2);
    assert!(treasury_matches.contains(&addr2));
    assert!(treasury_matches.contains(&addr3));

    // Unknown interface returns empty
    let unknown = client.list_contracts_by_interface(&Symbol::new(&env, "INone"));
    assert_eq!(unknown.len(), 0);
}

// ---------------------------------------------------------------------------
// Contract index — metadata update
// ---------------------------------------------------------------------------

#[test]
fn test_update_contract_metadata() {
    let (env, client, _admin) = setup();

    let addr = Address::generate(&env);
    let owner = Address::generate(&env);
    let v1 = Symbol::new(&env, "v1_0_0");
    let v2 = Symbol::new(&env, "v2_0_0");
    let iface_a = Symbol::new(&env, "IVault");
    let iface_b = Symbol::new(&env, "IVaultV2");

    client.index_contract(
        &addr,
        &Symbol::new(&env, "Vault"),
        &v1,
        &owner,
        &vec![&env, iface_a.clone()],
    );

    client.update_contract_metadata(&addr, &v2, &vec![&env, iface_a.clone(), iface_b.clone()]);

    let info = client.get_contract_info(&addr).expect("info must exist");
    assert_eq!(info.version, v2);
    assert_eq!(info.interfaces.len(), 2);
    assert!(info.interfaces.contains(&iface_b));
    // Owner and registered_at must be unchanged
    assert_eq!(info.owner, owner);
}

#[test]
#[should_panic(expected = "contract not indexed")]
fn test_update_metadata_panics_for_unknown() {
    let (env, client, _admin) = setup();
    let addr = Address::generate(&env);
    client.update_contract_metadata(&addr, &Symbol::new(&env, "v1_0_0"), &vec![&env]);
}

// ---------------------------------------------------------------------------
// Contract index — status management
// ---------------------------------------------------------------------------

#[test]
fn test_set_contract_status() {
    let (env, client, _admin) = setup();

    let addr = Address::generate(&env);
    let owner = Address::generate(&env);

    client.index_contract(
        &addr,
        &Symbol::new(&env, "Vault"),
        &Symbol::new(&env, "v1_0_0"),
        &owner,
        &vec![&env],
    );

    assert_eq!(client.is_contract_active(&addr), true);

    client.set_contract_status(&addr, &false);
    assert_eq!(client.is_contract_active(&addr), false);

    client.set_contract_status(&addr, &true);
    assert_eq!(client.is_contract_active(&addr), true);
}

#[test]
fn test_is_contract_active_returns_false_for_unknown() {
    let (env, client, _admin) = setup();
    let addr = Address::generate(&env);
    assert_eq!(client.is_contract_active(&addr), false);
}

// ---------------------------------------------------------------------------
// Registry consistency
// ---------------------------------------------------------------------------

#[test]
fn test_metadata_consistent_across_multiple_contracts() {
    let (env, client, _admin) = setup();

    let owner = Address::generate(&env);
    let ver = Symbol::new(&env, "v1_0_0");
    let iface = Symbol::new(&env, "ICore");

    let mut addrs: soroban_sdk::Vec<Address> = vec![&env];
    for i in 0..5u32 {
        let addr = Address::generate(&env);
        let name_str = match i {
            0 => "ContractA",
            1 => "ContractB",
            2 => "ContractC",
            3 => "ContractD",
            _ => "ContractE",
        };
        client.index_contract(
            &addr,
            &Symbol::new(&env, name_str),
            &ver,
            &owner,
            &vec![&env, iface.clone()],
        );
        addrs.push_back(addr);
    }

    // All are listed
    let listed = client.list_contracts();
    assert_eq!(listed.len(), 5);

    // All implement ICore
    let by_iface = client.list_contracts_by_interface(&iface);
    assert_eq!(by_iface.len(), 5);

    // Each info is retrievable and consistent
    for addr in addrs.iter() {
        let info = client.get_contract_info(&addr).expect("must exist");
        assert_eq!(info.is_active, true);
        assert_eq!(info.version, ver);
    }
}
