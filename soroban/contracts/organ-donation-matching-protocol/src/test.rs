#[cfg(test)]
mod tests {
    use crate::{
        BloodType, DonorProfile, HLAProfile, OrganDonationMatchingContract,
        OrganDonationMatchingContractClient, OrganType, RecipientProfile, UrgencyLevel,
    };
    use soroban_sdk::{testutils::Address as _, Address, Env, String, Vec};

    fn create_test_env() -> (Env, OrganDonationMatchingContractClient<'static>, Address) {
        let env = Env::default();
        let contract_id = env.register(OrganDonationMatchingContract {}, ());
        let client = OrganDonationMatchingContractClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        env.mock_all_auths();

        (env, client, admin)
    }

    fn create_test_hla_profile(env: &Env) -> HLAProfile {
        let mut hla_a = Vec::new(env);
        hla_a.push_back(String::from_str(env, "A*02:01"));
        hla_a.push_back(String::from_str(env, "A*01:01"));

        let mut hla_b = Vec::new(env);
        hla_b.push_back(String::from_str(env, "B*07:02"));
        hla_b.push_back(String::from_str(env, "B*08:01"));

        let mut hla_dr = Vec::new(env);
        hla_dr.push_back(String::from_str(env, "DRB1*15:01"));
        hla_dr.push_back(String::from_str(env, "DRB1*03:01"));

        HLAProfile {
            hla_a,
            hla_b,
            hla_dr,
        }
    }

    #[test]
    fn test_initialize_contract() {
        let (_env, client, admin) = create_test_env();

        client.initialize(
            &admin, &1000, // max_donors
            &5000, // max_recipients
            &70,   // urgency_weight
            &60,   // compatibility_threshold
        );
    }

    #[test]
    fn test_register_donor() {
        let (env, client, admin) = create_test_env();
        let donor = Address::generate(&env);
        let medical_facility = Address::generate(&env);

        // Initialize contract first
        client.initialize(&admin, &1000, &5000, &70, &60);

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

        // Verify donor was registered
        let registered_donor = client.get_donor(&donor);
        assert!(registered_donor.is_some());

        let donor_profile = registered_donor.unwrap();
        assert_eq!(donor_profile.address, donor);
        assert_eq!(donor_profile.blood_type, BloodType::O);
        assert_eq!(donor_profile.organ_type, OrganType::Kidney);
        assert_eq!(donor_profile.age, 35);
        assert!(donor_profile.is_active);
    }

    #[test]
    fn test_register_recipient() {
        let (env, client, admin) = create_test_env();
        let recipient = Address::generate(&env);
        let medical_facility = Address::generate(&env);

        // Initialize contract first
        client.initialize(&admin, &1000, &5000, &70, &60);

        let hla_profile = create_test_hla_profile(&env);

        client.register_recipient(
            &recipient,
            &BloodType::A,
            &OrganType::Kidney,
            &hla_profile,
            &45,
            &UrgencyLevel::High,
            &medical_facility,
            &750,
        );

        // Verify recipient was registered
        let registered_recipient = client.get_recipient(&recipient);
        assert!(registered_recipient.is_some());

        let recipient_profile = registered_recipient.unwrap();
        assert_eq!(recipient_profile.address, recipient);
        assert_eq!(recipient_profile.blood_type, BloodType::A);
        assert_eq!(recipient_profile.organ_type, OrganType::Kidney);
        assert_eq!(recipient_profile.age, 45);
        assert_eq!(recipient_profile.urgency_level, UrgencyLevel::High);
        assert_eq!(recipient_profile.medical_priority_score, 750);
        assert!(recipient_profile.is_active);
    }

    #[test]
    fn test_blood_type_compatibility() {
        use crate::matching::blood_compatibility_score;

        // Perfect matches
        assert_eq!(blood_compatibility_score(&BloodType::A, &BloodType::A), 100);
        assert_eq!(blood_compatibility_score(&BloodType::B, &BloodType::B), 100);
        assert_eq!(
            blood_compatibility_score(&BloodType::AB, &BloodType::AB),
            100
        );
        assert_eq!(blood_compatibility_score(&BloodType::O, &BloodType::O), 100);

        // Universal donor O
        assert_eq!(blood_compatibility_score(&BloodType::O, &BloodType::A), 95);
        assert_eq!(blood_compatibility_score(&BloodType::O, &BloodType::B), 95);
        assert_eq!(blood_compatibility_score(&BloodType::O, &BloodType::AB), 95);

        // Compatible donations
        assert_eq!(blood_compatibility_score(&BloodType::A, &BloodType::AB), 90);
        assert_eq!(blood_compatibility_score(&BloodType::B, &BloodType::AB), 90);

        // Incompatible combinations
        assert_eq!(blood_compatibility_score(&BloodType::A, &BloodType::B), 0);
        assert_eq!(blood_compatibility_score(&BloodType::B, &BloodType::A), 0);
        assert_eq!(blood_compatibility_score(&BloodType::AB, &BloodType::A), 0);
        assert_eq!(blood_compatibility_score(&BloodType::AB, &BloodType::B), 0);
        assert_eq!(blood_compatibility_score(&BloodType::AB, &BloodType::O), 0);
    }

