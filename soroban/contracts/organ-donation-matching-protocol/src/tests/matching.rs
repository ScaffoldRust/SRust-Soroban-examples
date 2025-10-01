
#![cfg(test)]

extern crate std;

use soroban_sdk::{testutils::{Address as _}, Address, Env, String, log, Vec};
use crate::tests::utils::{setup_test_environment, create_test_hla_profile, create_test_hla_profile_with_alleles};
use crate::{BloodType, OrganType, UrgencyLevel, HLAProfile, RecipientProfile, DonorProfile, MatchResult};
use crate::matching::*;

#[test]
fn test_find_match() {
    let (env, client, admin) = setup_test_environment();
    let donor = Address::generate(&env);
    let second_donor = Address::generate(&env);
    let recipient = Address::generate(&env);
    let medical_facility = Address::generate(&env);
    let max_donors = 100;
    let max_recipients = 500;
    let urgency_weight = 70;
    let compatibility_threshold = 70;
    client.initialize(&admin, &max_donors, &max_recipients, &urgency_weight, &compatibility_threshold);

    let hla_profile = create_test_hla_profile(&env);
    let consent_hash = String::from_str(&env, "test_consent_hash_1234567890123456");

    client.register_donor(
        &donor,
        &BloodType::O,
        &OrganType::Kidney,
        &hla_profile,
        &35,
        &medical_facility,
        &consent_hash,
    );

    // Register compatible donor
    client.register_donor(
        &second_donor,
        &BloodType::A, // Universal donor
        &OrganType::Kidney,
        &hla_profile.clone(),
        &35,
        &medical_facility,
        &consent_hash,
    );

    // Register recipient
    client.register_recipient(
        &recipient,
        &BloodType::A, // Can receive from O
        &OrganType::Kidney,
        &hla_profile,
        &45,
        &UrgencyLevel::High,
        &medical_facility,
        &750,
    );

        // Find matches
        let matches = client.find_match(&recipient);
        assert_eq!(matches.len(), 2);
        log!(&env, "matches: {}", matches);
}

#[test]
fn test_find_match_no_match() {
    let (env, client, admin) = setup_test_environment();
    let donor = Address::generate(&env);
    let second_donor = Address::generate(&env);
    let recipient = Address::generate(&env);
    let medical_facility = Address::generate(&env);
    let max_donors = 100;
    let max_recipients = 500;
    let urgency_weight = 70;
    let compatibility_threshold = 70;
    client.initialize(&admin, &max_donors, &max_recipients, &urgency_weight, &compatibility_threshold);

    let hla_profile = create_test_hla_profile(&env);
    let consent_hash = String::from_str(&env, "test_consent_hash_1234567890123456");

    client.register_donor(
        &donor,
        &BloodType::AB,
        &OrganType::Kidney,
        &hla_profile,
        &25,
        &medical_facility,
        &consent_hash,
    );

    // Register compatible donor
    client.register_donor(
        &second_donor,
        &BloodType::A, // Universal donor
        &OrganType::Kidney,
        &hla_profile.clone(),
        &35,
        &medical_facility,
        &consent_hash,
    );

    // Register recipient
    client.register_recipient(
        &recipient,
        &BloodType::B, // Can receive from O
        &OrganType::Kidney,
        &hla_profile,
        &45,
        &UrgencyLevel::High,
        &medical_facility,
        &750,
    );

    let matches = client.find_match(&recipient);
    assert_eq!(matches.len(), 0);
}

#[test]
fn test_incompatible_blood_type() {
    let (env, client, admin) = setup_test_environment();
    let donor = Address::generate(&env);
    let recipient = Address::generate(&env);
    let medical_facility = Address::generate(&env);
    let hla_profile = create_test_hla_profile(&env);
    let consent_hash = String::from_str(&env, "test_consent_hash_1234567890123456");
    client.initialize(&admin, &100, &500, &70, &70);

    // Register donor with BloodType::A
    client.register_donor(&donor, &BloodType::A, &OrganType::Kidney, &hla_profile, &35, &medical_facility, &consent_hash);

    // Register recipient with BloodType::B (incompatible)
    // client.register_recipient(&recipient, &BloodType::B, &OrganType::Kidney, &hla_profile, &35, &UrgencyLevel::High, &medical_facility, &750);
    client.register_recipient(
        &recipient,
        &BloodType::B,
        &OrganType::Kidney,
        &hla_profile,
        &45,
        &UrgencyLevel::High,
        &medical_facility,
        &750,
    );

    let matches = client.find_match(&recipient);
    assert_eq!(matches.len(), 0); // No matches due to blood type incompatibility
}


