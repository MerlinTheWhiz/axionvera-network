#[cfg(test)]
mod tests {
    use crate::{take_snapshot, get_latest_snapshot, get_snapshot_history, MIN_SNAPSHOT_INTERVAL};
    use soroban_sdk::{Env, testutils::Ledger};
    use axionvera_state::{VaultState};

    #[soroban_sdk::contract]
    pub struct TestContract;

    #[soroban_sdk::contractimpl]
    impl TestContract {
        pub fn noop(_e: Env) {}
    }

    #[test]
    fn test_take_snapshot_basic() {
        let e = Env::default();
        let contract_id = e.register(TestContract, ());

        e.as_contract(&contract_id, || {
            e.ledger().set_timestamp(MIN_SNAPSHOT_INTERVAL);

            let snapshot = take_snapshot(&e, None).unwrap();
            assert_eq!(snapshot.metadata.id, 1);
            assert_eq!(snapshot.metadata.timestamp, MIN_SNAPSHOT_INTERVAL);
            assert_eq!(snapshot.vault_state, VaultState::Uninitialized);

            let latest = get_latest_snapshot(&e).unwrap();
            assert_eq!(latest, snapshot);
        });
    }

    #[test]
    fn test_snapshot_interval_enforcement() {
        let e = Env::default();
        let contract_id = e.register(TestContract, ());

        e.as_contract(&contract_id, || {
            e.ledger().set_timestamp(MIN_SNAPSHOT_INTERVAL);
            take_snapshot(&e, None).unwrap();

            e.ledger().set_timestamp(MIN_SNAPSHOT_INTERVAL + 1);
            let result = take_snapshot(&e, None);
            assert!(result.is_err());

            e.ledger().set_timestamp(MIN_SNAPSHOT_INTERVAL * 2);
            let result = take_snapshot(&e, None);
            assert!(result.is_ok());
        });
    }

    #[test]
    fn test_snapshot_history() {
        let e = Env::default();
        let contract_id = e.register(TestContract, ());

        e.as_contract(&contract_id, || {
            for i in 1..=5 {
                e.ledger().set_timestamp(MIN_SNAPSHOT_INTERVAL * i);
                take_snapshot(&e, None).unwrap();
            }

            let history = get_snapshot_history(&e, 3);
            assert_eq!(history.len(), 3);
            assert_eq!(history.get(0).unwrap().metadata.id, 5);
            assert_eq!(history.get(2).unwrap().metadata.id, 3);
        });
    }
}
