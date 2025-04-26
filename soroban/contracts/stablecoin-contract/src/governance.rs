// stablecoin-contract/src/governance.rs

use soroban_sdk::{contracttype, Address, Env};
// use soroban_sdk::String;
use soroban_sdk::{String, Vec};

// use alloc::vec::Vec;

#[derive(Clone, Debug, PartialEq)]
#[contracttype]
pub enum ProposalStatus {
    Pending,
    Approved,
    Rejected,
    Executed,
}

#[derive(Clone, Debug, PartialEq)]
#[contracttype]
pub enum Vote {
    Yes,
    No,
    Abstain,
}

#[derive(Clone, Debug, PartialEq)]
#[contracttype]
pub struct GovernanceProposal {
    pub proposal_id: u64,
    pub action_type: String,
    pub parameters: Vec<String>,
    pub proposer: Address,
    pub status: ProposalStatus,
    pub votes: Vec<(Address, Vote)>,
}

#[contracttype]
pub enum DataKey {
    Proposal(u64),
    ProposalCount,
}

// Create a new proposal
pub fn propose_governance_action(
    env: &Env,
    proposer: Address,
    action_type: String,
    parameters: Vec<String>,
) -> u64 {
    proposer.require_auth();

    let count: u64 = env
        .storage()
        .instance()
        .get(&DataKey::ProposalCount)
        .unwrap_or(0);
    let new_id = count + 1;

    let proposal = GovernanceProposal {
        proposal_id: new_id,
        action_type,
        parameters,
        proposer: proposer.clone(),
        status: ProposalStatus::Pending,
        votes: Vec::new(env),
    };

    env.storage()
        .instance()
        .set(&DataKey::Proposal(new_id), &proposal);
    env.storage()
        .instance()
        .set(&DataKey::ProposalCount, &new_id);

    new_id
}

// Vote on a proposal
pub fn vote(env: &Env, voter: Address, proposal_id: u64, vote: Vote) {
    voter.require_auth();

    let mut proposal: GovernanceProposal = env
        .storage()
        .instance()
        .get(&DataKey::Proposal(proposal_id))
        .expect("Proposal not found");

    // Prevent double voting
    if proposal.votes.iter().any(|(addr, _)| addr == voter) {
        panic!("Already voted");
    }

    proposal.votes.push_back((voter.clone(), vote));
    env.storage()
        .instance()
        .set(&DataKey::Proposal(proposal_id), &proposal);
}

// (Simplified) Execute a proposal if majority Yes votes
pub fn execute_proposal(env: &Env, executor: Address, proposal_id: u64) {
    executor.require_auth();

    let mut proposal: GovernanceProposal = env
        .storage()
        .instance()
        .get(&DataKey::Proposal(proposal_id))
        .expect("Proposal not found");

    assert_eq!(
        proposal.status,
        ProposalStatus::Pending,
        "Already executed or finalized"
    );

    let yes_votes = proposal
        .votes
        .iter()
        .filter(|(_, v)| *v == Vote::Yes)
        .count();
    let no_votes = proposal
        .votes
        .iter()
        .filter(|(_, v)| *v == Vote::No)
        .count();

    // Simple majority check
    if yes_votes > no_votes {
        // Here, you would implement the actual effect of the proposal action
        proposal.status = ProposalStatus::Approved;
        // TODO: call functions like adjust_parameters() based on action_type & parameters
    } else {
        proposal.status = ProposalStatus::Rejected;
    }

    env.storage()
        .instance()
        .set(&DataKey::Proposal(proposal_id), &proposal);
}