#[test]
fn test_different_organ_type() {
    let (env, client, admin) = setup_test_environment();
    let donor = Address::generate(&env);
    let recipient = Address::generate(&env);
    let medical_facility = Address::generate(&env);
    let hla_profile = create_test_hla_profile(&env);
    let consent_hash = String::from_str(&env, "test_consent_hash_1234567890123456");
    client.initialize(&admin, &100, &500, &70, &70);

    // Register donor with OrganType::Heart
    client.register_donor(&donor, &BloodType::O, &OrganType::Heart, &hla_profile, &35, &medical_facility, &consent_hash);

    // Register recipient with OrganType::Kidney
    client.register_recipient(&recipient, &BloodType::O, &OrganType::Kidney, &hla_profile, &35, &UrgencyLevel::High, &medical_facility, &750);

    let matches = client.find_match(&recipient);
    assert_eq!(matches.len(), 0); // No matches due to organ type mismatch
}



#[test]
fn test_partial_hla_match() {
    let (env, client, admin) = setup_test_environment();
    let donor = Address::generate(&env);
    let recipient = Address::generate(&env);
    let medical_facility = Address::generate(&env);
    let consent_hash = String::from_str(&env, "test_consent_hash_1234567890123456");
    client.initialize(&admin, &100, &500, &70, &70);

    // Donor with partial HLA match
    let donor_hla = create_test_hla_profile_with_alleles(
        &env,
        &["A*02:01", "A*03:01"], // One match, one mismatch
        &["B*07:02", "B*44:02"], // One match, one mismatch
        &["DRB1*15:01", "DRB1*04:01"], // One match, one mismatch
    );
    client.register_donor(&donor, &BloodType::O, &OrganType::Kidney, &donor_hla, &35, &medical_facility, &consent_hash);

    // Recipient with test HLA profile
    let recipient_hla = create_test_hla_profile(&env); // A*02:01, A*01:01, B*07:02, B*08:01, DRB1*15:01, DRB1*03:01
    client.register_recipient(&recipient, &BloodType::O, &OrganType::Kidney, &recipient_hla, &35, &UrgencyLevel::High, &medical_facility, &750);

    let matches = client.find_match(&recipient);
    assert_eq!(matches.len(), 1);
    let match_result = matches.get(0).unwrap();
    let hla_score = hla_compatibility_score(&donor_hla, &recipient_hla);
    assert_eq!(hla_score, 50); // 3/6 alleles match
}

#[test]
fn test_partial_hla_match_2() {
    let (env, client, admin) = setup_test_environment();
    let donor = Address::generate(&env);
    let recipient = Address::generate(&env);
    let medical_facility = Address::generate(&env);
    let consent_hash = String::from_str(&env, "test_consent_hash_1234567890123456");
    client.initialize(&admin, &100, &500, &70, &70);

    // Donor with partial HLA match
    let donor_hla = create_test_hla_profile_with_alleles(
        &env,
        &["A*02:01", "A*03:01"], // One match, one mismatch
        &["B*04:02", "B*44:02"], // two mismatch
        &["DRB1*03:01", "DRB1*04:01"], // two mismatch
    );
    client.register_donor(&donor, &BloodType::O, &OrganType::Kidney, &donor_hla, &35, &medical_facility, &consent_hash);

    // Recipient with test HLA profile
    let recipient_hla = create_test_hla_profile(&env); // A*02:01, A*01:01, B*07:02, B*08:01, DRB1*15:01, DRB1*03:01
    client.register_recipient(&recipient, &BloodType::O, &OrganType::Kidney, &recipient_hla, &35, &UrgencyLevel::High, &medical_facility, &750);

    let matches = client.find_match(&recipient);
    assert_eq!(matches.len(), 1);
    let match_result = matches.get(0).unwrap();
    let hla_score = hla_compatibility_score(&donor_hla, &recipient_hla);
    assert_eq!(hla_score, 33); // 2/6 alleles match
}

