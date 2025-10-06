#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env, String,
};

use crate::{
    audit::AuditEventType,
    consent::DataScope,
};
use super::utils::*;

#[test]
fn test_audit_log_consent_created() {
    let env = Env::default();
    env.mock_all_auths();

    // Set ledger timestamp to non-zero value
    env.ledger().set_timestamp(1000);

    let contract = create_test_contract(&env);
    let patient = Address::generate(&env);
    let authorized_party = Address::generate(&env);

    let consent_id = create_default_consent(&contract, &env, &patient, &authorized_party);

    // Check audit log
    let audit_log = contract.audit_log(&consent_id);
    assert_eq!(audit_log.len(), 1);

    let event = audit_log.get(0).unwrap();
    assert_eq!(event.consent_id, consent_id);
    assert_eq!(event.event_type, AuditEventType::ConsentCreated);
    assert_eq!(event.actor, patient);
    assert!(event.timestamp >= 1000);
}

#[test]
fn test_audit_log_consent_updated() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    let patient = Address::generate(&env);
    let authorized_party = Address::generate(&env);

    let consent_id = create_default_consent(&contract, &env, &patient, &authorized_party);

    // Update consent
    let new_purpose = String::from_str(&env, "Updated purpose");
    contract.update_consent(&consent_id, &patient, &None, &Some(new_purpose), &None);

    // Check audit log
    let audit_log = contract.audit_log(&consent_id);
    assert_eq!(audit_log.len(), 2); // Created + Updated

    let update_event = audit_log.get(1).unwrap();
    assert_eq!(update_event.event_type, AuditEventType::ConsentUpdated);
    assert_eq!(update_event.actor, patient);
}

#[test]
fn test_audit_log_consent_revoked() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    let patient = Address::generate(&env);
    let authorized_party = Address::generate(&env);

    let consent_id = create_default_consent(&contract, &env, &patient, &authorized_party);

    // Revoke consent
    contract.revoke_consent(&consent_id, &patient);

    // Check audit log
    let audit_log = contract.audit_log(&consent_id);
    assert_eq!(audit_log.len(), 2); // Created + Revoked

    let revoke_event = audit_log.get(1).unwrap();
    assert_eq!(revoke_event.event_type, AuditEventType::ConsentRevoked);
    assert_eq!(revoke_event.actor, patient);
}

#[test]
fn test_audit_log_consent_suspended_and_resumed() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    let patient = Address::generate(&env);
    let authorized_party = Address::generate(&env);

    let consent_id = create_default_consent(&contract, &env, &patient, &authorized_party);

    // Suspend consent
    contract.suspend_consent(&consent_id, &patient);

    // Resume consent
    contract.resume_consent(&consent_id, &patient);

    // Check audit log
    let audit_log = contract.audit_log(&consent_id);
    assert_eq!(audit_log.len(), 3); // Created + Suspended + Resumed

    let suspend_event = audit_log.get(1).unwrap();
    assert_eq!(suspend_event.event_type, AuditEventType::ConsentSuspended);

    let resume_event = audit_log.get(2).unwrap();
    assert_eq!(resume_event.event_type, AuditEventType::ConsentResumed);
}

#[test]
fn test_multiple_access_logging() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    let patient = Address::generate(&env);
    let authorized_party = Address::generate(&env);

    let scopes = create_data_scopes(&env, &[DataScope::Diagnostics, DataScope::LabResults]);
    let purpose = create_purpose(&env, "Treatment");

    let consent_id = contract.create_consent(&patient, &authorized_party, &scopes, &purpose, &None);

    // Log multiple accesses
    contract.log_access(
        &consent_id,
        &authorized_party,
        &DataScope::Diagnostics,
        &String::from_str(&env, "Access 1"),
    );

    contract.log_access(
        &consent_id,
        &authorized_party,
        &DataScope::LabResults,
        &String::from_str(&env, "Access 2"),
    );

    contract.log_access(
        &consent_id,
        &authorized_party,
        &DataScope::Diagnostics,
        &String::from_str(&env, "Access 3"),
    );

    // Check access logs
    let access_logs = contract.get_access_logs(&consent_id);
    assert_eq!(access_logs.len(), 3);

    // Verify different scopes were logged
    assert_eq!(access_logs.get(0).unwrap().data_scope, DataScope::Diagnostics);
    assert_eq!(access_logs.get(1).unwrap().data_scope, DataScope::LabResults);
    assert_eq!(access_logs.get(2).unwrap().data_scope, DataScope::Diagnostics);
}

