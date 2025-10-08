
use crate::tests::utils::*;
use crate::{AccessLevel, SensitivityLevel, MedicalRecordsError};
use soroban_sdk::{testutils::Address as _, Address, String};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_access_with_valid_permission() {
        let env = TestEnvironment::new();
        
        let record_id = env.create_test_record(RECORD_TYPE_LAB_RESULT, SensitivityLevel::Medium);
        let future_time = env.env.ledger().timestamp() + ONE_WEEK;
        
        // Grant access
        env.contract.grant_access(
            &env.patient,
            &record_id,
            &env.provider1,
            &AccessLevel::Read,
            &Some(future_time),
            &String::from_str(&env.env, "Medical consultation"),
        );
        
        // Provider should be able to access the record
        let record = env.contract.access_record(
            &env.provider1,
            &record_id,
            &String::from_str(&env.env, "Reviewing for treatment"),
        );
        
        assert!(record.is_some());
        assert_eq!(record.unwrap().patient, env.patient);
    }

    #[test]
    fn test_unauthorized_provider_access_denied() {
        let env = TestEnvironment::new();
        
        let record_id = env.create_test_record(RECORD_TYPE_LAB_RESULT, SensitivityLevel::Medium);
        
        // Provider2 attempts to access without permission
        let record = env.contract.access_record(
            &env.provider2,
            &record_id,
            &String::from_str(&env.env, "Unauthorized attempt"),
        );
        
        assert!(record.is_none());
    }

    #[test]
    fn test_expired_access_denied() {
        let env = TestEnvironment::new();
        
        let record_id = env.create_test_record(RECORD_TYPE_LAB_RESULT, SensitivityLevel::Medium);
        let short_expiry = env.env.ledger().timestamp() + ONE_HOUR;
        
        // Grant access with short expiry
        env.contract.grant_access(
            &env.patient,
            &record_id,
            &env.provider1,
            &AccessLevel::Read,
            &Some(short_expiry),
            &String::from_str(&env.env, "Short-term access"),
        );
        
        // Advance time beyond expiry
        env.advance_time(ONE_HOUR + 1);
        
        // Access should be denied
        let record = env.contract.access_record(
            &env.provider1,
            &record_id,
            &String::from_str(&env.env, "Expired access attempt"),
        );
        
        assert!(record.is_none());
    }

    #[test]
    fn test_revoked_access_denied() {
        let env = TestEnvironment::new();
        
        let record_id = env.create_test_record(RECORD_TYPE_LAB_RESULT, SensitivityLevel::Medium);
        let future_time = env.env.ledger().timestamp() + ONE_WEEK;
        
        // Grant access
        env.contract.grant_access(
            &env.patient,
            &record_id,
            &env.provider1,
            &AccessLevel::Read,
            &Some(future_time),
            &String::from_str(&env.env, "Initial access"),
        );
        
        // Revoke access
        env.contract.revoke_access(&env.patient, &record_id, &env.provider1);
        
        // Access should be denied
        let record = env.contract.access_record(
            &env.provider1,
            &record_id,
            &String::from_str(&env.env, "Revoked access attempt"),
        );
        
        assert!(record.is_none());
    }

    #[test]
    fn test_time_limited_access_validation() {
        let env = TestEnvironment::new();
        
        let record_id = env.create_test_record(RECORD_TYPE_LAB_RESULT, SensitivityLevel::Medium);
        let future_time = env.env.ledger().timestamp() + ONE_DAY;
        
        // Grant time-limited access
        env.contract.grant_access(
            &env.patient,
            &record_id,
            &env.provider1,
            &AccessLevel::Read,
            &Some(future_time),
            &String::from_str(&env.env, "24-hour access"),
        );
        
        // Access should work within the time limit
        let record_before = env.contract.access_record(
            &env.provider1,
            &record_id,
            &String::from_str(&env.env, "Within time limit"),
        );
        assert!(record_before.is_some());
        
        // Advance time beyond limit
        env.advance_time(ONE_DAY + 1);
        
        // Access should be denied after expiry
        let record_after = env.contract.access_record(
            &env.provider1,
            &record_id,
            &String::from_str(&env.env, "After time limit"),
        );
        assert!(record_after.is_none());
    }

    #[test]
    fn test_scoped_access_restrictions() {
        let env = TestEnvironment::new();
        
        let record_id = env.create_test_record(RECORD_TYPE_LAB_RESULT, SensitivityLevel::Medium);
        let future_time = env.env.ledger().timestamp() + ONE_WEEK;
        
        // Grant read-only access
        env.contract.grant_access(
            &env.patient,
            &record_id,
            &env.provider1,
            &AccessLevel::Read,
            &Some(future_time),
            &String::from_str(&env.env, "Read-only consultation"),
        );
        
        // Provider can access for reading
        let record = env.contract.access_record(
            &env.provider1,
            &record_id,
            &String::from_str(&env.env, "Reading record"),
        );
        assert!(record.is_some());
        
        // Provider cannot perform write operations (this would be enforced at application level)
        // Here we verify the access level is correctly stored
        let permissions = env.contract.get_access_permissions(&env.patient, &record_id);
        assert!(permissions.is_some());
        let grant = permissions.unwrap().get(env.provider1.clone()).unwrap();
        assert_eq!(grant.access_level, AccessLevel::Read);
    }

    #[test]
    fn test_emergency_access_authorized_provider() {
        let env = TestEnvironment::new();
        
        let record_id = env.create_test_record(RECORD_TYPE_EMERGENCY, SensitivityLevel::High);
        
        // Provider1 is configured for emergency access
        let record = env.contract.emergency_access(
            &env.provider1,
            &record_id,
            &String::from_str(&env.env, "Patient unconscious, need medical history"),
            &env.emergency_contact,
        );
        
        assert!(record.is_some());
        assert_eq!(record.unwrap().record_type, String::from_str(&env.env, RECORD_TYPE_EMERGENCY));
    }

    #[test]
    fn test_emergency_access_unauthorized_provider() {
        let env = TestEnvironment::new();
        
        let record_id = env.create_test_record(RECORD_TYPE_EMERGENCY, SensitivityLevel::High);
        
        // Provider2 is NOT configured for emergency access
        let record = env.contract.emergency_access(
            &env.provider2,
            &record_id,
            &String::from_str(&env.env, "Emergency attempt"),
            &env.emergency_contact,
        );
        
        assert!(record.is_none());
    }

    #[test]
    fn test_multiple_providers_access_same_record() {
        let env = TestEnvironment::new();
        
        let record_id = env.create_test_record(RECORD_TYPE_DIAGNOSIS, SensitivityLevel::Medium);
        let future_time = env.env.ledger().timestamp() + ONE_WEEK;
        
        // Grant access to both providers
        env.contract.grant_access(
            &env.patient,
            &record_id,
            &env.provider1,
            &AccessLevel::Read,
            &Some(future_time),
            &String::from_str(&env.env, "Cardiology consultation"),
        );
        
        env.contract.grant_access(
            &env.patient,
            &record_id,
            &env.provider2,
            &AccessLevel::Read,
            &Some(future_time),
            &String::from_str(&env.env, "Neurology consultation"),
        );
        
        // Both providers should be able to access
        let record1 = env.contract.access_record(
            &env.provider1,
            &record_id,
            &String::from_str(&env.env, "Cardiology review"),
        );
        let record2 = env.contract.access_record(
            &env.provider2,
            &record_id,
            &String::from_str(&env.env, "Neurology review"),
        );
        
        assert!(record1.is_some());
        assert!(record2.is_some());
        
        // Verify access permissions show both providers
        let permissions = env.contract.get_access_permissions(&env.patient, &record_id);
        assert!(permissions.is_some());
        let perms = permissions.unwrap();
        assert!(perms.contains_key(env.provider1.clone()));
        assert!(perms.contains_key(env.provider2.clone()));
    }

    #[test]
    fn test_access_count_tracking() {
        let env = TestEnvironment::new();
        
        let record_id = env.create_test_record(RECORD_TYPE_LAB_RESULT, SensitivityLevel::Medium);
        let future_time = env.env.ledger().timestamp() + ONE_WEEK;
        
        // Grant access
        env.contract.grant_access(
            &env.patient,
            &record_id,
            &env.provider1,
            &AccessLevel::Read,
            &Some(future_time),
            &String::from_str(&env.env, "Monitoring access"),
        );
        
        // Access the record multiple times
        env.contract.access_record(
            &env.provider1,
            &record_id,
            &String::from_str(&env.env, "First access"),
        );
        env.contract.access_record(
            &env.provider1,
            &record_id,
            &String::from_str(&env.env, "Second access"),
        );
        env.contract.access_record(
            &env.provider1,
            &record_id,
            &String::from_str(&env.env, "Third access"),
        );
        
        // Verify access count is tracked
        let permissions = env.contract.get_access_permissions(&env.patient, &record_id);
        let grant = permissions.unwrap().get(env.provider1.clone()).unwrap();
        assert_eq!(grant.access_count, 3);
        assert!(grant.last_accessed.is_some());
    }

    #[test]
    fn test_different_access_levels() {
        let env = TestEnvironment::new();
        
        let record_id = env.create_test_record(RECORD_TYPE_PRESCRIPTION, SensitivityLevel::Medium);
        let future_time = env.env.ledger().timestamp() + ONE_WEEK;
        
        // Grant different access levels to different providers
        env.contract.grant_access(
            &env.patient,
            &record_id,
            &env.provider1,
            &AccessLevel::Read,
            &Some(future_time),
            &String::from_str(&env.env, "Read-only access"),
        );
        
        env.contract.grant_access(
            &env.patient,
            &record_id,
            &env.provider2,
            &AccessLevel::ReadWrite,
            &Some(future_time),
            &String::from_str(&env.env, "Full access"),
        );
        
        // Verify different access levels were granted
        let permissions = env.contract.get_access_permissions(&env.patient, &record_id);
        let perms = permissions.unwrap();
        
        let grant1 = perms.get(env.provider1.clone()).unwrap();
        let grant2 = perms.get(env.provider2.clone()).unwrap();
        
        assert_eq!(grant1.access_level, AccessLevel::Read);
        assert_eq!(grant2.access_level, AccessLevel::ReadWrite);
    }

    #[test]
    fn test_patient_consent_integration() {
        let env = TestEnvironment::new();
        
        let record_id = env.create_test_record(RECORD_TYPE_LAB_RESULT, SensitivityLevel::Medium);
        
        // Test consent verification
        let consent_result = env.contract.verify_patient_consent(
            &env.patient,
            &env.provider1,
            &String::from_str(&env.env, RECORD_TYPE_LAB_RESULT),
        );
        
        // Should return true since no consent contract is configured (default behavior)
        assert!(consent_result);
    }

    #[test]
    fn test_access_without_authentication() {
        let env = TestEnvironment::new();
        
        let record_id = env.create_test_record(RECORD_TYPE_LAB_RESULT, SensitivityLevel::Medium);
        let unauthorized_user = Address::generate(&env.env);
        
        // Unauthorized user attempts to access record
        let record = env.contract.access_record(
            &unauthorized_user,
            &record_id,
            &String::from_str(&env.env, "Unauthorized access"),
        );
        
        assert!(record.is_none());
    }

    #[test]
    fn test_access_permissions_visibility() {
        let env = TestEnvironment::new();
        
        let record_id = env.create_test_record(RECORD_TYPE_LAB_RESULT, SensitivityLevel::Medium);
        let future_time = env.env.ledger().timestamp() + ONE_WEEK;
        
        // Grant access
        env.contract.grant_access(
            &env.patient,
            &record_id,
            &env.provider1,
            &AccessLevel::Read,
            &Some(future_time),
            &String::from_str(&env.env, "Medical consultation"),
        );
        
        // Patient can see access permissions
        let patient_view = env.contract.get_access_permissions(&env.patient, &record_id);
        assert!(patient_view.is_some());
        
        // Provider cannot see access permissions (only patients can)
        let provider_view = env.contract.get_access_permissions(&env.provider1, &record_id);
        assert!(provider_view.is_none());
        
        // Other patient cannot see permissions
        let other_patient = Address::generate(&env.env);
        let other_view = env.contract.get_access_permissions(&other_patient, &record_id);
        assert!(other_view.is_none());
    }

    #[test]
    fn test_bulk_access_permissions() {
        let env = TestEnvironment::new();
        
        let record_ids = env.create_multiple_records(5, RECORD_TYPE_LAB_RESULT);
        let future_time = env.env.ledger().timestamp() + ONE_WEEK;
        
        // Bulk grant access
        let successful = env.contract.bulk_grant_access(
            &env.patient,
            &record_ids,
            &env.provider1,
            &AccessLevel::Read,
            &Some(future_time),
            &String::from_str(&env.env, "Bulk medical review"),
        );
        
        assert_eq!(successful, 5);
        
        // Verify provider can access all records
        for i in 0..record_ids.len() {
            let record_id = record_ids.get(i).unwrap();
            let record = env.contract.access_record(
                &env.provider1,
                &record_id,
                &String::from_str(&env.env, "Bulk access test"),
            );
            assert!(record.is_some());
        }
    }

    #[test]
    fn test_high_volume_access_requests() {
        let env = TestEnvironment::new();
        
        let record_id = env.create_test_record(RECORD_TYPE_LAB_RESULT, SensitivityLevel::Medium);
        let future_time = env.env.ledger().timestamp() + ONE_WEEK;
        
        // Grant access
        env.contract.grant_access(
            &env.patient,
            &record_id,
            &env.provider1,
            &AccessLevel::Read,
            &Some(future_time),
            &String::from_str(&env.env, "High volume test"),
        );
        
        // Perform multiple access requests to test scalability
        for i in 0..20 {
            let access_reason = String::from_str(&env.env, "High volume access");
            let record = env.contract.access_record(
                &env.provider1,
                &record_id,
                &access_reason,
            );
            assert!(record.is_some());
        }
        
        // Verify access count is correctly tracked
        let permissions = env.contract.get_access_permissions(&env.patient, &record_id);
        let grant = permissions.unwrap().get(env.provider1.clone()).unwrap();
        assert_eq!(grant.access_count, 20);
    }
}

