#![cfg(test)]

use super::*;
use soroban_sdk::testutils::{Events};
use soroban_sdk::{vec, Env, IntoVal};

#[test]
fn test_metrics_aggregation() {
    let e = Env::default();
    e.mock_all_auths();

    let contract_id = e.register(MetricsContract, ());
    let client = MetricsContractClient::new(&e, &contract_id);

    // Initial metrics should be zero
    let initial = client.get_metrics();
    assert_eq!(initial.total_value_locked, 0);
    assert_eq!(initial.active_users, 0);

    // Update metrics (even with no data in other contracts, it should still work)
    let updated = client.update_metrics();
    assert_eq!(updated.total_value_locked, 0);
    assert_eq!(updated.active_users, 0);
    assert_eq!(updated.last_updated, 0);

    let snapshots = client.get_snapshots();
    assert_eq!(snapshots.len(), 1);
}

#[test]
fn test_metrics_event_emission() {
    let e = Env::default();
    e.mock_all_auths();

    let contract_id = e.register(MetricsContract, ());
    let client = MetricsContractClient::new(&e, &contract_id);

    client.update_metrics();

    let events = e.events().all();
    let last_event = events.last().unwrap();

    // last_event is a tuple (Address, Vec<Val>, Val)
    // Topics are the second element (index 1)
    assert_eq!(
        last_event.1,
        vec![&e, symbol_short!("metrics").into_val(&e), symbol_short!("updated").into_val(&e)]
    );
}