#[test]
fn test_access_logging_creates_audit_event() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    let patient = Address::generate(&env);
    let authorized_party = Address::generate(&env);

    let consent_id = create_default_consent(&contract, &env, &patient, &authorized_party);

    // Log access
    contract.log_access(
        &consent_id,
        &authorized_party,
        &DataScope::Treatment,
        &String::from_str(&env, "Accessing treatment data"),
    );

    // Check that access created an audit event
    let audit_log = contract.audit_log(&consent_id);
    assert_eq!(audit_log.len(), 2); // Created + Accessed

    let access_event = audit_log.get(1).unwrap();
    assert_eq!(access_event.event_type, AuditEventType::ConsentAccessed);
    assert_eq!(access_event.actor, authorized_party);
}

#[test]
fn test_audit_summary_unauthorized_patient() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    let patient = Address::generate(&env);
    let other_patient = Address::generate(&env);
    let authorized_party = Address::generate(&env);

    let consent_id = create_default_consent(&contract, &env, &patient, &authorized_party);

    // Try to get summary with wrong patient
    let summary = contract.get_audit_summary(&other_patient, &consent_id);
    assert!(summary.is_none());
}

#[test]
fn test_audit_summary_complex_lifecycle() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    let patient = Address::generate(&env);
    let authorized_party = Address::generate(&env);

    let consent_id = create_default_consent(&contract, &env, &patient, &authorized_party);

    // Perform various operations
    contract.log_access(
        &consent_id,
        &authorized_party,
        &DataScope::Treatment,
        &String::from_str(&env, "Access 1"),
    );

    contract.suspend_consent(&consent_id, &patient);
    contract.resume_consent(&consent_id, &patient);

    contract.log_access(
        &consent_id,
        &authorized_party,
        &DataScope::Treatment,
        &String::from_str(&env, "Access 2"),
    );

    contract.revoke_consent(&consent_id, &patient);

    // Get summary
    let summary = contract.get_audit_summary(&patient, &consent_id);
    assert!(summary.is_some());

    let summary = summary.unwrap();
    assert_eq!(summary.total_events, 6); // Created, Accessed, Suspended, Resumed, Accessed, Revoked
    assert_eq!(summary.total_accesses, 2);
}

#[test]
fn test_audit_event_timestamps_sequential() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    let patient = Address::generate(&env);
    let authorized_party = Address::generate(&env);

    let consent_id = create_default_consent(&contract, &env, &patient, &authorized_party);

    // Perform operations with time progression
    env.ledger().set_timestamp(env.ledger().timestamp() + 100);
    contract.suspend_consent(&consent_id, &patient);

    env.ledger().set_timestamp(env.ledger().timestamp() + 100);
    contract.resume_consent(&consent_id, &patient);

    env.ledger().set_timestamp(env.ledger().timestamp() + 100);
    contract.revoke_consent(&consent_id, &patient);

    // Check timestamps are sequential
    let audit_log = contract.audit_log(&consent_id);
    assert_eq!(audit_log.len(), 4);

    let mut last_timestamp = 0;
    for i in 0..audit_log.len() {
        let event = audit_log.get(i).unwrap();
        assert!(event.timestamp >= last_timestamp);
        last_timestamp = event.timestamp;
    }
}

#[test]
fn test_access_log_nonexistent_consent() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);

    // Get access logs for non-existent consent
    let access_logs = contract.get_access_logs(&999);
    assert_eq!(access_logs.len(), 0);
}

#[test]
fn test_audit_log_nonexistent_consent() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);

    // Get audit log for non-existent consent
    let audit_log = contract.audit_log(&999);
    assert_eq!(audit_log.len(), 0);
}

#[test]
fn test_audit_immutability() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    let patient = Address::generate(&env);
    let authorized_party = Address::generate(&env);

    let consent_id = create_default_consent(&contract, &env, &patient, &authorized_party);

    // Get initial audit log
    let initial_log = contract.audit_log(&consent_id);
    assert_eq!(initial_log.len(), 1);
    let initial_event = initial_log.get(0).unwrap();

    // Perform more operations
    contract.suspend_consent(&consent_id, &patient);
    contract.resume_consent(&consent_id, &patient);

    // Get updated audit log
    let updated_log = contract.audit_log(&consent_id);
    assert_eq!(updated_log.len(), 3);

    // Verify original event remains unchanged
    let first_event = updated_log.get(0).unwrap();
    assert_eq!(first_event.event_id, initial_event.event_id);
    assert_eq!(first_event.event_type, initial_event.event_type);
    assert_eq!(first_event.timestamp, initial_event.timestamp);
}

#[test]
fn test_compliance_nonexistent_consent() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);

    // Check compliance for non-existent consent
    assert!(!contract.check_gdpr_compliance(&999));
    assert!(!contract.check_hipaa_compliance(&999));
}
