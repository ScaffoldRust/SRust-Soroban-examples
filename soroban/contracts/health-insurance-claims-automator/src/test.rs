#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::Address as _, Address, BytesN, Env, String,
};

fn create_test_contract(env: &Env) -> HealthInsuranceClaimsAutomatorClient {
    HealthInsuranceClaimsAutomatorClient::new(env, &env.register(HealthInsuranceClaimsAutomator, ()))
}

fn create_test_evidence_hash(env: &Env) -> BytesN<32> {
    let mut hash_bytes = [0u8; 32];
    hash_bytes[0] = 1;
    hash_bytes[31] = 255;
    BytesN::from_array(env, &hash_bytes)
}

#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    let admin = Address::generate(&env);
    let default_max_auto_amount = 10000;

    contract.initialize(&admin, &default_max_auto_amount);

    // Verify default auto-approval rule was created
    let rule = contract.get_auto_approval_rule(&0);
    assert!(rule.is_some());
    let rule = rule.unwrap();
    assert_eq!(rule.max_amount, default_max_auto_amount);
    assert_eq!(rule.enabled, true);
}

#[test]
fn test_submit_claim() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    let admin = Address::generate(&env);
    let patient = Address::generate(&env);
    let insurer = Address::generate(&env);
    let currency = Address::generate(&env);

    contract.initialize(&admin, &10000);

    let claim_amount = 5000;
    let diagnosis_code = String::from_str(&env, "J20.9");
    let evidence_hash = create_test_evidence_hash(&env);

    let claim_id = contract.submit_claim(
        &patient,
        &insurer,
        &claim_amount,
        &currency,
        &diagnosis_code,
        &evidence_hash,
    );

    assert_eq!(claim_id, 1);

    // Verify claim was created
    let claim = contract.get_claim(&claim_id);
    assert!(claim.is_some());
    let claim = claim.unwrap();
    assert_eq!(claim.patient, patient);
    assert_eq!(claim.insurer, insurer);
    assert_eq!(claim.claim_amount, claim_amount);
    assert_eq!(claim.status, claims::ClaimStatus::Submitted);
}

#[test]
fn test_submit_multiple_claims() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    let admin = Address::generate(&env);
    let patient = Address::generate(&env);
    let insurer = Address::generate(&env);
    let currency = Address::generate(&env);

    contract.initialize(&admin, &10000);

    let diagnosis_code = String::from_str(&env, "J20.9");
    let evidence_hash = create_test_evidence_hash(&env);

    let claim_id1 = contract.submit_claim(
        &patient,
        &insurer,
        &1000,
        &currency,
        &diagnosis_code,
        &evidence_hash,
    );

    let claim_id2 = contract.submit_claim(
        &patient,
        &insurer,
        &2000,
        &currency,
        &diagnosis_code,
        &evidence_hash,
    );

    assert_eq!(claim_id1, 1);
    assert_eq!(claim_id2, 2);

    // Verify patient claims list
    let patient_claims = contract.get_patient_claims(&patient);
    assert_eq!(patient_claims.len(), 2);
}

#[test]
fn test_process_claim_auto_approval() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    let admin = Address::generate(&env);
    let patient = Address::generate(&env);
    let insurer = Address::generate(&env);
    let currency = Address::generate(&env);

    contract.initialize(&admin, &10000);

    let claim_amount = 5000; // Below auto-approval threshold
    let diagnosis_code = String::from_str(&env, "J20.9");
    let evidence_hash = create_test_evidence_hash(&env);

    let claim_id = contract.submit_claim(
        &patient,
        &insurer,
        &claim_amount,
        &currency,
        &diagnosis_code,
        &evidence_hash,
    );

    // Process the claim
    contract.process_claim(&claim_id, &insurer);

    // Verify claim was auto-approved
    let claim = contract.get_claim(&claim_id).unwrap();
    assert_eq!(claim.status, claims::ClaimStatus::Approved);
    assert_eq!(claim.auto_approved, true);
    assert_eq!(claim.approved_amount, claim_amount);
}

#[test]
fn test_process_claim_manual_review() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    let admin = Address::generate(&env);
    let patient = Address::generate(&env);
    let insurer = Address::generate(&env);
    let currency = Address::generate(&env);

    contract.initialize(&admin, &10000);

    let claim_amount = 15000; // Above auto-approval threshold
    let diagnosis_code = String::from_str(&env, "J20.9");
    let evidence_hash = create_test_evidence_hash(&env);

    let claim_id = contract.submit_claim(
        &patient,
        &insurer,
        &claim_amount,
        &currency,
        &diagnosis_code,
        &evidence_hash,
    );

    // Process the claim
    contract.process_claim(&claim_id, &insurer);

    // Verify claim is under review (not auto-approved)
    let claim = contract.get_claim(&claim_id).unwrap();
    assert_eq!(claim.status, claims::ClaimStatus::UnderReview);
    assert_eq!(claim.auto_approved, false);
}

