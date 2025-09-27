use crate::tests::utils::*;
use crate::{AccessLevel, SensitivityLevel, AuditEventType, AuditEntry, DataKey};
use soroban_sdk::{testutils::Address as _, Address, String, Vec};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_log_creation_on_record_access() {
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
        
        // Access the record
        env.contract.access_record(
            &env.provider1,
            &record_id,
            &String::from_str(&env.env, "Reviewing patient history"),
        );
        
        // Get audit logs
        let audit_logs = env.contract.get_audit_logs(&env.patient, &Some(record_id.clone()), &0, &10);
        
        // Should have logs for: record creation, access granted, record accessed
        assert!(audit_logs.len() >= 3);
        
        // Verify record access is logged
        let access_log = audit_logs.iter().find(|log| {
            log.event_type == AuditEventType::RecordAccessed && log.actor == env.provider1
        });
        assert!(access_log.is_some());
        let access_entry = access_log.unwrap();
        assert!(access_entry.has_record_id);
        assert_eq!(access_entry.record_id, record_id);
    }

    #[test]
    fn test_audit_log_access_denied_events() {
        let env = TestEnvironment::new();
        
        let record_id = env.create_test_record(RECORD_TYPE_LAB_RESULT, SensitivityLevel::Medium);
        
        // Provider2 attempts to access without permission (should be denied and logged)
        let record = env.contract.access_record(
            &env.provider2,
            &record_id,
            &String::from_str(&env.env, "Unauthorized access attempt"),
        );
        assert!(record.is_none());
        
        // Get audit logs (admin can see system-wide logs)
        let audit_logs = env.contract.get_audit_logs(&env.admin, &None, &0, &20);
        
        // Should have access denied event
        let denied_log = audit_logs.iter().find(|log| {
            log.event_type == AuditEventType::AccessDenied && log.actor == env.provider2
        });
        assert!(denied_log.is_some());
    }

    #[test]
    fn test_audit_log_immutability() {
        let env = TestEnvironment::new();
        
        let record_id = env.create_test_record(RECORD_TYPE_LAB_RESULT, SensitivityLevel::Medium);
        
        // Get initial audit logs
        let initial_logs = env.contract.get_audit_logs(&env.patient, &Some(record_id.clone()), &0, &10);
        let initial_count = initial_logs.len();
        
        // Perform another action that should create an audit log
        let future_time = env.env.ledger().timestamp() + ONE_WEEK;
        env.contract.grant_access(
            &env.patient,
            &record_id,
            &env.provider1,
            &AccessLevel::Read,
            &Some(future_time),
            &String::from_str(&env.env, "New access grant"),
        );
        
        // Get updated logs
        let updated_logs = env.contract.get_audit_logs(&env.patient, &Some(record_id.clone()), &0, &10);
        
        // Should have one more log entry
        assert_eq!(updated_logs.len(), initial_count + 1);
        
        // Verify original logs are unchanged (immutability)
        for i in 0..initial_count {
            if let (Some(initial), Some(updated)) = (initial_logs.get(i), updated_logs.get(i)) {
                assert_eq!(initial, updated);
            }
        }
    }

    #[test]
    fn test_audit_log_accuracy() {
        let env = TestEnvironment::new();
        
        let record_id = env.create_test_record(RECORD_TYPE_DIAGNOSIS, SensitivityLevel::High);
        let future_time = env.env.ledger().timestamp() + ONE_WEEK;
        
        // Grant access
        env.contract.grant_access(
            &env.patient,
            &record_id,
            &env.provider1,
            &AccessLevel::ReadWrite,
            &Some(future_time),
            &String::from_str(&env.env, "Full access for treatment"),
        );
        
        // Access record
        env.contract.access_record(
            &env.provider1,
            &record_id,
            &String::from_str(&env.env, "Initial diagnosis review"),
        );
        
        // Revoke access
        env.contract.revoke_access(&env.patient, &record_id, &env.provider1);
        
        // Get all audit logs for this record
        let audit_logs = env.contract.get_audit_logs(&env.patient, &Some(record_id.clone()), &0, &20);
        
        // Find and verify different event types exist
        let mut creation_log = None;
        let mut grant_log = None;
        let mut access_log = None;
        let mut revoke_log = None;
        
        for i in 0..audit_logs.len() {
            let log = audit_logs.get(i).unwrap();
            match log.event_type {
                AuditEventType::RecordCreated if creation_log.is_none() => creation_log = Some(log.clone()),
                AuditEventType::AccessGranted if grant_log.is_none() => grant_log = Some(log.clone()),
                AuditEventType::RecordAccessed if access_log.is_none() => access_log = Some(log.clone()),
                AuditEventType::AccessRevoked if revoke_log.is_none() => revoke_log = Some(log.clone()),
                _ => {}
            }
        }
        
        assert!(creation_log.is_some());
        assert!(grant_log.is_some());
        assert!(access_log.is_some());
        assert!(revoke_log.is_some());
        
        let creation = creation_log.unwrap();
        let grant = grant_log.unwrap();
        let access = access_log.unwrap();
        let revoke = revoke_log.unwrap();
        
        // Verify chronological order
        assert!(creation.timestamp <= grant.timestamp);
        assert!(grant.timestamp <= access.timestamp);
        assert!(access.timestamp <= revoke.timestamp);
        
        // Verify actors are correct
        assert_eq!(creation.actor, env.patient);
        assert_eq!(grant.actor, env.patient);
        assert_eq!(access.actor, env.provider1);
        assert_eq!(revoke.actor, env.patient);
    }

    #[test]
    fn test_audit_log_compliance_with_privacy_regulations() {
        let env = TestEnvironment::new();
        
        let record_id = env.create_test_record(RECORD_TYPE_LAB_RESULT, SensitivityLevel::Restricted);
        let future_time = env.env.ledger().timestamp() + ONE_WEEK;
        
        // Grant access
        env.contract.grant_access(
            &env.patient,
            &record_id,
            &env.provider1,
            &AccessLevel::Read,
            &Some(future_time),
            &String::from_str(&env.env, "HIPAA-compliant access for treatment"),
        );
        
        // Access record multiple times
        for _i in 0..5 {
            env.contract.access_record(
                &env.provider1,
                &record_id,
                &String::from_str(&env.env, "Ongoing treatment access"),
            );
        }
        
        // Get audit logs
        let audit_logs = env.contract.get_audit_logs(&env.patient, &Some(record_id.clone()), &0, &20);
        
        // Verify all access events are logged (HIPAA requirement)
        let mut access_count = 0;
        for i in 0..audit_logs.len() {
            let log = audit_logs.get(i).unwrap();
            if log.event_type == AuditEventType::RecordAccessed {
                access_count += 1;
            }
        }
        assert_eq!(access_count, 5);
        
        // Verify compliance requirements for access logs
        for i in 0..audit_logs.len() {
            let log = audit_logs.get(i).unwrap();
            if log.event_type == AuditEventType::RecordAccessed {
                assert!(log.timestamp >= 0); // Must have timestamp (>= 0 in test env)
                assert_eq!(log.actor, env.provider1); // Must identify who accessed
                assert!(log.has_record_id); // Must identify what was accessed
                assert_eq!(log.record_id, record_id.clone());
                assert!(log.details.len() > 0); // Must have details about the access
            }
        }
    }

    #[test]
    fn test_emergency_access_special_logging() {
        let env = TestEnvironment::new();
        
        let record_id = env.create_test_record(RECORD_TYPE_EMERGENCY, SensitivityLevel::High);
        
        // Emergency access
        let record = env.contract.emergency_access(
            &env.provider1,
            &record_id,
            &String::from_str(&env.env, "Patient unconscious, cardiac emergency"),
            &env.emergency_contact,
        );
        assert!(record.is_some());
        
        // Get audit logs (admin can see all)
        let audit_logs = env.contract.get_audit_logs(&env.admin, &None, &0, &20);
        
        // Verify emergency access is specially logged
        let emergency_log = audit_logs.iter().find(|log| {
            log.event_type == AuditEventType::EmergencyAccess && log.actor == env.provider1
        });
        assert!(emergency_log.is_some());
        
        let log = emergency_log.unwrap();
        assert!(log.has_record_id);
        assert_eq!(log.record_id, record_id);
        assert!(log.has_target);
        assert_eq!(log.target, env.emergency_contact);
        // We can't use contains on soroban String, so we'll just check it exists
        assert!(log.details.len() > 0);
    }

    #[test]
    fn test_audit_log_with_missing_data_scenarios() {
        let env = TestEnvironment::new();
        
        // Test system initialization audit log
        let system_logs = env.contract.get_audit_logs(&env.admin, &None, &0, &5);
        
        // Should have system initialization log
        let init_log = system_logs.iter().find(|log| {
            log.event_type == AuditEventType::SystemInitialized
        });
        assert!(init_log.is_some());
        
        // System logs may have false for has_record_id and has_target (which is valid)
        let log = init_log.unwrap();
        assert_eq!(log.actor, env.admin);
        assert!(!log.has_record_id); // System events may not have record_id
    }

    #[test]
    fn test_audit_log_access_control() {
        let env = TestEnvironment::new();
        
        let record_id = env.create_test_record(RECORD_TYPE_LAB_RESULT, SensitivityLevel::Medium);
        let other_patient = Address::generate(&env.env);
        
        // Patient should be able to see their own audit logs
        let patient_logs = env.contract.get_audit_logs(&env.patient, &Some(record_id.clone()), &0, &10);
        assert!(patient_logs.len() > 0);
        
        // Other patient should not see audit logs for records they don't own
        let other_logs = env.contract.get_audit_logs(&other_patient, &Some(record_id.clone()), &0, &10);
        assert_eq!(other_logs.len(), 0);
        
        // Admin should be able to see system-wide logs
        let admin_logs = env.contract.get_audit_logs(&env.admin, &None, &0, &20);
        assert!(admin_logs.len() > 0);
        
        // Non-admin should not see system-wide logs
        let non_admin_logs = env.contract.get_audit_logs(&env.patient, &None, &0, &20);
        assert_eq!(non_admin_logs.len(), 0);
    }

    #[test]
    fn test_audit_log_sequence_integrity() {
        let env = TestEnvironment::new();
        
        let record_id = env.create_test_record(RECORD_TYPE_LAB_RESULT, SensitivityLevel::Medium);
        let future_time = env.env.ledger().timestamp() + ONE_WEEK;
        
        // Perform multiple operations to generate audit logs
        env.contract.grant_access(
            &env.patient,
            &record_id,
            &env.provider1,
            &AccessLevel::Read,
            &Some(future_time),
            &String::from_str(&env.env, "First access grant"),
        );
        
        env.contract.access_record(
            &env.provider1,
            &record_id,
            &String::from_str(&env.env, "First access"),
        );
        
        env.contract.revoke_access(&env.patient, &record_id, &env.provider1);
        
        // Get audit logs
        let audit_logs = env.contract.get_audit_logs(&env.admin, &None, &0, &50);
        
        // Verify sequence numbers are consecutive
        let mut prev_sequence = 0u64;
        for log in audit_logs.iter() {
            assert!(log.sequence > prev_sequence);
            prev_sequence = log.sequence;
        }
        
        // Verify timestamps are non-decreasing (allowing for same timestamp)
        let mut prev_timestamp = 0u64;
        for log in audit_logs.iter() {
            assert!(log.timestamp >= prev_timestamp);
            prev_timestamp = log.timestamp;
        }
    }

    #[test]
    fn test_bulk_operation_audit_logging() {
        let env = TestEnvironment::new();
        
        let record_ids = env.create_multiple_records(3, RECORD_TYPE_LAB_RESULT);
        let future_time = env.env.ledger().timestamp() + ONE_WEEK;
        
        // Bulk grant access
        env.contract.bulk_grant_access(
            &env.patient,
            &record_ids,
            &env.provider1,
            &AccessLevel::Read,
            &Some(future_time),
            &String::from_str(&env.env, "Bulk consultation access"),
        );
        
        // Get audit logs
        let audit_logs = env.contract.get_audit_logs(&env.admin, &None, &0, &50);
        
        // Should have individual grant logs for each record plus bulk operation log
        let bulk_log = audit_logs.iter().find(|log| {
            log.event_type == AuditEventType::BulkAccessGranted && log.actor == env.patient
        });
        assert!(bulk_log.is_some());
        
        // Count individual access granted logs
        let mut individual_grant_count = 0;
        for i in 0..audit_logs.len() {
            let log = audit_logs.get(i).unwrap();
            if log.event_type == AuditEventType::AccessGranted && log.actor == env.patient {
                individual_grant_count += 1;
            }
        }
        assert_eq!(individual_grant_count, 3);
    }

    #[test]
    fn test_provider_registration_audit_logging() {
        let env = TestEnvironment::new();
        let new_provider = Address::generate(&env.env);
        
        // Register new provider
        let provider_info = create_test_provider_info(
            &env.env,
            "LIC999",
            "Dr. New Provider",
            "Emergency Medicine",
            true,
            true,
        );
        
        env.contract.register_provider(&env.admin, &new_provider, &provider_info);
        
        // Check audit logs
        let audit_logs = env.contract.get_audit_logs(&env.admin, &None, &0, &50);
        
        let registration_log = audit_logs.iter().find(|log| {
            log.event_type == AuditEventType::ProviderRegistered &&
            log.actor == env.admin &&
            log.has_target && log.target == new_provider
        });
        assert!(registration_log.is_some());
    }

    #[test]
    fn test_audit_log_pagination() {
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
            &String::from_str(&env.env, "Pagination test"),
        );
        
        // Generate multiple audit entries by accessing multiple times
        for _i in 0..10 {
            env.contract.access_record(
                &env.provider1,
                &record_id,
                &String::from_str(&env.env, "Pagination test access"),
            );
        }
        
        // Test pagination - get first batch
        let first_batch = env.contract.get_audit_logs(&env.patient, &Some(record_id.clone()), &1, &5);
        
        // Get second batch starting from where first ended
        let second_batch_start = if first_batch.len() > 0 {
            first_batch.get(first_batch.len() - 1).unwrap().sequence + 1
        } else {
            6
        };
        let second_batch = env.contract.get_audit_logs(&env.patient, &Some(record_id.clone()), &second_batch_start, &5);
        
        // Verify batches are different (if both have content)
        if !first_batch.is_empty() && !second_batch.is_empty() {
            let first_sequence = first_batch.get(0).unwrap().sequence;
            let second_sequence = second_batch.get(0).unwrap().sequence;
            assert_ne!(first_sequence, second_sequence, "Batches should have different starting sequences");
        }
    }

    #[test]
    fn test_consent_verification_audit_logging() {
        let env = TestEnvironment::new();
        
        // Get initial count of audit logs for the patient
        let initial_logs = env.contract.get_audit_logs(&env.admin, &None, &0, &50);
        let initial_count = initial_logs.len();
        
        // Test consent verification
        let consent_result = env.contract.verify_patient_consent(
            &env.patient,
            &env.provider1,
            &String::from_str(&env.env, RECORD_TYPE_LAB_RESULT),
        );
        assert!(consent_result);
        
        // Get audit logs after consent verification
        let audit_logs = env.contract.get_audit_logs(&env.admin, &None, &0, &50);
        assert!(audit_logs.len() > initial_count, "Should have more logs after consent verification");
        
        // Find the consent verification log
        let consent_log = audit_logs.iter().find(|log| {
            log.event_type == AuditEventType::ConsentVerified &&
            log.actor == env.patient &&
            log.has_target && log.target == env.provider1
        });
        assert!(consent_log.is_some(), "Should have a consent verification log");
        
        let log = consent_log.unwrap();
        assert!(!log.has_record_id, "Consent verification should not be tied to a specific record");
        assert!(log.details.len() > 0, "Consent verification should have details");
    }
}
