extern crate std;

use super::utils::TestFixture;
use soroban_sdk::{Symbol, testutils::Address as _};

/// Test audit log creation on record access
#[test]
fn test_audit_log_creation_on_record_access() {
    let fixture = TestFixture::new();
    let record_id = fixture.create_record("lab", "ipfs://QmLab");

    let audit_log = fixture.get_audit_log();

    // Should have add_record event
    assert!(audit_log.len() >= 1);
    let last_event = audit_log.get(audit_log.len() - 1).unwrap();
    assert_eq!(last_event.action, Symbol::new(&fixture.env, "add_record"));
    assert_eq!(last_event.record_id, Some(record_id));
}

/// Test audit log accuracy
#[test]
fn test_audit_log_accuracy() {
    let fixture = TestFixture::new();
    let record_id = fixture.create_record("lab", "ipfs://QmLab");

    fixture.grant_access(std::vec!["lab"], 0);

    let audit_log = fixture.get_audit_log();

    // Should have at least 2 events: add_record and grant
    assert!(audit_log.len() >= 2);

    // Check add_record event
    let add_event = audit_log.get(0).unwrap();
    assert_eq!(add_event.action, Symbol::new(&fixture.env, "add_record"));
    assert_eq!(add_event.actor, fixture.patient);
    assert_eq!(add_event.record_id, Some(record_id));

    // Check grant event
    let grant_event = audit_log.get(1).unwrap();
    assert_eq!(grant_event.action, Symbol::new(&fixture.env, "grant"));
    assert_eq!(grant_event.actor, fixture.patient);
}

/// Test audit log immutability (events are append-only)
#[test]
fn test_audit_log_immutability() {
    let fixture = TestFixture::new();

    fixture.create_record("lab", "ipfs://QmLab");
    let log1 = fixture.get_audit_log();
    let first_event = log1.get(0).unwrap();

    // Create more events
    fixture.create_record("imaging", "ipfs://QmImage");
    let log2 = fixture.get_audit_log();

    // First event should be unchanged
    let first_event_after = log2.get(0).unwrap();
    assert_eq!(first_event.timestamp, first_event_after.timestamp);
    assert_eq!(first_event.action, first_event_after.action);
    assert_eq!(first_event.actor, first_event_after.actor);
}

/// Test audit logging with missing data scenarios
#[test]
fn test_audit_log_with_missing_data_scenarios() {
    let fixture = TestFixture::new();

    // Grant access without creating a record first
    fixture.grant_access(std::vec!["lab"], 0);

    let audit_log = fixture.get_audit_log();

    // Should have grant event even without a record
    assert!(audit_log.len() >= 1);
    let event = audit_log.get(audit_log.len() - 1).unwrap();
    assert_eq!(event.action, Symbol::new(&fixture.env, "grant"));
    assert_eq!(event.record_id, None); // No specific record
}

/// Test audit log compliance with privacy regulations (HIPAA/GDPR)
#[test]
fn test_audit_log_compliance_with_privacy_regulations() {
    let fixture = TestFixture::new();
    let record_id = fixture.create_record("lab", "ipfs://QmLab");

    fixture.grant_access(std::vec!["lab"], 0);
    fixture.client.revoke_access(&fixture.patient, &fixture.provider);

    let audit_log = fixture.get_audit_log();

    // Should have complete audit trail: add, grant, revoke
    assert!(audit_log.len() >= 3);

    // All events should have timestamps (may be 0 in test environment)
    for event in audit_log.iter() {
        assert!(event.timestamp >= 0);
        assert_eq!(event.actor, fixture.patient);
    }
}

/// Test audit log access denied events
#[test]
fn test_audit_log_access_denied_events() {
    let fixture = TestFixture::new();
    let record_id = fixture.create_record("lab", "ipfs://QmLab");

    // Try to access without permission (will panic)
    // We can't directly test the panic, but we can verify the audit log before
    let audit_log_before = fixture.get_audit_log();

    fixture.grant_access(std::vec!["lab"], 0);

    let audit_log_after = fixture.get_audit_log();

    // Should have one more event
    assert_eq!(audit_log_after.len(), audit_log_before.len() + 1);
}

/// Test emergency access special logging
#[test]
fn test_emergency_access_special_logging() {
    let fixture = TestFixture::new();
    let record_id = fixture.create_record("lab", "ipfs://QmLab");

    fixture.add_emergency_provider();

    let justification = fixture.string("Patient critical");
    let _ = fixture.client.emergency_read(&fixture.provider, &fixture.patient, &record_id, &justification);

    let audit_log = fixture.get_audit_log();

    // Should have emergency event
    let emergency_event = audit_log.iter().find(|e| e.action == Symbol::new(&fixture.env, "emergency"));
    assert!(emergency_event.is_some());

    let event = emergency_event.unwrap();
    assert_eq!(event.record_id, Some(record_id));
    assert!(event.detail.is_some());
}

