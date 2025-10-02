use soroban_sdk::{testutils::Address as _, Address, String, Vec};
use crate::tests::utils::*;

#[cfg(test)]
mod claim_processing_tests {
    use super::*;

    #[test]
    fn test_automatic_claim_approval_below_threshold() {
        let env = create_test_env();
        let (patient, insurer, provider) = generate_test_addresses(&env);

        // Setup low-value claim for auto-approval
        let mut claim = create_valid_claim(&env, patient.clone(), insurer.clone(), provider.clone());
        claim.amount = 30000; // $300 - below $500 auto-approve threshold

        let rules = create_valid_insurer_rules(&env, insurer.clone());

        let result = simulate_claim_processing_result(&claim, &rules);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), String::from_str(&env, "auto_approved"));
    }

    #[test]
    fn test_manual_review_above_auto_approve_threshold() {
        let env = create_test_env();
        let (patient, insurer, provider) = generate_test_addresses(&env);

        // Setup medium-value claim requiring manual review
        let mut claim = create_valid_claim(&env, patient.clone(), insurer.clone(), provider.clone());
        claim.amount = 75000; // $750 - above $500 auto-approve, below $1000 pre-auth
        claim.pre_authorization = false;

        let rules = create_valid_insurer_rules(&env, insurer.clone());

        let result = simulate_claim_processing_result(&claim, &rules);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), String::from_str(&env, "approved"));
    }

    #[test]
    fn test_pre_authorization_required_rejection() {
        let env = create_test_env();
        let (patient, insurer, provider) = generate_test_addresses(&env);

        let mut claim = create_high_value_claim(&env, patient.clone(), insurer.clone(), provider.clone());
        claim.pre_authorization = false; 

        let rules = create_valid_insurer_rules(&env, insurer.clone());

        // Test claim processing should fail
        let result = simulate_claim_processing_result(&claim, &rules);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), String::from_str(&env, "Pre-authorization required"));
    }

    #[test]
    fn test_claim_approval_with_valid_pre_authorization() {
        let env = create_test_env();
        let (patient, insurer, provider) = generate_test_addresses(&env);

        // Setup high-value claim with pre-authorization
        let claim = create_high_value_claim(&env, patient.clone(), insurer.clone(), provider.clone());
        assert!(claim.pre_authorization);

        let rules = create_valid_insurer_rules(&env, insurer.clone());

        // Test claim processing should succeed
        let result = simulate_claim_processing_result(&claim, &rules);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), String::from_str(&env, "approved"));
    }

    #[test]
    fn test_claim_rejection_exceeding_maximum_amount() {
        let env = create_test_env();
        let (patient, insurer, provider) = generate_test_addresses(&env);

        // Setup claim exceeding maximum allowed amount
        let mut claim = create_valid_claim(&env, patient.clone(), insurer.clone(), provider.clone());
        claim.amount = 15000000; // $150,000 - exceeds $100,000 max

        let rules = create_valid_insurer_rules(&env, insurer.clone());

        // Test claim processing should fail
        let result = simulate_claim_processing_result(&claim, &rules);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), String::from_str(&env, "Claim exceeds maximum allowed amount"));
    }

    #[test]
    fn test_unsupported_treatment_type_rejection() {
        let env = create_test_env();
        let (patient, insurer, provider) = generate_test_addresses(&env);

        // Setup claim with unsupported treatment type
        let mut claim = create_valid_claim(&env, patient.clone(), insurer.clone(), provider.clone());
        claim.treatment_type = String::from_str(&env, "cosmetic_surgery"); // Not in supported list

        let rules = create_valid_insurer_rules(&env, insurer.clone());

        // Test claim processing should fail
        let result = simulate_claim_processing_result(&claim, &rules);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), String::from_str(&env, "Treatment type not covered"));
    }

    #[test]
    fn test_processing_with_invalid_insurer_rules() {
        let env = create_test_env();
        let (patient, insurer, provider) = generate_test_addresses(&env);

        let claim = create_valid_claim(&env, patient.clone(), insurer.clone(), provider.clone());
        let invalid_rules = create_invalid_insurer_rules(&env, insurer.clone());

        // Test claim processing should fail due to invalid rules
        let result = simulate_claim_processing_result(&claim, &invalid_rules);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), String::from_str(&env, "Invalid insurer rules"));
    }

    #[test]
    fn test_insurer_rules_validation() {
        let env = create_test_env();
        let insurer = Address::generate(&env);

        let valid_rules = create_valid_insurer_rules(&env, insurer.clone());
        assert!(validate_insurer_rules(&valid_rules));

        let invalid_rules = create_invalid_insurer_rules(&env, insurer.clone());
        assert!(!validate_insurer_rules(&invalid_rules));

        // Test edge cases
        let mut edge_case_rules = valid_rules.clone();
        edge_case_rules.auto_approve_threshold = edge_case_rules.required_pre_auth_threshold + 1;
        assert!(!validate_insurer_rules(&edge_case_rules));
    }

    #[test]
    fn test_claim_processing_workflow_states() {
        let env = create_test_env();
        let (patient, insurer, provider) = generate_test_addresses(&env);

        let rules = create_valid_insurer_rules(&env, insurer.clone());

        // Test different claim amounts and their processing paths
        let mut test_cases = Vec::new(&env);
        test_cases.push_back((25000, String::from_str(&env, "auto_approved")));    // $250 - auto approve
        test_cases.push_back((75000, String::from_str(&env, "approved")));         // $750 - manual review
        test_cases.push_back((150000, String::from_str(&env, "approved")));        // $1500 - with pre-auth (we'll set it)

        for i in 0..test_cases.len() {
            let (amount, expected_status) = test_cases.get(i).unwrap();
            let mut claim = create_valid_claim(&env, patient.clone(), insurer.clone(), provider.clone());
            claim.amount = amount;
            claim.pre_authorization = amount >= rules.required_pre_auth_threshold;

            let result = simulate_claim_processing_result(&claim, &rules);
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), expected_status);
        }
    }

    #[test]
    fn test_claim_processing_with_multiple_insurers() {
        let env = create_test_env();
        let (patient, _, provider) = generate_test_addresses(&env);

        // Create multiple insurers with different rules
        let insurer1 = Address::generate(&env);
        let insurer2 = Address::generate(&env);

        let mut rules1 = create_valid_insurer_rules(&env, insurer1.clone());
        rules1.auto_approve_threshold = 100000; // $1000

        let mut rules2 = create_valid_insurer_rules(&env, insurer2.clone());
        rules2.auto_approve_threshold = 25000; // $250

        // Same claim amount, different outcomes based on insurer rules
        let amount = 50000; // $500
        let claim1 = MockClaim {
            id: 1,
            patient: patient.clone(),
            insurer: insurer1.clone(),
            provider: provider.clone(),
            treatment_type: String::from_str(&env, "consultation"),
            amount,
            diagnosis_code: String::from_str(&env, "Z00.00"),
            evidence_hash: String::from_str(&env, "evidence_123"),
            pre_authorization: false,
            submitted_at: env.ledger().timestamp(),
        };

        let claim2 = MockClaim {
            id: 2,
            patient: patient.clone(),
            insurer: insurer2.clone(),
            provider: provider.clone(),
            treatment_type: String::from_str(&env, "consultation"),
            amount,
            diagnosis_code: String::from_str(&env, "Z00.00"),
            evidence_hash: String::from_str(&env, "evidence_456"),
            pre_authorization: false,
            submitted_at: env.ledger().timestamp(),
        };

        // Process with insurer1 rules (should be auto-approved)
        let result1 = simulate_claim_processing_result(&claim1, &rules1);
        assert!(result1.is_ok());
        assert_eq!(result1.unwrap(), String::from_str(&env, "auto_approved"));

        // Process with insurer2 rules (should need manual review)
        let result2 = simulate_claim_processing_result(&claim2, &rules2);
        assert!(result2.is_ok());
        assert_eq!(result2.unwrap(), String::from_str(&env, "approved"));
    }

    #[test]
    fn test_claim_processing_edge_cases() {
        let env = create_test_env();
        let (patient, insurer, provider) = generate_test_addresses(&env);

        let rules = create_valid_insurer_rules(&env, insurer.clone());

        // Test claim at exact threshold boundaries
        let mut boundary_amounts = Vec::new(&env);
        boundary_amounts.push_back(rules.auto_approve_threshold);     // Exact auto-approve limit
        boundary_amounts.push_back(rules.auto_approve_threshold + 1); // Just above auto-approve
        boundary_amounts.push_back(rules.required_pre_auth_threshold); // Exact pre-auth threshold
        boundary_amounts.push_back(rules.max_claim_amount);           // Maximum allowed amount

        for i in 0..boundary_amounts.len() {
            let amount = boundary_amounts.get(i).unwrap();
            let mut claim = create_valid_claim(&env, patient.clone(), insurer.clone(), provider.clone());
            claim.amount = amount;
            claim.pre_authorization = amount >= rules.required_pre_auth_threshold;

            let result = simulate_claim_processing_result(&claim, &rules);
            // All should succeed with proper pre-auth
            assert!(result.is_ok(), "Failed for amount: {}", amount);
        }
    }

    #[test]
    fn test_claim_processing_compliance_validation() {
        let env = create_test_env();
        let (patient, insurer, provider) = generate_test_addresses(&env);

        let rules = create_valid_insurer_rules(&env, insurer.clone());

        // Test claims with valid treatment types from supported list
        let mut supported_treatments = Vec::new(&env);
        supported_treatments.push_back(String::from_str(&env, "consultation"));
        supported_treatments.push_back(String::from_str(&env, "surgery"));
        supported_treatments.push_back(String::from_str(&env, "diagnostic"));
        supported_treatments.push_back(String::from_str(&env, "emergency"));

        for i in 0..supported_treatments.len() {
            let treatment = supported_treatments.get(i).unwrap();
            let mut claim = create_valid_claim(&env, patient.clone(), insurer.clone(), provider.clone());
            claim.treatment_type = treatment;
            claim.amount = 30000; // Low amount for auto-approval

            let result = simulate_claim_processing_result(&claim, &rules);
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_claim_processing_performance_metrics() {
        let env = create_test_env();
        let (_patient, insurer, _provider) = generate_test_addresses(&env);

        let rules = create_valid_insurer_rules(&env, insurer.clone());

        // Generate large batch for performance testing
        let start_time = env.ledger().timestamp();
        let claims = generate_multiple_claims(&env, 1000);

        let mut processing_times = Vec::new(&env);

        let test_count = if claims.len() > 100 { 100 } else { claims.len() };
        for i in 0..test_count { // Test first 100 for performance
            let claim = claims.get(i).unwrap();
            let mut test_claim = claim.clone();
            test_claim.treatment_type = String::from_str(&env, "consultation");
            test_claim.insurer = insurer.clone();
            test_claim.pre_authorization = test_claim.amount >= rules.required_pre_auth_threshold;

            let process_start = env.ledger().timestamp();
            let _result = simulate_claim_processing_result(&test_claim, &rules);
            let process_end = env.ledger().timestamp();

            processing_times.push_back(process_end - process_start);
        }

        let end_time = env.ledger().timestamp();
        let total_time = end_time - start_time;

        // Verify processing completed in reasonable time
        assert!(total_time < 300, "Processing took too long: {} seconds", total_time);
        assert_eq!(processing_times.len(), 100);
    }

    #[test]
    fn test_inactive_insurer_rules_handling() {
        let env = create_test_env();
        let (patient, insurer, provider) = generate_test_addresses(&env);

        // Create inactive insurer rules
        let mut rules = create_valid_insurer_rules(&env, insurer.clone());
        rules.active = false;

        let claim = create_valid_claim(&env, patient.clone(), insurer.clone(), provider.clone());

        // Test claim processing should fail with inactive rules
        let result = simulate_claim_processing_result(&claim, &rules);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), String::from_str(&env, "Invalid insurer rules"));
    }

    #[test]
    fn test_claim_processing_audit_requirements() {
        let env = create_test_env();
        let (patient, insurer, provider) = generate_test_addresses(&env);

        let rules = create_valid_insurer_rules(&env, insurer.clone());
        let claim = create_valid_claim(&env, patient.clone(), insurer.clone(), provider.clone());

        // Verify all required audit fields are present for regulatory compliance
        assert!(claim.submitted_at > 0);
        assert!(!claim.diagnosis_code.is_empty());
        assert!(!claim.evidence_hash.is_empty());
        assert_eq!(claim.insurer, insurer);
        assert_eq!(claim.patient, patient);
        assert_eq!(claim.provider, provider);

        // Test processing creates auditable result
        let result = simulate_claim_processing_result(&claim, &rules);
        assert!(result.is_ok());

        // Verify result contains processing decision
        let status = result.unwrap();
        assert!(status == String::from_str(&env, "auto_approved") || status == String::from_str(&env, "approved"));
    }
}