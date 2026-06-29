use super::*;
use soroban_sdk::{testutils::Address as _, vec, Address, Env};

fn formula() -> RewardFormula {
    RewardFormula {
        min_emission_bps: 2_000,
        max_emission_bps: 10_000,
        target_activity: 1_000,
        activity_weight_bps: 6_000,
        participation_weight_bps: 4_000,
    }
}

fn metrics(activity_score: i128, active_participants: u32) -> ProtocolMetrics {
    ProtocolMetrics {
        activity_score,
        active_participants,
        target_participants: 10,
    }
}

#[test]
fn rewards_adapt_to_protocol_activity_and_participation() {
    let e = Env::default();
    let alice = Address::generate(&e);
    let bob = Address::generate(&e);
    let participants = vec![
        &e,
        ParticipationMetric {
            participant: alice,
            stake: 100,
            activity_score: 50,
        },
        ParticipationMetric {
            participant: bob,
            stake: 100,
            activity_score: 50,
        },
    ];

    let low =
        simulate_allocations(&e, &formula(), &metrics(100, 1), &participants, 10_000).unwrap();
    let high =
        simulate_allocations(&e, &formula(), &metrics(1_000, 10), &participants, 10_000).unwrap();

    assert_eq!(low.activity_multiplier_bps, 1_000);
    assert_eq!(low.participation_multiplier_bps, 1_000);
    assert_eq!(low.adjusted_pool, 2_800);
    assert_eq!(high.activity_multiplier_bps, 10_000);
    assert_eq!(high.participation_multiplier_bps, 10_000);
    assert_eq!(high.adjusted_pool, 10_000);
    assert!(high.adjusted_pool > low.adjusted_pool);
}

#[test]
fn allocation_is_deterministic_and_conserves_adjusted_pool() {
    let e = Env::default();
    let participants = vec![
        &e,
        ParticipationMetric {
            participant: Address::generate(&e),
            stake: 250,
            activity_score: 50,
        },
        ParticipationMetric {
            participant: Address::generate(&e),
            stake: 100,
            activity_score: 100,
        },
        ParticipationMetric {
            participant: Address::generate(&e),
            stake: 50,
            activity_score: 50,
        },
    ];

    let first =
        simulate_allocations(&e, &formula(), &metrics(500, 5), &participants, 10_001).unwrap();
    let second =
        simulate_allocations(&e, &formula(), &metrics(500, 5), &participants, 10_001).unwrap();

    assert_eq!(first, second);
    let mut total = 0_i128;
    for allocation in first.allocations.iter() {
        total += allocation.amount;
    }
    assert_eq!(total, first.adjusted_pool);
    assert_eq!(first.adjusted_pool, 6_000);
}

#[test]
fn rejects_edge_cases() {
    let e = Env::default();
    let participant = ParticipationMetric {
        participant: Address::generate(&e),
        stake: 0,
        activity_score: 0,
    };
    let participants = vec![&e, participant];

    assert_eq!(
        simulate_allocations(&e, &formula(), &metrics(1, 1), &participants, 0).unwrap_err(),
        RewardError::InvalidRewardPool
    );
    assert_eq!(
        simulate_allocations(&e, &formula(), &metrics(1, 1), &Vec::new(&e), 1).unwrap_err(),
        RewardError::EmptyParticipants
    );
    assert_eq!(
        simulate_allocations(&e, &formula(), &metrics(1, 1), &participants, 1).unwrap_err(),
        RewardError::InvalidParticipantMetric
    );

    let mut invalid_formula = formula();
    invalid_formula.activity_weight_bps = 5_000;
    invalid_formula.participation_weight_bps = 4_000;
    assert_eq!(
        simulate_allocations(&e, &invalid_formula, &metrics(1, 1), &participants, 1).unwrap_err(),
        RewardError::InvalidFormula
    );
}
