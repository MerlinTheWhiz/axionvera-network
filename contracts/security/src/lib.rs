#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, symbol_short};

#[contracttype]
#[derive(Clone)]
enum DataKey {
    Admin,
    Paused,
}

const INSTANCE_TTL: u32 = 518400;

#[contract]
pub struct EmergencyPause;

#[contractimpl]
impl EmergencyPause {
    pub fn init(e: Env, admin: Address) {
        if e.storage().instance().has(&DataKey::Admin) { panic!("Already initialized"); }
        e.storage().instance().set(&DataKey::Admin, &admin);
        e.storage().instance().set(&DataKey::Paused, &false);
        e.storage().instance().extend_ttl(INSTANCE_TTL, INSTANCE_TTL);
    }

    pub fn pause(e: Env, caller: Address) {
        caller.require_auth();
        let admin: Address = e.storage().instance().get(&DataKey::Admin).unwrap();
        if caller != admin { panic!("Not authorized"); }
        e.storage().instance().set(&DataKey::Paused, &true);
        e.events().publish((symbol_short!("pause"),), symbol_short!("paused"));
    }

    pub fn unpause(e: Env, caller: Address) {
        caller.require_auth();
        let admin: Address = e.storage().instance().get(&DataKey::Admin).unwrap();
        if caller != admin { panic!("Not authorized"); }
        e.storage().instance().set(&DataKey::Paused, &false);
        e.events().publish((symbol_short!("pause"),), symbol_short!("unpaused"));
    }

    pub fn is_paused(e: Env) -> bool {
        e.storage().instance().get(&DataKey::Paused).unwrap_or(false)
    }

    pub fn admin(e: Env) -> Address {
        e.storage().instance().get(&DataKey::Admin).unwrap()
    }
}