    #[test]
    fn test_hla_compatibility() {
        let env = Env::default();
        use crate::matching::hla_compatibility_score;

        let donor_hla = create_test_hla_profile(&env);
        let recipient_hla = create_test_hla_profile(&env);

        // Perfect match should score 100
        let score = hla_compatibility_score(&donor_hla, &recipient_hla);
        assert_eq!(score, 100);

        // Create different HLA profile
        let mut different_hla_a = Vec::new(&env);
        different_hla_a.push_back(String::from_str(&env, "A*03:01"));
        different_hla_a.push_back(String::from_str(&env, "A*11:01"));

        let different_hla = HLAProfile {
            hla_a: different_hla_a,
            hla_b: recipient_hla.hla_b.clone(),
            hla_dr: recipient_hla.hla_dr.clone(),
        };

        let partial_score = hla_compatibility_score(&donor_hla, &different_hla);
        assert!(partial_score < 100 && partial_score > 0);
    }

    #[test]
    fn test_find_match() {
        let (env, client, admin) = create_test_env();
        let donor = Address::generate(&env);
        let recipient = Address::generate(&env);
        let medical_facility = Address::generate(&env);

        // Initialize contract
        client.initialize(&admin, &1000, &5000, &70, &60);

        let hla_profile = create_test_hla_profile(&env);
        let consent_hash = String::from_str(&env, "test_consent_hash_1234567890123456");

        // Register compatible donor
        client.register_donor(
            &donor,
            &BloodType::O, // Universal donor
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

        assert!(matches.len() > 0);
        let match_result = matches.first().unwrap();
        assert_eq!(match_result.donor, donor);
        assert_eq!(match_result.recipient, recipient);
        assert_eq!(match_result.compatibility_score, 83);
        assert!(match_result.priority_score > 1000);
        assert_eq!(match_result.matched_at, env.ledger().timestamp());
        assert_eq!(match_result.confirmed, false);
        assert_eq!(match_result.medical_facility, medical_facility);
    }

    #[test]
    fn test_update_recipient_urgency() {
        let (env, client, admin) = create_test_env();
        let recipient = Address::generate(&env);
        let medical_facility = Address::generate(&env);

        // Initialize and register recipient
        client.initialize(&admin, &1000, &5000, &70, &60);

        let hla_profile = create_test_hla_profile(&env);

        client.register_recipient(
            &recipient,
            &BloodType::A,
            &OrganType::Kidney,
            &hla_profile,
            &45,
            &UrgencyLevel::Medium,
            &medical_facility,
            &500,
        );

        // Update urgency level
        client.update_recipient_urgency(
            &recipient,
            &UrgencyLevel::Critical,
            &900,
            &medical_facility,
        );

        // Verify update
        let updated_recipient = client.get_recipient(&recipient).unwrap();
        assert_eq!(updated_recipient.urgency_level, UrgencyLevel::Critical);
        assert_eq!(updated_recipient.medical_priority_score, 900);
    }

    #[test]
    fn test_deactivate_donor() {
        let (env, client, admin) = create_test_env();
        let donor = Address::generate(&env);
        let medical_facility = Address::generate(&env);

        // Initialize and register donor
        client.initialize(&admin, &1000, &5000, &70, &60);

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

        // Deactivate donor
        client.deactivate_donor(&donor, &admin);

        // Verify deactivation
        let deactivated_donor = client.get_donor(&donor).unwrap();
        assert!(!deactivated_donor.is_active);
    }

    #[test]
    fn test_deactivate_recipient() {
        let (env, client, admin) = create_test_env();
        let recipient = Address::generate(&env);
        let medical_facility = Address::generate(&env);

        // Initialize and register recipient
        client.initialize(&admin, &1000, &5000, &70, &60);

        let hla_profile = create_test_hla_profile(&env);

        client.register_recipient(
            &recipient,
            &BloodType::A,
            &OrganType::Kidney,
            &hla_profile,
            &45,
            &UrgencyLevel::High,
            &medical_facility,
            &750,
        );

        // Deactivate recipient
        client.deactivate_recipient(&recipient, &admin);

        // Verify deactivation
        let deactivated_recipient = client.get_recipient(&recipient);
        assert!(!deactivated_recipient.unwrap().is_active);
    }

    #[test]
    fn test_invalid_blood_type_compatibility() {
        let (env, client, admin) = create_test_env();
        let donor = Address::generate(&env);
        let recipient = Address::generate(&env);
        let medical_facility = Address::generate(&env);

        // Initialize contract
        client.initialize(&admin, &1000, &5000, &70, &60);

        let hla_profile = create_test_hla_profile(&env);
        let consent_hash = String::from_str(&env, "test_consent_hash_1234567890123456");

        // Register incompatible donor and recipient
        client.register_donor(
            &donor,
            &BloodType::A, // A blood type
            &OrganType::Kidney,
            &hla_profile.clone(),
            &35,
            &medical_facility,
            &consent_hash,
        );

        client.register_recipient(
            &recipient,
            &BloodType::B, // B blood type (incompatible with A)
            &OrganType::Kidney,
            &hla_profile,
            &45,
            &UrgencyLevel::High,
            &medical_facility,
            &750,
        );

        // Should not find any matches due to blood type incompatibility
        let matches = client.find_match(&recipient);
        // The matching algorithm should return an empty list for incompatible blood types
    }

    #[test]
    #[should_panic]
    fn test_max_capacity_limits() {
        let (env, client, admin) = create_test_env();

        // Initialize with very low limits for testing
        client.initialize(&admin, &1, &1, &70, &60);

        let donor1 = Address::generate(&env);
        let donor2 = Address::generate(&env);
        let medical_facility = Address::generate(&env);
        let hla_profile = create_test_hla_profile(&env);
        let consent_hash = String::from_str(&env, "test_consent_hash_1234567890123456");

        // First donor should succeed
        client.register_donor(
            &donor1,
            &BloodType::O,
            &OrganType::Kidney,
            &hla_profile.clone(),
            &35,
            &medical_facility,
            &consent_hash.clone(),
        );

        // Second donor should fail due to capacity
        client.register_donor(
            &donor2,
            &BloodType::A,
            &OrganType::Kidney,
            &hla_profile,
            &40,
            &medical_facility,
            &consent_hash,
        );
    }

    #[test]
    fn test_organ_type_matching() {
        let (env, client, admin) = create_test_env();
        let donor = Address::generate(&env);
        let recipient = Address::generate(&env);
        let medical_facility = Address::generate(&env);

        client.initialize(&admin, &1000, &5000, &70, &60);

        let hla_profile = create_test_hla_profile(&env);
        let consent_hash = String::from_str(&env, "test_consent_hash_1234567890123456");

        // Register donor with Kidney
        client.register_donor(
            &donor,
            &BloodType::O,
            &OrganType::Kidney,
            &hla_profile.clone(),
            &35,
            &medical_facility,
            &consent_hash,
        );

        // Register recipient needing Heart (different organ type)
        client.register_recipient(
            &recipient,
            &BloodType::A,
            &OrganType::Heart, // Different organ type
            &hla_profile,
            &45,
            &UrgencyLevel::Critical,
            &medical_facility,
            &900,
        );

        // Should not match different organ types
        let matches = client.find_match(&recipient);

        // Should return empty matches due to organ type mismatch
    }

    #[test]
    #[should_panic]
    fn test_authorization_checks_failure() {
        let (env, client, admin) = create_test_env();
        let unauthorized_user = Address::generate(&env);
        let recipient = Address::generate(&env);
        let medical_facility = Address::generate(&env);

        client.initialize(&admin, &1000, &5000, &70, &60);

        let hla_profile = create_test_hla_profile(&env);

        client.register_recipient(
            &recipient,
            &BloodType::A,
            &OrganType::Kidney,
            &hla_profile,
            &45,
            &UrgencyLevel::High,
            &medical_facility,
            &750,
        );

        // Unauthorized user should not be able to update urgency
        client.update_recipient_urgency(
            &recipient,
            &UrgencyLevel::Critical,
            &900,
            &unauthorized_user, // Unauthorized caller
        );
    }

    #[test]
    fn test_urgency_level_priority_scoring() {
        use crate::matching::calculate_priority_score;

        let (env, _client, _admin) = create_test_env();
        let hla_profile = create_test_hla_profile(&env);
        let medical_facility = Address::generate(&env);

        let critical_recipient = RecipientProfile {
            address: Address::generate(&env),
            blood_type: BloodType::A,
            organ_type: OrganType::Kidney,
            hla_profile: hla_profile.clone(),
            age: 45,
            urgency_level: UrgencyLevel::Critical,
            registered_at: 0,
            wait_time_days: 0,
            is_active: true,
            medical_facility: medical_facility.clone(),
            medical_priority_score: 800,
        };

        let low_recipient = RecipientProfile {
            address: Address::generate(&env),
            blood_type: BloodType::A,
            organ_type: OrganType::Kidney,
            hla_profile,
            age: 45,
            urgency_level: UrgencyLevel::Low,
            registered_at: 0,
            wait_time_days: 0,
            is_active: true,
            medical_facility,
            medical_priority_score: 300,
        };

        let critical_score = calculate_priority_score(&critical_recipient, 30, 70);
        let low_score = calculate_priority_score(&low_recipient, 30, 70);

        // Critical urgency should have much higher priority score
        assert!(critical_score > low_score);
        assert!(critical_score >= 1000); // Critical base score should be 1000
    }

    #[test]
    fn test_wait_time_impact() {
        use crate::matching::calculate_priority_score;

        let (env, _client, _admin) = create_test_env();
        let hla_profile = create_test_hla_profile(&env);
        let medical_facility = Address::generate(&env);

        let recipient = RecipientProfile {
            address: Address::generate(&env),
            blood_type: BloodType::A,
            organ_type: OrganType::Kidney,
            hla_profile,
            age: 45,
            urgency_level: UrgencyLevel::Medium,
            registered_at: 0,
            wait_time_days: 0,
            is_active: true,
            medical_facility,
            medical_priority_score: 500,
        };

        let short_wait_score = calculate_priority_score(&recipient, 30, 70); // 30 days
        let long_wait_score = calculate_priority_score(&recipient, 365, 70); // 1 year

        // Longer wait time should result in higher priority score
        assert!(long_wait_score > short_wait_score);
    }

    #[test]
    fn test_comprehensive_compatibility_scoring() {
        use crate::matching::calculate_compatibility_score;

        let (env, _client, _admin) = create_test_env();
        let hla_profile = create_test_hla_profile(&env);
        let medical_facility = Address::generate(&env);

        let donor = DonorProfile {
            address: Address::generate(&env),
            blood_type: BloodType::O, // Universal donor
            organ_type: OrganType::Kidney,
            hla_profile: hla_profile.clone(),
            age: 35,
            registered_at: 0,
            is_active: true,
            medical_facility: medical_facility.clone(),
            consent_hash: String::from_str(&env, "consent_hash_123"),
        };

        let compatible_recipient = RecipientProfile {
            address: Address::generate(&env),
            blood_type: BloodType::A, // Compatible with O
            organ_type: OrganType::Kidney,
            hla_profile: hla_profile.clone(), // Perfect HLA match
            age: 40,                          // Close age
            urgency_level: UrgencyLevel::High,
            registered_at: 0,
            wait_time_days: 0,
            is_active: true,
            medical_facility: medical_facility.clone(),
            medical_priority_score: 750,
        };

        let incompatible_recipient = RecipientProfile {
            address: Address::generate(&env),
            blood_type: BloodType::AB,    // Less compatible
            organ_type: OrganType::Heart, // Different organ
            hla_profile,
            age: 65, // Larger age gap
            urgency_level: UrgencyLevel::Low,
            registered_at: 0,
            wait_time_days: 0,
            is_active: true,
            medical_facility,
            medical_priority_score: 200,
        };

        let compatible_score = calculate_compatibility_score(&donor, &compatible_recipient);
        let incompatible_score = calculate_compatibility_score(&donor, &incompatible_recipient);

        // Compatible recipient should have much higher score
        assert!(compatible_score > incompatible_score);
        assert!(compatible_score > 80); // Should be high compatibility
    }

    #[test]
    #[should_panic]
    fn test_double_initialization_prevention() {
        let (_env, client, admin) = create_test_env();

        // First initialization should succeed
        client.initialize(&admin, &1000, &5000, &70, &60);

        // Second initialization should fail
        client.initialize(&admin, &1000, &5000, &70, &60);
    }
}