#[test]
fn test_blood_compatibility_score() {
    let env = Env::default();

    // Test perfect matches
    assert_eq!(blood_compatibility_score(&BloodType::A, &BloodType::A), 100);
    assert_eq!(blood_compatibility_score(&BloodType::B, &BloodType::B), 100);
    assert_eq!(blood_compatibility_score(&BloodType::AB, &BloodType::AB), 100);
    assert_eq!(blood_compatibility_score(&BloodType::O, &BloodType::O), 100);

    // Test universal donor (O)
    assert_eq!(blood_compatibility_score(&BloodType::O, &BloodType::A), 95);
    assert_eq!(blood_compatibility_score(&BloodType::O, &BloodType::B), 95);
    assert_eq!(blood_compatibility_score(&BloodType::O, &BloodType::AB), 95);

    // Test A and B to AB
    assert_eq!(blood_compatibility_score(&BloodType::A, &BloodType::AB), 90);
    assert_eq!(blood_compatibility_score(&BloodType::B, &BloodType::AB), 90);

    // Test incompatible combinations
    assert_eq!(blood_compatibility_score(&BloodType::A, &BloodType::B), 0);
    assert_eq!(blood_compatibility_score(&BloodType::B, &BloodType::A), 0);
    assert_eq!(blood_compatibility_score(&BloodType::AB, &BloodType::A), 0);
    assert_eq!(blood_compatibility_score(&BloodType::AB, &BloodType::B), 0);
}

#[test]
fn test_age_compatibility_score() {
    // Test perfect age match
    assert_eq!(age_compatibility_score(40, 40, &OrganType::Kidney), 100);

    // Test within half tolerance (Kidney: max 15 years)
    assert_eq!(age_compatibility_score(40, 45, &OrganType::Kidney), 65); // Diff 5: 90 - (5 * 5)
    assert_eq!(age_compatibility_score(40, 47, &OrganType::Kidney), 55); // Diff 7: 90 - (7 * 5)

    // Test within full tolerance (Kidney: max 15 years)
    assert_eq!(age_compatibility_score(40, 50, &OrganType::Kidney), 40); // Diff 10: 60 - (10 * 2)

    // Test beyond tolerance (Kidney)
    assert_eq!(age_compatibility_score(40, 60, &OrganType::Kidney), 20); // Suboptimal

    // Test strict organ (Heart: max 10 years)
    assert_eq!(age_compatibility_score(40, 45, &OrganType::Heart), 65); // Diff 5: 90 - (5 * 5)
    assert_eq!(age_compatibility_score(40, 51, &OrganType::Heart), 20); // Beyond 10 years
}

#[test]
fn test_calculate_priority_score() {
    let env = Env::default();
    let recipient = RecipientProfile {
        address: Address::generate(&env),
        blood_type: BloodType::O,
        organ_type: OrganType::Kidney,
        hla_profile: create_test_hla_profile(&env),
        age: 40,
        urgency_level: UrgencyLevel::High,
        medical_facility: Address::generate(&env),
        medical_priority_score: 500,
        registered_at: 0,
        wait_time_days:100,
        is_active: true
    };

    // Test urgency levels with wait time and weight
    let urgency_weight = 70;
    let score = calculate_priority_score(&recipient, recipient.wait_time_days, urgency_weight);
    // High urgency: 750 * 70/100 = 525, wait_time: 100 * 2 = 200, medical: 500
    assert_eq!(score, 525 + 200 + 500); // 1225

    // Test critical urgency
    let critical_recipient = RecipientProfile {
        urgency_level: UrgencyLevel::Critical,
        ..recipient.clone()
    };
    let critical_score = calculate_priority_score(&critical_recipient, recipient.wait_time_days, urgency_weight);
    // Critical: 1000 * 70/100 = 700, wait_time: 200, medical: 500
    assert_eq!(critical_score, 700 + 200 + 500); // 1400

    // Test max wait time (capped at 365 days)
    let long_wait_score = calculate_priority_score(&recipient, 500, urgency_weight);
    // High: 525, wait_time: 365 * 2 = 730, medical: 500
    assert_eq!(long_wait_score, 525 + 730 + 500); // 1755
}

#[test]
fn test_calculate_compatibility_score() {
    let env = Env::default();
    let donor = DonorProfile {
        address: Address::generate(&env),
        blood_type: BloodType::O,
        organ_type: OrganType::Kidney,
        hla_profile: create_test_hla_profile(&env),
        age: 35,
        medical_facility: Address::generate(&env),
        consent_hash: String::from_str(&env, "test_hash"),
        is_active: true,
        registered_at: 10,
    };
    let recipient = RecipientProfile {
        address: Address::generate(&env),
        blood_type: BloodType::A,
        organ_type: OrganType::Kidney,
        hla_profile: create_test_hla_profile(&env),
        age: 45,
        urgency_level: UrgencyLevel::High,
        medical_facility: Address::generate(&env),
        medical_priority_score: 500,
        registered_at: 0,
        wait_time_days:100,
        is_active: true
    };

    // Test compatible match
    let score = calculate_compatibility_score(&donor, &recipient);
    // Blood: O -> A = 95 (40%), HLA: identical = 100 (35%), Age: diff 10 for Kidney = 40 (25%)
    // (95 * 40 + 100 * 35 + 40 * 25) / 100 = (3800 + 3500 + 1000) / 100 = 83
    assert_eq!(score, 83);

    // Test incompatible blood type
    let incompatible_donor = DonorProfile {
        blood_type: BloodType::B,
        ..donor.clone()
    };
    assert_eq!(calculate_compatibility_score(&incompatible_donor, &recipient), 0);
}

