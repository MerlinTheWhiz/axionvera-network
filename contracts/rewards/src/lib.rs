#![no_std]

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, Address, Env, Vec};

/// Fixed-point denominator for protocol activity and participation weights.
pub const REWARD_BPS_DENOMINATOR: u32 = 10_000;
const MAX_PARTICIPANTS: u32 = 64;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RewardFormula {
    /// Minimum emission rate in basis points of the requested reward pool.
    pub min_emission_bps: u32,
    /// Maximum emission rate in basis points of the requested reward pool.
    pub max_emission_bps: u32,
    /// Target activity score that maps to the maximum activity multiplier.
    pub target_activity: i128,
    /// Weight of total protocol activity in the emission multiplier.
    pub activity_weight_bps: u32,
    /// Weight of participant breadth in the emission multiplier.
    pub participation_weight_bps: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProtocolMetrics {
    /// Deterministic aggregate activity value, e.g. volume, deposits, or tx count.
    pub activity_score: i128,
    /// Number of eligible active participants in the epoch.
    pub active_participants: u32,
    /// Target participant count that maps to the maximum participation multiplier.
    pub target_participants: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParticipationMetric {
    pub participant: Address,
    /// Stake, liquidity, or other eligible balance for the epoch.
    pub stake: i128,
    /// User activity score for the same epoch.
    pub activity_score: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RewardAllocation {
    pub participant: Address,
    pub amount: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RewardSimulation {
    pub requested_pool: i128,
    pub adjusted_pool: i128,
    pub activity_multiplier_bps: u32,
    pub participation_multiplier_bps: u32,
    pub allocations: Vec<RewardAllocation>,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum RewardError {
    InvalidFormula = 1,
    InvalidMetrics = 2,
    InvalidRewardPool = 3,
    EmptyParticipants = 4,
    TooManyParticipants = 5,
    InvalidParticipantMetric = 6,
    ArithmeticOverflow = 7,
}

#[contract]
pub struct RewardEngine;

#[contractimpl]
impl RewardEngine {
    pub fn version() -> u32 {
        1
    }

    pub fn simulate(
        e: Env,
        formula: RewardFormula,
        metrics: ProtocolMetrics,
        participants: Vec<ParticipationMetric>,
        reward_pool: i128,
    ) -> Result<RewardSimulation, RewardError> {
        simulate_allocations(&e, &formula, &metrics, &participants, reward_pool)
    }
}

pub fn simulate_allocations(
    e: &Env,
    formula: &RewardFormula,
    metrics: &ProtocolMetrics,
    participants: &Vec<ParticipationMetric>,
    reward_pool: i128,
) -> Result<RewardSimulation, RewardError> {
    validate_formula(formula)?;
    validate_metrics(metrics)?;
    if reward_pool <= 0 {
        return Err(RewardError::InvalidRewardPool);
    }
    if participants.is_empty() {
        return Err(RewardError::EmptyParticipants);
    }
    if participants.len() > MAX_PARTICIPANTS {
        return Err(RewardError::TooManyParticipants);
    }

    let activity_multiplier_bps = ratio_bps(metrics.activity_score, formula.target_activity)?;
    let participation_multiplier_bps = if metrics.target_participants == 0 {
        return Err(RewardError::InvalidMetrics);
    } else {
        capped_ratio_bps(
            metrics.active_participants as i128,
            metrics.target_participants as i128,
        )?
    };

    let dynamic_span = formula
        .max_emission_bps
        .checked_sub(formula.min_emission_bps)
        .ok_or(RewardError::InvalidFormula)?;
    let blended_signal = weighted_signal_bps(
        activity_multiplier_bps,
        participation_multiplier_bps,
        formula.activity_weight_bps,
        formula.participation_weight_bps,
    )?;
    let emission_bps = formula
        .min_emission_bps
        .checked_add(scale_u32(dynamic_span, blended_signal)?)
        .ok_or(RewardError::ArithmeticOverflow)?;
    let adjusted_pool = mul_div_i128(
        reward_pool,
        emission_bps as i128,
        REWARD_BPS_DENOMINATOR as i128,
    )?;

    let mut total_score = 0_i128;
    for participant in participants.iter() {
        if participant.stake < 0 || participant.activity_score < 0 {
            return Err(RewardError::InvalidParticipantMetric);
        }
        total_score = total_score
            .checked_add(participant.stake)
            .and_then(|v| v.checked_add(participant.activity_score))
            .ok_or(RewardError::ArithmeticOverflow)?;
    }
    if total_score <= 0 {
        return Err(RewardError::InvalidParticipantMetric);
    }

    let mut allocations = Vec::new(e);
    let mut allocated = 0_i128;
    let last_index = participants.len() - 1;
    for (index, participant) in participants.iter().enumerate() {
        let participant_score = participant
            .stake
            .checked_add(participant.activity_score)
            .ok_or(RewardError::ArithmeticOverflow)?;
        let amount = if index as u32 == last_index {
            adjusted_pool
                .checked_sub(allocated)
                .ok_or(RewardError::ArithmeticOverflow)?
        } else {
            mul_div_i128(adjusted_pool, participant_score, total_score)?
        };
        allocated = allocated
            .checked_add(amount)
            .ok_or(RewardError::ArithmeticOverflow)?;
        allocations.push_back(RewardAllocation {
            participant: participant.participant,
            amount,
        });
    }

    Ok(RewardSimulation {
        requested_pool: reward_pool,
        adjusted_pool,
        activity_multiplier_bps,
        participation_multiplier_bps,
        allocations,
    })
}

fn validate_formula(formula: &RewardFormula) -> Result<(), RewardError> {
    let total_weight = formula
        .activity_weight_bps
        .checked_add(formula.participation_weight_bps)
        .ok_or(RewardError::InvalidFormula)?;
    if formula.min_emission_bps > formula.max_emission_bps
        || formula.max_emission_bps > REWARD_BPS_DENOMINATOR
        || formula.target_activity <= 0
        || total_weight != REWARD_BPS_DENOMINATOR
    {
        return Err(RewardError::InvalidFormula);
    }
    Ok(())
}

fn validate_metrics(metrics: &ProtocolMetrics) -> Result<(), RewardError> {
    if metrics.activity_score < 0 || metrics.target_participants == 0 {
        return Err(RewardError::InvalidMetrics);
    }
    Ok(())
}

fn ratio_bps(value: i128, target: i128) -> Result<u32, RewardError> {
    capped_ratio_bps(value, target)
}

fn capped_ratio_bps(value: i128, target: i128) -> Result<u32, RewardError> {
    if value < 0 || target <= 0 {
        return Err(RewardError::InvalidMetrics);
    }
    if value >= target {
        return Ok(REWARD_BPS_DENOMINATOR);
    }
    Ok(mul_div_i128(value, REWARD_BPS_DENOMINATOR as i128, target)? as u32)
}

fn weighted_signal_bps(
    activity_bps: u32,
    participation_bps: u32,
    activity_weight_bps: u32,
    participation_weight_bps: u32,
) -> Result<u32, RewardError> {
    let activity = (activity_bps as u128)
        .checked_mul(activity_weight_bps as u128)
        .ok_or(RewardError::ArithmeticOverflow)?;
    let participation = (participation_bps as u128)
        .checked_mul(participation_weight_bps as u128)
        .ok_or(RewardError::ArithmeticOverflow)?;
    Ok(((activity + participation) / REWARD_BPS_DENOMINATOR as u128) as u32)
}

fn scale_u32(value: u32, bps: u32) -> Result<u32, RewardError> {
    Ok(((value as u128)
        .checked_mul(bps as u128)
        .ok_or(RewardError::ArithmeticOverflow)?
        / REWARD_BPS_DENOMINATOR as u128) as u32)
}

fn mul_div_i128(value: i128, numerator: i128, denominator: i128) -> Result<i128, RewardError> {
    if denominator <= 0 {
        return Err(RewardError::ArithmeticOverflow);
    }
    value
        .checked_mul(numerator)
        .and_then(|v| v.checked_div(denominator))
        .ok_or(RewardError::ArithmeticOverflow)
}

#[cfg(test)]
mod test;
