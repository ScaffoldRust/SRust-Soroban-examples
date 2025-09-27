use crate::escrow::{storage_key_donation, Donation, DonationStatus, ADMIN};
use soroban_sdk::{contracttype, symbol_short, Address, Env, String, Symbol, Vec};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MilestoneVerification {
    pub milestone_index: u32,
    pub verified: bool,
    pub verified_at: Option<u64>,
    pub verifier: Option<Address>,
    pub verification_data: Option<String>,
}

// Storage keys
const MILESTONE_VERIFICATIONS: Symbol = symbol_short!("MILESTONE");
const RELEASE_QUEUE: Symbol = symbol_short!("RELEASE");

pub fn verify_milestone(
    env: &Env,
    donation_id: u64,
    verifier: Address,
    milestone_index: u32,
    verification_data: String,
) {
    let mut donation = crate::escrow::get_donation(env, donation_id)
        .unwrap_or_else(|| panic!("Donation not found"));

    // Validate donation is in correct state
    if donation.status != DonationStatus::Funded && donation.status != DonationStatus::InProgress {
        panic!("Invalid donation status");
    }

    // Validate milestone index
    if milestone_index >= (donation.milestones.len() as u32) {
        panic!("Invalid milestone index");
    }

    // Check if milestone already verified
    if get_milestone_status(env, donation_id, milestone_index) {
        panic!("Milestone already verified");
    }

    // Verify the verifier is authorized (recipient or admin)
    let admin = env
        .storage()
        .instance()
        .get(&ADMIN)
        .unwrap_or_else(|| panic!("No admin set"));

    if verifier != donation.recipient && verifier != admin {
        panic!("Unauthorized verifier");
    }

    // Create milestone verification record
    let verification = MilestoneVerification {
        milestone_index,
        verified: true,
        verified_at: Some(env.ledger().timestamp()),
        verifier: Some(verifier.clone()),
        verification_data: Some(verification_data),
    };

    // Store verification
    env.storage().persistent().set(
        &storage_key_milestone(donation_id, milestone_index),
        &verification,
    );

    // Check if all milestones are now verified
    let all_verified = check_all_milestones_verified(env, &donation);

    if all_verified {
        // Update donation status to completed
        donation.status = DonationStatus::Completed;
        donation.completed_at = Some(env.ledger().timestamp());
        env.storage()
            .persistent()
            .set(&storage_key_donation(donation_id), &donation);

        // Add to release queue
        add_to_release_queue(env, donation_id);
    } else {
        // Update donation status to in progress
        donation.status = DonationStatus::InProgress;
        env.storage()
            .persistent()
            .set(&storage_key_donation(donation_id), &donation);
    }
}

pub fn release_funds(env: &Env, donation_id: u64, releaser: Address) {
    let donation = crate::escrow::get_donation(env, donation_id)
        .unwrap_or_else(|| panic!("Donation not found"));

    // Validate donation is completed
    if donation.status != DonationStatus::Completed {
        panic!("Invalid donation status");
    }

    // Verify releaser is authorized (recipient or admin)
    let admin = env
        .storage()
        .instance()
        .get(&ADMIN)
        .unwrap_or_else(|| panic!("No admin set"));

    if releaser != donation.recipient && releaser != admin {
        panic!("Unauthorized releaser");
    }

    // Check if funds are in release queue
    if !is_in_release_queue(env, donation_id) {
        panic!("Not in release queue");
    }

    // In a real implementation, this would transfer tokens
    // For now, we'll just mark as released by removing from queue
    remove_from_release_queue(env, donation_id);

    // Log the release event
    env.events().publish(
        (symbol_short!("RELEASED"), donation_id),
        (donation.amount, donation.token, donation.recipient),
    );
}

pub fn get_milestone_status(env: &Env, donation_id: u64, milestone_index: u32) -> bool {
    env.storage()
        .persistent()
        .get(&storage_key_milestone(donation_id, milestone_index))
        .map(|v: MilestoneVerification| v.verified)
        .unwrap_or(false)
}

pub fn get_milestone_verification(
    env: &Env,
    donation_id: u64,
    milestone_index: u32,
) -> Option<MilestoneVerification> {
    env.storage()
        .persistent()
        .get(&storage_key_milestone(donation_id, milestone_index))
}

pub fn get_all_milestone_verifications(env: &Env, donation_id: u64) -> Vec<MilestoneVerification> {
    let donation = crate::escrow::get_donation(env, donation_id)
        .unwrap_or_else(|| panic!("Donation not found"));

    let mut verifications = Vec::new(&env);
    for i in 0..donation.milestones.len() {
        if let Some(verification) = get_milestone_verification(env, donation_id, i as u32) {
            verifications.push_back(verification);
        }
    }
    verifications
}

fn check_all_milestones_verified(env: &Env, donation: &Donation) -> bool {
    for i in 0..donation.milestones.len() {
        if !get_milestone_status(env, donation.id, i as u32) {
            return false;
        }
    }
    true
}

fn add_to_release_queue(env: &Env, donation_id: u64) {
    let mut queue = get_release_queue(env);
    queue.push_back(donation_id);
    env.storage().persistent().set(&RELEASE_QUEUE, &queue);
}

fn remove_from_release_queue(env: &Env, donation_id: u64) {
    let queue = get_release_queue(env);
    let mut new_queue = Vec::new(&env);

    for id in queue.iter() {
        if id != donation_id {
            new_queue.push_back(id);
        }
    }

    env.storage().persistent().set(&RELEASE_QUEUE, &new_queue);
}

fn is_in_release_queue(env: &Env, donation_id: u64) -> bool {
    let queue = get_release_queue(env);
    queue.contains(donation_id)
}

pub fn get_release_queue(env: &Env) -> Vec<u64> {
    env.storage()
        .persistent()
        .get(&RELEASE_QUEUE)
        .unwrap_or_else(|| Vec::new(&env))
}

fn storage_key_milestone(donation_id: u64, milestone_index: u32) -> (Symbol, u64, u32) {
    (MILESTONE_VERIFICATIONS, donation_id, milestone_index)
}
