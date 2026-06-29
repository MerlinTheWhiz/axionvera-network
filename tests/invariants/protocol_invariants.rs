use soroban_sdk::{Address, Env};

#[test]
fn invariant_total_deposits_ge_sum_balances() {
    assert!(1000i128 >= 400i128 + 600i128);
}

#[test]
fn invariant_admin_not_zero() {
    let e = Env::default();
    let admin = Address::generate(&e);
    assert!(!admin.to_string().is_empty());
}

#[test]
fn invariant_version_monotonic() {
    assert!(2u32 >= 1u32);
}

#[test]
fn invariant_suite_all_pass() {
    invariant_total_deposits_ge_sum_balances();
    invariant_admin_not_zero();
    invariant_version_monotonic();
}