#[test]
fn test_manually_approve_claim() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    let admin = Address::generate(&env);
    let patient = Address::generate(&env);
    let insurer = Address::generate(&env);
    let currency = Address::generate(&env);

    contract.initialize(&admin, &10000);

    let claim_amount = 15000;
    let diagnosis_code = String::from_str(&env, "J20.9");
    let evidence_hash = create_test_evidence_hash(&env);

    let claim_id = contract.submit_claim(
        &patient,
        &insurer,
        &claim_amount,
        &currency,
        &diagnosis_code,
        &evidence_hash,
    );

    // Process and approve manually
    contract.process_claim(&claim_id, &insurer);
    contract.approve_claim(&claim_id, &insurer, &12000);

    // Verify claim was approved with custom amount
    let claim = contract.get_claim(&claim_id).unwrap();
    assert_eq!(claim.status, claims::ClaimStatus::Approved);
    assert_eq!(claim.approved_amount, 12000);
}

#[test]
fn test_reject_claim() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    let admin = Address::generate(&env);
    let patient = Address::generate(&env);
    let insurer = Address::generate(&env);
    let currency = Address::generate(&env);

    contract.initialize(&admin, &10000);

    let claim_amount = 15000;
    let diagnosis_code = String::from_str(&env, "J20.9");
    let evidence_hash = create_test_evidence_hash(&env);

    let claim_id = contract.submit_claim(
        &patient,
        &insurer,
        &claim_amount,
        &currency,
        &diagnosis_code,
        &evidence_hash,
    );

    // Process and reject
    contract.process_claim(&claim_id, &insurer);
    let rejection_reason = String::from_str(&env, "Insufficient documentation");
    contract.reject_claim(&claim_id, &insurer, &rejection_reason);

    // Verify claim was rejected
    let claim = contract.get_claim(&claim_id).unwrap();
    assert_eq!(claim.status, claims::ClaimStatus::Rejected);
    assert!(claim.rejection_reason.is_some());
}

#[test]
fn test_payout() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    let admin = Address::generate(&env);
    let patient = Address::generate(&env);
    let insurer = Address::generate(&env);
    let currency = Address::generate(&env);

    contract.initialize(&admin, &10000);

    let claim_amount = 5000;
    let diagnosis_code = String::from_str(&env, "J20.9");
    let evidence_hash = create_test_evidence_hash(&env);

    let claim_id = contract.submit_claim(
        &patient,
        &insurer,
        &claim_amount,
        &currency,
        &diagnosis_code,
        &evidence_hash,
    );

    // Process (auto-approve) and payout
    contract.process_claim(&claim_id, &insurer);
    contract.payout(&claim_id, &insurer);

    // Verify claim status and payout record
    let claim = contract.get_claim(&claim_id).unwrap();
    assert_eq!(claim.status, claims::ClaimStatus::Paid);

    let payout_record = contract.get_payout_record(&claim_id);
    assert!(payout_record.is_some());
    let payout_record = payout_record.unwrap();
    assert_eq!(payout_record.recipient, patient);
    assert_eq!(payout_record.amount, claim_amount);
}

#[test]
fn test_file_dispute() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    let admin = Address::generate(&env);
    let patient = Address::generate(&env);
    let insurer = Address::generate(&env);
    let currency = Address::generate(&env);

    contract.initialize(&admin, &10000);

    let claim_amount = 15000;
    let diagnosis_code = String::from_str(&env, "J20.9");
    let evidence_hash = create_test_evidence_hash(&env);

    let claim_id = contract.submit_claim(
        &patient,
        &insurer,
        &claim_amount,
        &currency,
        &diagnosis_code,
        &evidence_hash,
    );

    contract.process_claim(&claim_id, &insurer);
    contract.reject_claim(&claim_id, &insurer, &String::from_str(&env, "Denied"));

    // File dispute
    let dispute_reason = String::from_str(&env, "Valid medical necessity");
    contract.file_dispute(&claim_id, &patient, &dispute_reason);

    // Verify dispute was filed
    let claim = contract.get_claim(&claim_id).unwrap();
    assert_eq!(claim.status, claims::ClaimStatus::Disputed);

    let dispute = contract.get_dispute(&claim_id);
    assert!(dispute.is_some());
}