#[test]
fn test_organ_type_match() {
    let env = Env::default();
    let donor = DonorProfile {
        address: Address::generate(&env),
        blood_type: BloodType::O,
        organ_type: OrganType::Kidney,
        hla_profile: create_test_hla_profile(&env),
        age: 35,
        medical_facility: Address::generate(&env),
        consent_hash: String::from_str(&env, "test_hash"),
        is_active: true,
        registered_at: 10,
    };
    let recipient = RecipientProfile {
        address: Address::generate(&env),
        blood_type: BloodType::A,
        organ_type: OrganType::Kidney,
        hla_profile: create_test_hla_profile(&env),
        age: 45,
        urgency_level: UrgencyLevel::High,
        medical_facility: Address::generate(&env),
        medical_priority_score: 500,
        registered_at: 0,
        wait_time_days:100,
        is_active: true
    };

    assert!(organ_type_match(&donor, &recipient));
    let mismatched_recipient = RecipientProfile {
        organ_type: OrganType::Heart,
        ..recipient.clone()
    };
    assert!(!organ_type_match(&donor, &mismatched_recipient));
}

#[test]
fn test_sort_matches_by_priority() {
    let env = Env::default();
    let mut matches = Vec::new(&env);
    let medical_facility = Address::generate(&env);

    // Create three match results
    let match1 = MatchResult {
        match_id: 1,
        donor: Address::generate(&env),
        recipient: Address::generate(&env),
        compatibility_score: 80,
        priority_score: 1000,
        matched_at: 0,
        confirmed: false,
        medical_facility: medical_facility.clone(),
    };
    let match2 = MatchResult {
        match_id: 2,
        compatibility_score: 85,
        priority_score: 1000, // Same priority as match1, higher compatibility
        ..match1.clone()
    };
    let match3 = MatchResult {
        match_id: 3,
        compatibility_score: 75,
        priority_score: 1200, // Higher priority
        ..match1.clone()
    };

    matches.push_back(match1.clone());
    matches.push_back(match2.clone());
    matches.push_back(match3.clone());

    sort_matches_by_priority(&env, &mut matches);

    // Expected order: match3 (1200 priority), match2 (1000 priority, 85 compat), match1 (1000 priority, 80 compat)
    assert_eq!(matches.get(0).unwrap().match_id, 3);
    assert_eq!(matches.get(1).unwrap().match_id,2);
    assert_eq!(matches.get(2).unwrap().match_id, 1);
}

#[test]
fn test_check_crossmatch_compatibility() {
    let env = Env::default();
    let hla1 = create_test_hla_profile(&env);
    let hla2 = create_test_hla_profile(&env);
    assert!(check_crossmatch_compatibility(&hla1, &hla2)); // Score=100, passes threshold 60

    // Test failing crossmatch
    let mut hla_low = HLAProfile {
        hla_a: Vec::new(&env),
        hla_b: Vec::new(&env),
        hla_dr: Vec::new(&env),
    };
    hla_low.hla_a.push_back(String::from_str(&env, "A*03:01")); // No match
    hla_low.hla_b.push_back(String::from_str(&env, "B*44:02")); // No match
    hla_low.hla_dr.push_back(String::from_str(&env, "DRB1*04:01")); // No match
    assert!(!check_crossmatch_compatibility(&hla1, &hla_low)); // Score=0, fails threshold
}

#[test]
fn test_estimate_viability_remaining() {
    let env = Env::default();
    let procurement_time = 1000;
    let current_time = 1000 + 2 * 3600; // 2 hours later

    // Test Heart (4 hours viability)
    assert_eq!(estimate_viability_remaining(&OrganType::Heart, procurement_time, current_time), 2 * 3600); // 2 hours left
    assert_eq!(estimate_viability_remaining(&OrganType::Heart, procurement_time, procurement_time + 5 * 3600), 0); // Expired

    // Test Kidney (24 hours viability)
    assert_eq!(estimate_viability_remaining(&OrganType::Kidney, procurement_time, current_time), 22 * 3600); // 22 hours left
}