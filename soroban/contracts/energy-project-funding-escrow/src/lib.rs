#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env, String, Symbol, Vec};

mod escrow;
mod milestones;
mod types;
mod utils;

#[cfg(test)]
mod tests;

pub use escrow::*;
pub use milestones::*;
pub use types::*;
pub use utils::*;

#[contract]
pub struct EnergyProjectFundingEscrow;

#[contractimpl]
impl EnergyProjectFundingEscrow {
    /// Initialize the contract with the first project ID
    pub fn initialize(env: Env) {
        env.storage().instance().set(&DataKey::NextProjectId, &1u64);

        env.events().publish(
            (Symbol::new(&env, "ContractInitialized"),),
            env.ledger().timestamp(),
        );
    }

    /// Create a new energy project with escrow funding
    pub fn initialize_project(
        env: Env,
        investor: Address,
        project_manager: Address,
        name: String,
        description: String,
        total_funding: i128,
        milestone_count: u32,
        energy_type: String,
        expected_capacity: i128,
    ) -> u64 {
        escrow::initialize_project(
            env,
            investor,
            project_manager,
            name,
            description,
            total_funding,
            milestone_count,
            energy_type,
            expected_capacity,
        )
    }

    /// Deposit additional funds into an existing project
    pub fn deposit(env: Env, project_id: u64, depositor: Address, amount: i128) {
        escrow::deposit_funds(env, project_id, depositor, amount)
    }

    /// Release funds to project manager upon milestone completion
    pub fn release_funds(env: Env, project_id: u64, milestone_id: u32, approver: Address) {
        escrow::release_funds(env, project_id, milestone_id, approver)
    }

    /// Request a refund from the escrow
    pub fn request_refund(
        env: Env,
        project_id: u64,
        requester: Address,
        amount: i128,
        reason: String,
    ) {
        escrow::request_refund(env, project_id, requester, amount, reason)
    }

    /// Process an approved refund request
    pub fn process_refund(env: Env, project_id: u64, approver: Address) {
        escrow::process_refund(env, project_id, approver)
    }

    /// Setup multi-signature requirements for high-value releases
    pub fn setup_multisig(
        env: Env,
        project_id: u64,
        setup_by: Address,
        required_signatures: u32,
        authorized_signers: Vec<Address>,
        threshold_amount: i128,
    ) {
        escrow::setup_multisig(
            env,
            project_id,
            setup_by,
            required_signatures,
            authorized_signers,
            threshold_amount,
        )
    }

    /// Create a new milestone for a project
    pub fn create_milestone(
        env: Env,
        project_id: u64,
        creator: Address,
        name: String,
        description: String,
        funding_percentage: u32,
        required_verifications: Vec<String>,
        due_date: u64,
        energy_output_target: Option<i128>,
        carbon_offset_target: Option<i128>,
    ) -> u32 {
        milestones::create_milestone(
            env,
            project_id,
            creator,
            name,
            description,
            funding_percentage,
            required_verifications,
            due_date,
            energy_output_target,
            carbon_offset_target,
        )
    }

    /// Start progress on a milestone
    pub fn start_milestone(env: Env, project_id: u64, milestone_id: u32, starter: Address) {
        milestones::start_milestone(env, project_id, milestone_id, starter)
    }

    /// Verify a milestone completion requirement
    pub fn verify_milestone(
        env: Env,
        project_id: u64,
        milestone_id: u32,
        verifier: Address,
        verification_type: String,
        verification_data: String,
    ) {
        milestones::verify_milestone(
            env,
            project_id,
            milestone_id,
            verifier,
            verification_type,
            verification_data,
        )
    }

    /// Mark a milestone as failed
    pub fn fail_milestone(
        env: Env,
        project_id: u64,
        milestone_id: u32,
        manager: Address,
        reason: String,
    ) {
        milestones::fail_milestone(env, project_id, milestone_id, manager, reason)
    }