#[test]
fn test_resolve_dispute() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    let admin = Address::generate(&env);
    let patient = Address::generate(&env);
    let insurer = Address::generate(&env);
    let currency = Address::generate(&env);

    contract.initialize(&admin, &10000);

    let claim_amount = 15000;
    let diagnosis_code = String::from_str(&env, "J20.9");
    let evidence_hash = create_test_evidence_hash(&env);

    let claim_id = contract.submit_claim(
        &patient,
        &insurer,
        &claim_amount,
        &currency,
        &diagnosis_code,
        &evidence_hash,
    );

    contract.process_claim(&claim_id, &insurer);
    contract.reject_claim(&claim_id, &insurer, &String::from_str(&env, "Denied"));
    contract.file_dispute(&claim_id, &patient, &String::from_str(&env, "Valid"));

    // Resolve dispute
    let resolution = String::from_str(&env, "Approved after review");
    contract.resolve_dispute(&claim_id, &admin, &resolution, &claims::ClaimStatus::Approved);

    // Verify dispute was resolved
    let dispute = contract.get_dispute(&claim_id).unwrap();
    assert!(dispute.resolved_at.is_some());
    assert!(dispute.resolution.is_some());

    let claim = contract.get_claim(&claim_id).unwrap();
    assert_eq!(claim.status, claims::ClaimStatus::Approved);
}

#[test]
fn test_add_auto_approval_rule() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    let admin = Address::generate(&env);

    contract.initialize(&admin, &10000);

    let mut diagnosis_codes = soroban_sdk::Vec::new(&env);
    diagnosis_codes.push_back(String::from_str(&env, "J20.9"));
    diagnosis_codes.push_back(String::from_str(&env, "J45.9"));

    let rule_id = contract.add_auto_approval_rule(&admin, &5000, &diagnosis_codes);

    // Verify rule was created
    let rule = contract.get_auto_approval_rule(&rule_id);
    assert!(rule.is_some());
    let rule = rule.unwrap();
    assert_eq!(rule.max_amount, 5000);
    assert_eq!(rule.diagnosis_codes.len(), 2);
}

#[test]
fn test_update_auto_approval_rule() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    let admin = Address::generate(&env);

    contract.initialize(&admin, &10000);

    // Update default rule
    contract.update_auto_approval_rule(&admin, &0, &Some(20000), &None, &None);

    // Verify rule was updated
    let rule = contract.get_auto_approval_rule(&0).unwrap();
    assert_eq!(rule.max_amount, 20000);
}

#[test]
fn test_validate_claim_parameters() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);

    let valid_amount = 1000;
    let valid_diagnosis = String::from_str(&env, "J20.9");
    let valid_hash = create_test_evidence_hash(&env);

    let result = contract.validate_claim_parameters(&valid_amount, &valid_diagnosis, &valid_hash);
    assert_eq!(result, true);

    // Test invalid amount
    let invalid_amount = -100;
    let result = contract.validate_claim_parameters(&invalid_amount, &valid_diagnosis, &valid_hash);
    assert_eq!(result, false);
}

#[test]
fn test_calculate_approval_percentage() {
    let env = Env::default();
    let contract = create_test_contract(&env);

    let claim_amount = 10000;
    let approved_amount = 8000;

    let percentage = contract.calculate_approval_percentage(&claim_amount, &approved_amount);
    assert_eq!(percentage, 80);
}

#[test]
fn test_calculate_processing_fee() {
    let env = Env::default();
    let contract = create_test_contract(&env);

    let claim_amount = 10000;
    let fee_basis_points = 50; // 0.5%

    let fee = contract.calculate_processing_fee(&claim_amount, &fee_basis_points);
    assert_eq!(fee, 50);
}

#[test]
fn test_get_pending_queue() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    let admin = Address::generate(&env);
    let patient = Address::generate(&env);
    let insurer = Address::generate(&env);
    let currency = Address::generate(&env);

    contract.initialize(&admin, &10000);

    let diagnosis_code = String::from_str(&env, "J20.9");
    let evidence_hash = create_test_evidence_hash(&env);

    // Submit multiple claims
    contract.submit_claim(&patient, &insurer, &1000, &currency, &diagnosis_code, &evidence_hash);
    contract.submit_claim(&patient, &insurer, &2000, &currency, &diagnosis_code, &evidence_hash);

    // Check pending queue
    let pending = contract.get_pending_queue();
    assert_eq!(pending.len(), 2);
}

#[test]
fn test_full_claim_lifecycle() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    let admin = Address::generate(&env);
    let patient = Address::generate(&env);
    let insurer = Address::generate(&env);
    let currency = Address::generate(&env);

    // Initialize contract
    contract.initialize(&admin, &10000);

    // Submit claim
    let claim_amount = 5000;
    let diagnosis_code = String::from_str(&env, "J20.9");
    let evidence_hash = create_test_evidence_hash(&env);

    let claim_id = contract.submit_claim(
        &patient,
        &insurer,
        &claim_amount,
        &currency,
        &diagnosis_code,
        &evidence_hash,
    );

    // Process claim (auto-approve)
    contract.process_claim(&claim_id, &insurer);

    // Execute payout
    contract.payout(&claim_id, &insurer);

    // Verify final status
    let claim = contract.get_claim(&claim_id).unwrap();
    assert_eq!(claim.status, claims::ClaimStatus::Paid);
    assert_eq!(claim.auto_approved, true);

    let payout_record = contract.get_payout_record(&claim_id).unwrap();
    assert_eq!(payout_record.amount, claim_amount);
}
