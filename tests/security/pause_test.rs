#[cfg(test)]
mod tests {
    use soroban_sdk::{Address, Env};
    use axionvera_security::EmergencyPause;

    #[test]
    fn test_pause_unpause() {
        let e = Env::default();
        let contract = e.register_contract(None, EmergencyPause);
        let client = EmergencyPauseClient::new(&e, &contract);
        let admin = Address::generate(&e);
        client.init(&admin);
        assert!(!client.is_paused());
        client.pause(&admin);
        assert!(client.is_paused());
        client.unpause(&admin);
        assert!(!client.is_paused());
    }
}
