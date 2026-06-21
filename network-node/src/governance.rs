use crate::chain_params::{
    normalize_id, ChainParameterRegistry, GovernanceConfig, NetworkParametersPatch, Role,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, instrument, warn};

/// Represents the lifecycle stages of a governance proposal.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProposalStatus {
    /// Proposal is open for voting.
    Voting,
    /// Proposal passed the required threshold.
    Passed,
    /// Proposal failed to reach the threshold within the voting period.
    Rejected,
    /// Proposal has been successfully applied to the network parameters.
    Executed,
    /// Proposal expired before reaching a decision.
    Expired,
}

/// A single vote cast by a network participant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    pub voter: String,
    pub approve: bool,
    pub weight: u64,
}

/// A governance proposal to modify network parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    pub id: String,
    pub proposer: String,
    pub patch: NetworkParametersPatch,
    pub activation_height: u64,
    pub status: ProposalStatus,
    pub votes: HashMap<String, Vote>,
    pub end_height: u64,
}

/// Manages the governance lifecycle, handling proposal creation, voting, and execution.
pub struct GovernanceService {
    proposals: HashMap<String, Proposal>,
    voting_duration: u64,
}

impl GovernanceService {
    /// Creates a new GovernanceService with the specified voting duration in blocks.
    pub fn new(voting_duration: u64) -> Self {
        Self {
            proposals: HashMap::new(),
            voting_duration,
        }
    }

    /// Submits a new proposal to the governance system.
    #[instrument(skip(self, patch))]
    pub fn propose(
        &mut self,
        proposer: String,
        patch: NetworkParametersPatch,
        activation_height: u64,
        current_height: u64,
        registry: &ChainParameterRegistry,
    ) -> Result<String, String> {
        // Ensure proposer has Operator or Admin role to submit proposals
        registry
            .check_role(&proposer, Role::Operator)
            .map_err(|e| format!("Proposer unauthorized: {}", e))?;

        let id = uuid::Uuid::new_v4().to_string();
        let proposal = Proposal {
            id: id.clone(),
            proposer,
            patch,
            activation_height,
            status: ProposalStatus::Voting,
            votes: HashMap::new(),
            end_height: current_height + self.voting_duration,
        };

        self.proposals.insert(id.clone(), proposal);
        info!(proposal_id = %id, proposer = %proposer, "New governance proposal submitted");
        Ok(id)
    }

    /// Casts a vote on an active proposal.
    #[instrument(skip(self))]
    pub fn cast_vote(
        &mut self,
        proposal_id: &str,
        voter: String,
        approve: bool,
        registry: &ChainParameterRegistry,
    ) -> Result<(), String> {
        // Ensure voter has at least Operator role to participate in governance voting
        registry
            .check_role(&voter, Role::Operator)
            .map_err(|e| format!("Voter unauthorized: {}", e))?;

        let proposal = self
            .proposals
            .get_mut(proposal_id)
            .ok_or_else(|| "Proposal not found".to_string())?;

        if proposal.status != ProposalStatus::Voting {
            return Err(format!(
                "Cannot vote on proposal in {:?} state",
                proposal.status
            ));
        }

        proposal.votes.insert(
            normalize_id(&voter),
            Vote {
                voter,
                approve,
                weight: 1, // Fixed weight for now
            },
        );

        info!(proposal_id = %proposal_id, "Vote recorded");
        Ok(())
    }

    /// Tallies votes for proposals that have reached their end height and executes those that passed.
    pub fn tick(&mut self, current_height: u64, registry: &mut ChainParameterRegistry) {
        let mut to_execute = Vec::new();

        for proposal in self.proposals.values_mut() {
            if proposal.status == ProposalStatus::Voting && current_height >= proposal.end_height {
                let config = registry.governance_config();

                match config {
                    GovernanceConfig::Dao {
                        members,
                        min_approvals,
                    } => {
                        let yes_votes = proposal
                            .votes
                            .values()
                            .filter(|v| {
                                v.approve
                                    && members
                                        .iter()
                                        .any(|m| normalize_id(m) == normalize_id(&v.voter))
                            })
                            .count() as u32;

                        if yes_votes >= *min_approvals {
                            proposal.status = ProposalStatus::Passed;
                            to_execute.push(proposal.id.clone());
                        } else {
                            proposal.status = ProposalStatus::Rejected;
                            info!(proposal_id = %proposal.id, "Proposal rejected: insufficient approvals");
                        }
                    }
                    GovernanceConfig::AdminKeys { .. } => {
                        // Admin keys typically skip the voting process, but we support it for uniformity
                        proposal.status = ProposalStatus::Passed;
                        to_execute.push(proposal.id.clone());
                    }
                }
            }
        }

        for id in to_execute {
            if let Some(proposal) = self.proposals.get_mut(&id) {
                let voters: Vec<String> =
                    proposal.votes.values().map(|v| v.voter.clone()).collect();
                match registry.submit_parameter_upgrade(
                    proposal.patch.clone(),
                    proposal.activation_height,
                    &proposal.proposer,
                    &voters,
                ) {
                    Ok(_) => {
                        proposal.status = ProposalStatus::Executed;
                        info!(proposal_id = %id, "Governance proposal executed successfully");
                    }
                    Err(e) => {
                        warn!(proposal_id = %id, error = %e, "Failed to execute governance proposal");
                    }
                }
            }
        }
    }
}
