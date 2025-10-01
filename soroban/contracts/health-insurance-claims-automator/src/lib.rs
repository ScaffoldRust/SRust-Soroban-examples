#![no_std]

mod claims;
mod automator;
mod utils;

use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, String, Vec};

#[contract]
pub struct HealthInsuranceClaimsAutomator;

#[contractimpl]
impl HealthInsuranceClaimsAutomator {
    /// Initialize the health insurance claims automator
    pub fn initialize(
        env: Env,
        admin: Address,
        default_max_auto_amount: i128,
    ) {
        automator::initialize_automator(&env, admin, default_max_auto_amount);
    }

    /// Submit a new insurance claim
    pub fn submit_claim(
        env: Env,
        patient: Address,
        insurer: Address,
        claim_amount: i128,
        currency: Address,
        diagnosis_code: String,
        evidence_hash: BytesN<32>,
    ) -> u64 {
        claims::submit_claim(
            &env,
            patient,
            insurer,
            claim_amount,
            currency,
            diagnosis_code,
            evidence_hash,
        )
    }

    /// Process a claim (triggers auto-approval check)
    pub fn process_claim(env: Env, claim_id: u64, insurer: Address) {
        automator::process_claim(&env, claim_id, insurer);
    }

    /// Manually approve a claim with specific amount
    pub fn approve_claim(
        env: Env,
        claim_id: u64,
        insurer: Address,
        approved_amount: i128,
    ) {
        automator::manually_approve_claim(&env, claim_id, insurer, approved_amount);
    }

    /// Reject a claim with reason
    pub fn reject_claim(
        env: Env,
        claim_id: u64,
        insurer: Address,
        rejection_reason: String,
    ) {
        automator::reject_claim(&env, claim_id, insurer, rejection_reason);
    }

    /// Execute payout for an approved claim
    pub fn payout(env: Env, claim_id: u64, insurer: Address) {
        automator::execute_payout(&env, claim_id, insurer);
    }

    /// Get claim status
    pub fn get_status(env: Env, claim_id: u64) -> Option<claims::ClaimStatus> {
        claims::get_claim_status(&env, claim_id)
    }

    /// Get full claim details
    pub fn get_claim(env: Env, claim_id: u64) -> Option<claims::Claim> {
        claims::get_claim(&env, claim_id)
    }

    /// Get all claims for a patient
    pub fn get_patient_claims(env: Env, patient: Address) -> Vec<u64> {
        claims::get_patient_claims(&env, patient)
    }

    /// Get all claims for an insurer
    pub fn get_insurer_claims(env: Env, insurer: Address) -> Vec<u64> {
        claims::get_insurer_claims(&env, insurer)
    }

    /// Get pending claims queue
    pub fn get_pending_queue(env: Env) -> Vec<u64> {
        claims::get_pending_queue(&env)
    }

    /// Add auto-approval rule
    pub fn add_auto_approval_rule(
        env: Env,
        admin: Address,
        max_amount: i128,
        diagnosis_codes: Vec<String>,
    ) -> u64 {
        automator::add_auto_approval_rule(&env, admin, max_amount, diagnosis_codes)
    }

    /// Update auto-approval rule
    pub fn update_auto_approval_rule(
        env: Env,
        admin: Address,
        rule_id: u64,
        max_amount: Option<i128>,
        diagnosis_codes: Option<Vec<String>>,
        enabled: Option<bool>,
    ) {
        automator::update_auto_approval_rule(
            &env,
            admin,
            rule_id,
            max_amount,
            diagnosis_codes,
            enabled,
        );
    }

    /// Get auto-approval rule details
    pub fn get_auto_approval_rule(env: Env, rule_id: u64) -> Option<automator::AutoApprovalRule> {
        automator::get_rule(&env, rule_id)
    }

    /// Get payout record for a claim
    pub fn get_payout_record(env: Env, claim_id: u64) -> Option<automator::PayoutRecord> {
        automator::get_payout(&env, claim_id)
    }

    /// File a dispute for a claim
    pub fn file_dispute(
        env: Env,
        claim_id: u64,
        disputer: Address,
        reason: String,
    ) {
        claims::file_dispute(&env, claim_id, disputer, reason);
    }

    /// Resolve a dispute (admin only)
    pub fn resolve_dispute(
        env: Env,
        claim_id: u64,
        admin: Address,
        resolution: String,
        final_status: claims::ClaimStatus,
    ) {
        claims::resolve_dispute(&env, claim_id, admin, resolution, final_status);
    }

    /// Get dispute record
    pub fn get_dispute(env: Env, claim_id: u64) -> Option<claims::DisputeRecord> {
        claims::get_dispute(&env, claim_id)
    }

    /// Validate claim parameters before submission
    pub fn validate_claim_parameters(
        env: Env,
        claim_amount: i128,
        diagnosis_code: String,
        evidence_hash: BytesN<32>,
    ) -> bool {
        utils::validate_claim_parameters(&env, claim_amount, &diagnosis_code, &evidence_hash).is_ok()
    }

    /// Calculate approval percentage
    pub fn calculate_approval_percentage(
        _env: Env,
        claim_amount: i128,
        approved_amount: i128,
    ) -> u32 {
        utils::calculate_approval_percentage(claim_amount, approved_amount)
    }

    /// Calculate processing fee
    pub fn calculate_processing_fee(
        _env: Env,
        claim_amount: i128,
        fee_basis_points: u32,
    ) -> i128 {
        utils::calculate_processing_fee(claim_amount, fee_basis_points)
    }

    /// Check if claim requires manual review
    pub fn requires_manual_review(
        env: Env,
        claim_id: u64,
        high_risk_threshold: i128,
    ) -> bool {
        let claim = claims::get_claim(&env, claim_id);
        match claim {
            Some(c) => utils::requires_manual_review(&c, high_risk_threshold),
            None => false,
        }
    }

    /// Calculate claim processing time
    pub fn get_processing_time(env: Env, claim_id: u64) -> u64 {
        let claim = claims::get_claim(&env, claim_id);
        match claim {
            Some(c) => utils::calculate_processing_time(&env, &c),
            None => 0,
        }
    }
}

#[cfg(test)]
mod test;