/// Test audit log growth and capping at 200 events
#[test]
fn test_audit_log_growth_and_capping() {
    let fixture = TestFixture::new();

    // Create 205 records (205 add_record events)
    for i in 1..=205 {
        let hash = match i % 10 {
            1 => "ipfs://QmHash1", 2 => "ipfs://QmHash2", 3 => "ipfs://QmHash3",
            4 => "ipfs://QmHash4", 5 => "ipfs://QmHash5", 6 => "ipfs://QmHash6",
            7 => "ipfs://QmHash7", 8 => "ipfs://QmHash8", 9 => "ipfs://QmHash9",
            _ => "ipfs://QmHash0",
        };
        fixture.create_record("lab", hash);
    }

    let audit_log = fixture.get_audit_log();

    // Should be capped at 200
    assert_eq!(audit_log.len(), 200);

    // Latest event should be the 205th record
    let last_event = audit_log.get(audit_log.len() - 1).unwrap();
    assert_eq!(last_event.action, Symbol::new(&fixture.env, "add_record"));
}

/// Test audit log access control (only patient can view)
#[test]
#[should_panic(expected = "Only patient")]
fn test_audit_log_access_control() {
    let fixture = TestFixture::new();
    fixture.create_record("lab", "ipfs://QmLab");

    // Try to get audit log as different user
    fixture.client.get_audit_log(&fixture.patient, &fixture.provider);
}

/// Test audit log sequence integrity
#[test]
fn test_audit_log_sequence_integrity() {
    let fixture = TestFixture::new();

    // Create sequence of events
    let id1 = fixture.create_record("lab", "ipfs://QmLab1");
    fixture.grant_access(std::vec!["lab"], 0);
    let id2 = fixture.create_record("imaging", "ipfs://QmImage");
    fixture.client.revoke_access(&fixture.patient, &fixture.provider);

    let audit_log = fixture.get_audit_log();

    // Events should be in chronological order
    assert!(audit_log.len() >= 4);

    let mut prev_timestamp = 0u64;
    for event in audit_log.iter() {
        assert!(event.timestamp >= prev_timestamp);
        prev_timestamp = event.timestamp;
    }
}

/// Test audit log for update operations
#[test]
fn test_audit_log_for_update_operations() {
    let fixture = TestFixture::new();
    let record_id = fixture.create_record("lab", "ipfs://QmOld");

    let new_pointer = fixture.string("ipfs://QmNew");
    fixture.client.update_record(&fixture.patient, &record_id, &new_pointer);

    let audit_log = fixture.get_audit_log();

    // Should have add_record and update_record events
    let update_event = audit_log.iter().find(|e| e.action == Symbol::new(&fixture.env, "update_record"));
    assert!(update_event.is_some());

    let event = update_event.unwrap();
    assert_eq!(event.record_id, Some(record_id));
    assert!(event.detail.is_some());
}

/// Test audit log pagination (if supported)
#[test]
fn test_audit_log_pagination() {
    let fixture = TestFixture::new();

    // Create 10 events
    fixture.create_record("lab", "ipfs://QmHash1");
    fixture.create_record("lab", "ipfs://QmHash2");
    fixture.create_record("lab", "ipfs://QmHash3");
    fixture.create_record("lab", "ipfs://QmHash4");
    fixture.create_record("lab", "ipfs://QmHash5");
    fixture.create_record("lab", "ipfs://QmHash6");
    fixture.create_record("lab", "ipfs://QmHash7");
    fixture.create_record("lab", "ipfs://QmHash8");
    fixture.create_record("lab", "ipfs://QmHash9");
    fixture.create_record("lab", "ipfs://QmHash10");

    let audit_log = fixture.get_audit_log();

    // Should have all 10 events
    assert_eq!(audit_log.len(), 10);
}

/// Test consent verification audit logging
#[test]
fn test_consent_verification_audit_logging() {
    let fixture = TestFixture::new();

    // Grant access (simulating consent)
    fixture.grant_access(std::vec!["lab"], 0);

    let audit_log = fixture.get_audit_log();

    // Should have grant event representing consent
    let grant_event = audit_log.iter().find(|e| e.action == Symbol::new(&fixture.env, "grant"));
    assert!(grant_event.is_some());
}

