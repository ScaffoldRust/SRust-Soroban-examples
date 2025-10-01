use soroban_sdk::{contracttype, BytesN, Env, String};

use crate::claims::Claim;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClaimMetrics {
    pub total_submitted: u64,
    pub total_approved: u64,
    pub total_rejected: u64,
    pub total_paid: u64,
    pub total_disputed: u64,
    pub total_amount_claimed: i128,
    pub total_amount_approved: i128,
    pub total_amount_paid: i128,
}

/// Validate claim submission parameters
pub fn validate_claim_parameters(
    env: &Env,
    claim_amount: i128,
    diagnosis_code: &String,
    evidence_hash: &BytesN<32>,
) -> Result<(), String> {
    // Validate amount is positive
    if claim_amount <= 0 {
        return Err(String::from_str(env, "Claim amount must be positive"));
    }

    // Validate diagnosis code is not empty
    if diagnosis_code.len() == 0 {
        return Err(String::from_str(env, "Diagnosis code required"));
    }

    // Validate diagnosis code format (basic check)
    if diagnosis_code.len() < 3 || diagnosis_code.len() > 20 {
        return Err(String::from_str(env, "Invalid diagnosis code format"));
    }

    // Validate evidence hash is not zero
    let zero_hash = BytesN::from_array(env, &[0u8; 32]);
    if *evidence_hash == zero_hash {
        return Err(String::from_str(env, "Evidence hash required"));
    }

    Ok(())
}

/// Calculate approval percentage based on claim amount
pub fn calculate_approval_percentage(claim_amount: i128, approved_amount: i128) -> u32 {
    if claim_amount <= 0 {
        return 0;
    }

    let percentage = (approved_amount as u128 * 100) / claim_amount as u128;
    percentage as u32
}

/// Calculate platform fee for processing claim
pub fn calculate_processing_fee(claim_amount: i128, fee_basis_points: u32) -> i128 {
    // Fee is in basis points (1/100th of a percent)
    // Example: 50 basis points = 0.5%
    let fee = (claim_amount as u128 * fee_basis_points as u128) / 10000;
    fee as i128
}

/// Validate insurer has sufficient balance for payout
pub fn validate_insurer_balance(
    _env: &Env,
    available_balance: i128,
    payout_amount: i128,
) -> Result<(), String> {
    if available_balance < payout_amount {
        return Err(String::from_str(_env, "Insufficient insurer balance"));
    }

    Ok(())
}

/// Calculate time elapsed since claim submission
pub fn calculate_processing_time(env: &Env, claim: &Claim) -> u64 {
    let current_time = env.ledger().timestamp();
    if current_time >= claim.submitted_at {
        current_time - claim.submitted_at
    } else {
        0
    }
}

/// Check if claim is within processing deadline
pub fn is_within_processing_deadline(
    env: &Env,
    claim: &Claim,
    deadline_seconds: u64,
) -> bool {
    let processing_time = calculate_processing_time(env, claim);
    processing_time <= deadline_seconds
}

/// Generate claim summary for reporting
pub fn generate_claim_summary(claim: &Claim) -> String {
    // This is a simplified version - in production would format better
    String::from_str(
        &claim.patient.env(),
        "Claim summary generated",
    )
}

/// Validate diagnosis code format (HIPAA compliant)
pub fn validate_diagnosis_code_format(env: &Env, code: &String) -> Result<(), String> {
    // ICD-10 codes are typically 3-7 characters
    if code.len() < 3 || code.len() > 7 {
        return Err(String::from_str(env, "Invalid ICD-10 code length"));
    }

    // Additional validation could include:
    // - First character should be a letter
    // - Subsequent characters can be alphanumeric
    // For simplicity, we'll just check length

    Ok(())
}

/// Calculate average claim amount for analytics
pub fn calculate_average_claim_amount(total_amount: i128, claim_count: u64) -> i128 {
    if claim_count == 0 {
        return 0;
    }

    total_amount / claim_count as i128
}

/// Check if claim requires manual review based on risk factors
pub fn requires_manual_review(
    claim: &Claim,
    high_risk_threshold: i128,
) -> bool {
    // Claims above threshold require manual review
    if claim.claim_amount > high_risk_threshold {
        return true;
    }

    // Additional risk factors could include:
    // - Previous rejected claims from patient
    // - Unusual diagnosis codes
    // - Missing documentation
    // For now, just check amount

    false
}

/// Format claim amount for display
pub fn format_claim_amount(_env: &Env, _amount: i128) -> String {
    // Simplified formatting - in production would handle decimal places
    String::from_str(_env, "Formatted amount")
}

/// Validate evidence hash integrity
pub fn validate_evidence_integrity(
    env: &Env,
    provided_hash: &BytesN<32>,
    expected_hash: &BytesN<32>,
) -> Result<(), String> {
    if provided_hash != expected_hash {
        return Err(String::from_str(env, "Evidence hash mismatch"));
    }

    Ok(())
}

/// Calculate claim complexity score for routing
pub fn calculate_claim_complexity(claim: &Claim, base_complexity: u32) -> u32 {
    let mut complexity = base_complexity;

    // Higher amounts increase complexity
    if claim.claim_amount > 10000 {
        complexity += 2;
    } else if claim.claim_amount > 5000 {
        complexity += 1;
    }

    // Could add other factors:
    // - Number of procedures
    // - Diagnosis code rarity
    // - Provider history

    complexity
}

/// Check compliance with HIPAA data handling requirements
pub fn validate_hipaa_compliance(
    env: &Env,
    data_encrypted: bool,
    authorized_parties_only: bool,
) -> Result<(), String> {
    if !data_encrypted {
        return Err(String::from_str(env, "Data must be encrypted"));
    }

    if !authorized_parties_only {
        return Err(String::from_str(env, "Unauthorized access detected"));
    }

    Ok(())
}

/// Calculate maximum allowable claim amount based on policy
pub fn calculate_max_allowable_amount(
    policy_coverage_limit: i128,
    already_claimed_this_period: i128,
) -> i128 {
    if policy_coverage_limit <= already_claimed_this_period {
        return 0;
    }

    policy_coverage_limit - already_claimed_this_period
}

/// Validate claim is within policy period
pub fn is_within_policy_period(
    env: &Env,
    claim_date: u64,
    policy_start: u64,
    policy_end: u64,
) -> Result<(), String> {
    if claim_date < policy_start || claim_date > policy_end {
        return Err(String::from_str(env, "Claim outside policy period"));
    }

    Ok(())
}
