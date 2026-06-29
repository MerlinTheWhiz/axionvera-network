use axionvera_auth::{AccessPolicy, PolicyViolation};
use axionvera_security::{Authenticated, MatchAddress, PredicatePolicy};
use axionvera_vault_contract::{errors::VaultError, VaultContract, VaultContractClient};
use soroban_sdk::{
    testutils::Address as _,
    Address, Env,
};

#[derive(Clone)]
struct PolicyContext {
    actor: Address,
    expected: Address,
    emergency_operator: Address,
    emergency_mode: bool,
}

fn actor(context: &PolicyContext) -> Address {
    context.actor.clone()
}

fn expected(context: &PolicyContext) -> Address {
    context.expected.clone()
}

fn emergency_operator(context: &PolicyContext) -> Address {
    context.emergency_operator.clone()
}

fn emergency_mode(context: &PolicyContext) -> bool {
    context.emergency_mode
}

fn setup_initialized_vault(env: &Env) -> (VaultContractClient<'_>, Address) {
    env.mock_all_auths();

    let contract_id = env.register_contract(None, VaultContract);
    let client = VaultContractClient::new(env, &contract_id);

    let admin = Address::generate(env);
    let deposit_token = Address::generate(env);
    let reward_token = Address::generate(env);

    client.initialize(
        &admin,
        &deposit_token,
        &reward_token,
        &86_400_u64,
        &0_i128,
        &soroban_sdk::Vec::new(env),
    );

    (client, admin)
}

#[test]
fn composed_policy_supports_fallback_operator() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let operator = Address::generate(&env);
    let context = PolicyContext {
        actor: operator.clone(),
        expected: admin,
        emergency_operator: operator,
        emergency_mode: true,
    };

    let policy = Authenticated::new(actor).and(
        MatchAddress::new(actor, expected, PolicyViolation::AddressMismatch).or(
            MatchAddress::new(
                actor,
                emergency_operator,
                PolicyViolation::Unauthorized,
            )
            .and(PredicatePolicy::new(
                emergency_mode,
                PolicyViolation::PredicateFailed,
            )),
        ),
    );

    assert_eq!(policy.enforce(&context), Ok(()));
}

#[test]
fn vault_add_asset_rejects_non_admin_actor() {
    let env = Env::default();
    let (client, _admin) = setup_initialized_vault(&env);

    let attacker = Address::generate(&env);
    let asset = Address::generate(&env);

    let result = client.try_add_asset(&attacker, &asset);

    assert_eq!(result, Err(Ok(VaultError::Unauthorized)));
}

#[test]
fn vault_set_penalty_rate_rejects_non_admin_actor() {
    let env = Env::default();
    let (client, _admin) = setup_initialized_vault(&env);

    let attacker = Address::generate(&env);
    let result = client.try_set_penalty_rate(&attacker, &500_u32);

    assert_eq!(result, Err(Ok(VaultError::Unauthorized)));
}

#[test]
fn vault_accept_admin_rejects_non_pending_admin() {
    let env = Env::default();
    let (client, _admin) = setup_initialized_vault(&env);

    let candidate = Address::generate(&env);
    let outsider = Address::generate(&env);

    client.propose_new_admin(&candidate);

    let result = client.try_accept_admin(&outsider);

    assert_eq!(result, Err(Ok(VaultError::Unauthorized)));
}
