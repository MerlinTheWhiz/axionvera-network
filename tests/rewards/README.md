# Dynamic Reward Engine Simulations

The reward engine in `contracts/rewards` models deterministic reward allocation for an epoch.

## Formula

1. Normalize protocol activity: `activity_multiplier_bps = min(activity_score / target_activity, 1) * 10_000`.
2. Normalize participation breadth: `participation_multiplier_bps = min(active_participants / target_participants, 1) * 10_000`.
3. Blend both protocol signals with formula weights that must sum to 10,000 bps.
4. Adjust the pool between `min_emission_bps` and `max_emission_bps` using the blended signal.
5. Split the adjusted pool across participants by `stake + activity_score`; any integer remainder is assigned to the final participant so allocation is deterministic and exactly conserves the pool.

## Simulation Results Covered by Tests

- Low activity and low participation emit only 2,800 of a 10,000 reward pool under the default test formula.
- Target activity and target participation emit the full 10,000 reward pool.
- Repeated simulations with identical inputs produce identical allocations and conserve the adjusted pool.
- Invalid pools, empty participant sets, zero participant score, and invalid formula weights are rejected.