    /// Update project performance metrics
    pub fn update_metrics(
        env: Env,
        project_id: u64,
        updater: Address,
        actual_energy_output: i128,
        actual_carbon_offset: i128,
        efficiency_rating: u32,
    ) {
        milestones::update_project_metrics(
            env,
            project_id,
            updater,
            actual_energy_output,
            actual_carbon_offset,
            efficiency_rating,
        )
    }

    // Query Functions

    /// Get complete project details
    pub fn get_project(env: Env, project_id: u64) -> ProjectDetails {
        escrow::get_project_details(env, project_id)
    }

    /// Get available funds remaining in escrow
    pub fn get_available_funds(env: Env, project_id: u64) -> i128 {
        escrow::get_available_funds(env, project_id)
    }

    /// Get specific milestone details
    pub fn get_milestone(env: Env, project_id: u64, milestone_id: u32) -> MilestoneDetails {
        milestones::get_milestone_details(env, project_id, milestone_id)
    }

    /// Get all milestones for a project
    pub fn get_milestones(env: Env, project_id: u64) -> Vec<MilestoneDetails> {
        milestones::get_project_milestones(env, project_id)
    }

    /// Get pending fund release information
    pub fn get_pending_release(env: Env, project_id: u64) -> Option<PendingRelease> {
        milestones::get_pending_release(env, project_id)
    }

    /// Calculate project completion percentage
    pub fn get_progress(env: Env, project_id: u64) -> u32 {
        milestones::calculate_project_progress(env, project_id)
    }

    // Utility Functions

    /// Validate if an energy project setup is valid
    pub fn validate_project(
        env: Env,
        investor: Address,
        project_manager: Address,
        total_funding: i128,
        milestone_count: u32,
        energy_type: String,
        expected_capacity: i128,
    ) -> bool {
        utils::validate_project_setup(
            &env,
            &investor,
            &project_manager,
            total_funding,
            milestone_count,
        ) && utils::validate_energy_project_data(&env, &energy_type, expected_capacity)
    }

    /// Calculate milestone funding amount based on percentage
    pub fn calculate_milestone_funding(env: Env, total_funding: i128, percentage: u32) -> i128 {
        utils::calculate_milestone_funding(&env, total_funding, percentage)
    }

    /// Calculate potential refund amount based on project progress
    pub fn calculate_refund(env: Env, project_id: u64) -> i128 {
        let project = escrow::get_project_details(env.clone(), project_id);
        let progress = milestones::calculate_project_progress(env.clone(), project_id);
        let completed_milestones = (progress * project.milestone_count) / 100;

        utils::calculate_refund_amount(&env, &project, completed_milestones)
    }

    /// Check if a milestone is overdue
    pub fn is_milestone_overdue(env: Env, project_id: u64, milestone_id: u32) -> bool {
        let milestone = milestones::get_milestone_details(env.clone(), project_id, milestone_id);
        utils::is_milestone_overdue(&env, &milestone)
    }

    /// Calculate penalty amount for overdue milestone
    pub fn calculate_penalty(env: Env, project_id: u64, milestone_id: u32) -> i128 {
        let milestone = milestones::get_milestone_details(env.clone(), project_id, milestone_id);
        let days_overdue = utils::calculate_days_overdue(&env, milestone.due_date);
        utils::calculate_penalty_amount(&env, &milestone, days_overdue)
    }

    /// Get formatted status message for project
    pub fn get_project_status_message(env: Env, project_id: u64) -> String {
        let project = escrow::get_project_details(env.clone(), project_id);
        utils::format_project_status_message(&project.status)
    }

    /// Get formatted status message for milestone
    pub fn get_milestone_status_message(env: Env, project_id: u64, milestone_id: u32) -> String {
        let milestone = milestones::get_milestone_details(env.clone(), project_id, milestone_id);
        utils::format_milestone_status_message(&milestone.status)
    }
}