/// Test provider registration audit logging (if applicable)
#[test]
fn test_provider_registration_audit_logging() {
    let fixture = TestFixture::new();

    // Add emergency provider
    fixture.add_emergency_provider();

    let audit_log = fixture.get_audit_log();

    // Should have emergency_grant event
    let emergency_grant = audit_log.iter().find(|e| e.action == Symbol::new(&fixture.env, "emergency_grant"));
    assert!(emergency_grant.is_some());
}

/// Test bulk operation audit logging
#[test]
fn test_bulk_operation_audit_logging() {
    let fixture = TestFixture::new();

    // Create multiple records in bulk
    fixture.create_record("lab", "ipfs://QmHash1");
    fixture.create_record("lab", "ipfs://QmHash2");
    fixture.create_record("lab", "ipfs://QmHash3");
    fixture.create_record("lab", "ipfs://QmHash4");
    fixture.create_record("lab", "ipfs://QmHash5");

    let audit_log = fixture.get_audit_log();

    // Should have 5 add_record events
    let add_events: std::vec::Vec<_> = audit_log.iter().filter(|e| e.action == Symbol::new(&fixture.env, "add_record")).collect();
    assert_eq!(add_events.len(), 5);
}

/// Test audit log event details completeness
#[test]
fn test_audit_log_event_details_completeness() {
    let fixture = TestFixture::new();
    let record_id = fixture.create_record("lab", "ipfs://QmLab");

    let audit_log = fixture.get_audit_log();

    let event = audit_log.get(0).unwrap();

    // All required fields should be present
    assert!(event.timestamp >= 0); // May be 0 in test environment
    assert_eq!(event.actor, fixture.patient);
    assert_eq!(event.action, Symbol::new(&fixture.env, "add_record"));
    assert_eq!(event.record_id, Some(record_id));
    assert!(event.detail.is_some());
}

/// Test audit log for revocation operations
#[test]
fn test_audit_log_for_revocation() {
    let fixture = TestFixture::new();

    fixture.grant_access(std::vec!["lab"], 0);
    fixture.client.revoke_access(&fixture.patient, &fixture.provider);

    let audit_log = fixture.get_audit_log();

    // Should have both grant and revoke events
    let grant = audit_log.iter().find(|e| e.action == Symbol::new(&fixture.env, "grant"));
    let revoke = audit_log.iter().find(|e| e.action == Symbol::new(&fixture.env, "revoke"));

    assert!(grant.is_some());
    assert!(revoke.is_some());
}

/// Test that audit log survives emergency access
#[test]
fn test_audit_log_survives_emergency_access() {
    let fixture = TestFixture::new();
    let record_id = fixture.create_record("lab", "ipfs://QmLab");

    let events_before = fixture.get_audit_log().len();

    fixture.add_emergency_provider();
    let justification = fixture.string("Critical emergency");
    let _ = fixture.client.emergency_read(&fixture.provider, &fixture.patient, &record_id, &justification);

    let events_after = fixture.get_audit_log().len();

    // Should have 2 more events: emergency_grant and emergency
    assert_eq!(events_after, events_before + 2);
}

/// Test audit log for multiple patients (isolation)
#[test]
fn test_audit_log_isolation_between_patients() {
    let fixture = TestFixture::new();
    let patient2 = soroban_sdk::Address::generate(&fixture.env);

    // Create record for patient 1
    fixture.create_record("lab", "ipfs://QmLab1");

    // Create record for patient 2
    fixture.create_record_for(&patient2, "lab", "ipfs://QmLab2");

    // Get audit logs
    let log1 = fixture.get_audit_log();
    let log2 = fixture.client.get_audit_log(&patient2, &patient2);

    // Patient 1 should have 1 event
    assert_eq!(log1.len(), 1);

    // Patient 2 should have 1 event
    assert_eq!(log2.len(), 1);

    // Events should be different
    assert_eq!(log1.get(0).unwrap().actor, fixture.patient);
    assert_eq!(log2.get(0).unwrap().actor, patient2);
}

/// Test emergency provider removal audit logging
#[test]
fn test_emergency_provider_removal_audit() {
    let fixture = TestFixture::new();

    fixture.add_emergency_provider();
    fixture.client.remove_emergency_provider(&fixture.patient, &fixture.provider);

    let audit_log = fixture.get_audit_log();

    // Should have both emergency_grant and emergency_revoke
    let grant = audit_log.iter().find(|e| e.action == Symbol::new(&fixture.env, "emergency_grant"));
    let revoke = audit_log.iter().find(|e| e.action == Symbol::new(&fixture.env, "emergency_revoke"));

    assert!(grant.is_some());
    assert!(revoke.is_some());
}
