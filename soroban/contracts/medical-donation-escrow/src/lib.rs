#![no_std]

mod escrow;
mod release;
mod utils;

use soroban_sdk::{contract, contractimpl, Address, Env, String, Symbol, Vec};

#[contract]
pub struct MedicalDonationEscrow;

#[contractimpl]
impl MedicalDonationEscrow {
    /// Initialize a new donation escrow
    pub fn initialize(
        env: Env,
        donation_id: u64,
        donor: Address,
        recipient: Address,
        amount: i128,
        token: Symbol,
        description: String,
        milestones: Vec<String>,
    ) {
        escrow::initialize(
            &env,
            donation_id,
            donor,
            recipient,
            amount,
            token,
            description,
            milestones,
        );
    }

    /// Deposit funds into the escrow
    pub fn deposit(env: Env, donation_id: u64, donor: Address, amount: i128, token: Symbol) {
        escrow::deposit(&env, donation_id, donor, amount, token);
    }

    /// Verify a milestone has been completed
    pub fn verify_milestone(
        env: Env,
        donation_id: u64,
        verifier: Address,
        milestone_index: u32,
        verification_data: String,
    ) {
        release::verify_milestone(
            &env,
            donation_id,
            verifier,
            milestone_index,
            verification_data,
        );
    }

    /// Release funds to recipient after milestone completion
    pub fn release_funds(env: Env, donation_id: u64, releaser: Address) {
        release::release_funds(&env, donation_id, releaser);
    }

    /// Process refund to donor if conditions not met
    pub fn refund(env: Env, donation_id: u64, refund_processor: Address) {
        utils::refund(&env, donation_id, refund_processor);
    }

    /// Get donation details
    pub fn get_donation(env: Env, donation_id: u64) -> Option<escrow::Donation> {
        escrow::get_donation(&env, donation_id)
    }

    /// Get donation status
    pub fn get_donation_status(env: Env, donation_id: u64) -> Option<escrow::DonationStatus> {
        escrow::get_donation_status(&env, donation_id)
    }

    /// List all donations for a user
    pub fn get_user_donations(env: Env, user: Address) -> Vec<u64> {
        escrow::get_user_donations(&env, user)
    }

    /// Get milestone verification status
    pub fn get_milestone_status(env: Env, donation_id: u64, milestone_index: u32) -> bool {
        release::get_milestone_status(&env, donation_id, milestone_index)
    }

    /// Emergency pause for a donation (admin only)
    pub fn pause_donation(env: Env, donation_id: u64, admin: Address) {
        escrow::pause_donation(&env, donation_id, admin);
    }

    /// Resume a paused donation (admin only)
    pub fn resume_donation(env: Env, donation_id: u64, admin: Address) {
        escrow::resume_donation(&env, donation_id, admin);
    }

    /// Request a refund for a donation
    pub fn request_refund(env: Env, donation_id: u64, requester: Address, reason: String) {
        utils::request_refund(&env, donation_id, requester, reason);
    }

    /// Approve a refund request (admin only)
    pub fn approve_refund(env: Env, donation_id: u64, approver: Address) {
        utils::approve_refund(&env, donation_id, approver);
    }

    /// Get refund request details
    pub fn get_refund_request(env: Env, donation_id: u64) -> Option<utils::RefundRequest> {
        utils::get_refund_request(&env, donation_id)
    }

    /// Get refund queue
    pub fn get_refund_queue(env: Env) -> Vec<u64> {
        utils::get_refund_queue(&env)
    }

    /// Validate donation parameters
    pub fn validate_donation_parameters(
        env: Env,
        amount: i128,
        token: Symbol,
        milestones: Vec<String>,
    ) -> bool {
        utils::validate_donation_parameters(&env, amount, token, &milestones).is_ok()
    }

    /// Calculate refund amount based on completed milestones
    pub fn calculate_refund_amount(
        env: Env,
        donation_id: u64,
        completed_milestones: u32,
    ) -> Option<i128> {
        let donation = escrow::get_donation(&env, donation_id)?;
        Some(utils::calculate_refund_amount(
            &env,
            &donation,
            completed_milestones,
        ))
    }

    /// Get donation metrics and analytics
    pub fn get_donation_metrics(env: Env, donation_id: u64) -> Option<utils::DonationMetrics> {
        utils::get_donation_metrics(&env, donation_id)
    }

    /// Get milestone verification details
    pub fn get_milestone_verification(
        env: Env,
        donation_id: u64,
        milestone_index: u32,
    ) -> Option<release::MilestoneVerification> {
        release::get_milestone_verification(&env, donation_id, milestone_index)
    }

    /// Get all milestone verifications for a donation
    pub fn get_all_milestone_verifications(
        env: Env,
        donation_id: u64,
    ) -> Vec<release::MilestoneVerification> {
        release::get_all_milestone_verifications(&env, donation_id)
    }

    /// Get release queue
    pub fn get_release_queue(env: Env) -> Vec<u64> {
        release::get_release_queue(&env)
    }
}

#[cfg(test)]
mod test;
