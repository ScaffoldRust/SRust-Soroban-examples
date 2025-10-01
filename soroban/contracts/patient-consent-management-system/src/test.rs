#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env, String,
};

fn create_test_contract(env: &Env) -> PatientConsentManagementSystemClient {
    PatientConsentManagementSystemClient::new(env, &env.register(PatientConsentManagementSystem, ()))
}

#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    contract.initialize();

    // Contract should be initialized without errors
}

#[test]
fn test_create_consent() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    contract.initialize();

    let patient = Address::generate(&env);
    let authorized_party = Address::generate(&env);

    let mut data_scopes = soroban_sdk::Vec::new(&env);
    data_scopes.push_back(consent::DataScope::Diagnostics);
    data_scopes.push_back(consent::DataScope::LabResults);

    let purpose = String::from_str(&env, "Treatment and care");
    let expires_at = Some(env.ledger().timestamp() + 86400 * 30); // 30 days

    let consent_id = contract.create_consent(
        &patient,
        &authorized_party,
        &data_scopes,
        &purpose,
        &expires_at,
    );

    assert_eq!(consent_id, 1);

    // Verify consent was created
    let consent = contract.get_consent(&consent_id);
    assert!(consent.is_some());
    let consent = consent.unwrap();
    assert_eq!(consent.patient, patient);
    assert_eq!(consent.authorized_party, authorized_party);
    assert_eq!(consent.status, consent::ConsentStatus::Active);
}

#[test]
fn test_create_multiple_consents() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    contract.initialize();

    let patient = Address::generate(&env);
    let party1 = Address::generate(&env);
    let party2 = Address::generate(&env);

    let mut scopes = soroban_sdk::Vec::new(&env);
    scopes.push_back(consent::DataScope::Diagnostics);

    let purpose = String::from_str(&env, "Treatment");

    let consent_id1 = contract.create_consent(
        &patient,
        &party1,
        &scopes,
        &purpose,
        &None,
    );

    let consent_id2 = contract.create_consent(
        &patient,
        &party2,
        &scopes,
        &purpose,
        &None,
    );

    assert_eq!(consent_id1, 1);
    assert_eq!(consent_id2, 2);

    // Verify patient has both consents
    let patient_consents = contract.get_patient_consents(&patient);
    assert_eq!(patient_consents.len(), 2);
}

#[test]
fn test_update_consent() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    contract.initialize();

    let patient = Address::generate(&env);
    let authorized_party = Address::generate(&env);

    let mut scopes = soroban_sdk::Vec::new(&env);
    scopes.push_back(consent::DataScope::Diagnostics);

    let purpose = String::from_str(&env, "Initial treatment");

    let consent_id = contract.create_consent(
        &patient,
        &authorized_party,
        &scopes,
        &purpose,
        &None,
    );

    // Update purpose
    let new_purpose = String::from_str(&env, "Updated treatment plan");
    contract.update_consent(
        &consent_id,
        &patient,
        &None,
        &Some(new_purpose.clone()),
        &None,
    );

    // Verify update
    let consent = contract.get_consent(&consent_id).unwrap();
    assert_eq!(consent.purpose, new_purpose);
}

#[test]
fn test_revoke_consent() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    contract.initialize();

    let patient = Address::generate(&env);
    let authorized_party = Address::generate(&env);

    let mut scopes = soroban_sdk::Vec::new(&env);
    scopes.push_back(consent::DataScope::Research);

    let purpose = String::from_str(&env, "Research study");

    let consent_id = contract.create_consent(
        &patient,
        &authorized_party,
        &scopes,
        &purpose,
        &None,
    );

    // Revoke consent
    contract.revoke_consent(&consent_id, &patient);

    // Verify revocation
    let consent = contract.get_consent(&consent_id).unwrap();
    assert_eq!(consent.status, consent::ConsentStatus::Revoked);
    assert!(consent.revoked_at.is_some());
}

#[test]
fn test_suspend_and_resume_consent() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    contract.initialize();

    let patient = Address::generate(&env);
    let authorized_party = Address::generate(&env);

    let mut scopes = soroban_sdk::Vec::new(&env);
    scopes.push_back(consent::DataScope::Treatment);

    let purpose = String::from_str(&env, "Treatment");

    let consent_id = contract.create_consent(
        &patient,
        &authorized_party,
        &scopes,
        &purpose,
        &None,
    );

    // Suspend consent
    contract.suspend_consent(&consent_id, &patient);

    let consent = contract.get_consent(&consent_id).unwrap();
    assert_eq!(consent.status, consent::ConsentStatus::Suspended);

    // Resume consent
    contract.resume_consent(&consent_id, &patient);

    let consent = contract.get_consent(&consent_id).unwrap();
    assert_eq!(consent.status, consent::ConsentStatus::Active);
}

#[test]
fn test_check_consent() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    contract.initialize();

    let patient = Address::generate(&env);
    let authorized_party = Address::generate(&env);

    let mut scopes = soroban_sdk::Vec::new(&env);
    scopes.push_back(consent::DataScope::Diagnostics);
    scopes.push_back(consent::DataScope::LabResults);

    let purpose = String::from_str(&env, "Treatment");

    let consent_id = contract.create_consent(
        &patient,
        &authorized_party,
        &scopes,
        &purpose,
        &None,
    );

    // Check valid scope
    let is_valid = contract.check_consent(
        &consent_id,
        &authorized_party,
        &consent::DataScope::Diagnostics,
    );
    assert_eq!(is_valid, true);

    // Check invalid scope
    let is_valid = contract.check_consent(
        &consent_id,
        &authorized_party,
        &consent::DataScope::Research,
    );
    assert_eq!(is_valid, false);

    // Check with wrong party
    let wrong_party = Address::generate(&env);
    let is_valid = contract.check_consent(
        &consent_id,
        &wrong_party,
        &consent::DataScope::Diagnostics,
    );
    assert_eq!(is_valid, false);
}

#[test]
fn test_all_data_scope() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    contract.initialize();

    let patient = Address::generate(&env);
    let authorized_party = Address::generate(&env);

    let mut scopes = soroban_sdk::Vec::new(&env);
    scopes.push_back(consent::DataScope::AllData);

    let purpose = String::from_str(&env, "Complete care");

    let consent_id = contract.create_consent(
        &patient,
        &authorized_party,
        &scopes,
        &purpose,
        &None,
    );

    // Check various scopes - all should be valid with AllData
    assert_eq!(
        contract.check_consent(&consent_id, &authorized_party, &consent::DataScope::Diagnostics),
        true
    );
    assert_eq!(
        contract.check_consent(&consent_id, &authorized_party, &consent::DataScope::Research),
        true
    );
    assert_eq!(
        contract.check_consent(&consent_id, &authorized_party, &consent::DataScope::Imaging),
        true
    );
}

#[test]
fn test_consent_expiration() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    contract.initialize();

    let patient = Address::generate(&env);
    let authorized_party = Address::generate(&env);

    let mut scopes = soroban_sdk::Vec::new(&env);
    scopes.push_back(consent::DataScope::Research);

    let purpose = String::from_str(&env, "Research study");
    let expires_at = Some(env.ledger().timestamp() + 3600); // 1 hour

    let consent_id = contract.create_consent(
        &patient,
        &authorized_party,
        &scopes,
        &purpose,
        &expires_at,
    );

    // Should be active initially
    assert_eq!(contract.is_consent_active(&consent_id), true);
    assert_eq!(contract.is_expired(&consent_id), false);

    // Fast forward time
    env.ledger().set_timestamp(env.ledger().timestamp() + 7200); // 2 hours

    // Should now be expired
    assert_eq!(contract.is_expired(&consent_id), true);
    assert_eq!(contract.is_consent_active(&consent_id), false);
}

#[test]
fn test_audit_log() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    contract.initialize();

    let patient = Address::generate(&env);
    let authorized_party = Address::generate(&env);

    let mut scopes = soroban_sdk::Vec::new(&env);
    scopes.push_back(consent::DataScope::Treatment);

    let purpose = String::from_str(&env, "Treatment");

    let consent_id = contract.create_consent(
        &patient,
        &authorized_party,
        &scopes,
        &purpose,
        &None,
    );

    // Perform some actions
    contract.suspend_consent(&consent_id, &patient);
    contract.resume_consent(&consent_id, &patient);
    contract.revoke_consent(&consent_id, &patient);

    // Check audit log
    let audit_log = contract.audit_log(&consent_id);
    assert_eq!(audit_log.len(), 4); // Created, Suspended, Resumed, Revoked
}

#[test]
fn test_access_logging() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    contract.initialize();

    let patient = Address::generate(&env);
    let authorized_party = Address::generate(&env);

    let mut scopes = soroban_sdk::Vec::new(&env);
    scopes.push_back(consent::DataScope::Diagnostics);

    let purpose = String::from_str(&env, "Treatment");

    let consent_id = contract.create_consent(
        &patient,
        &authorized_party,
        &scopes,
        &purpose,
        &None,
    );

    // Log access
    let access_purpose = String::from_str(&env, "Viewing test results");
    contract.log_access(
        &consent_id,
        &authorized_party,
        &consent::DataScope::Diagnostics,
        &access_purpose,
    );

    // Check access logs
    let access_logs = contract.get_access_logs(&consent_id);
    assert_eq!(access_logs.len(), 1);
    assert_eq!(access_logs.get(0).unwrap().accessed_by, authorized_party);
}

#[test]
fn test_audit_summary() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    contract.initialize();

    let patient = Address::generate(&env);
    let authorized_party = Address::generate(&env);

    let mut scopes = soroban_sdk::Vec::new(&env);
    scopes.push_back(consent::DataScope::Research);

    let purpose = String::from_str(&env, "Research");

    let consent_id = contract.create_consent(
        &patient,
        &authorized_party,
        &scopes,
        &purpose,
        &None,
    );

    // Log some accesses
    contract.log_access(
        &consent_id,
        &authorized_party,
        &consent::DataScope::Research,
        &String::from_str(&env, "Access 1"),
    );

    let summary = contract.get_audit_summary(&patient, &consent_id);
    assert!(summary.is_some());
    let summary = summary.unwrap();
    assert_eq!(summary.consent_id, consent_id);
    assert_eq!(summary.total_events, 2); // Created event + access event
    assert_eq!(summary.total_accesses, 1);
}

#[test]
fn test_get_patient_consents() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    contract.initialize();

    let patient = Address::generate(&env);
    let party1 = Address::generate(&env);
    let party2 = Address::generate(&env);

    let mut scopes = soroban_sdk::Vec::new(&env);
    scopes.push_back(consent::DataScope::Treatment);

    let purpose = String::from_str(&env, "Care");

    contract.create_consent(&patient, &party1, &scopes, &purpose, &None);
    contract.create_consent(&patient, &party2, &scopes, &purpose, &None);

    let patient_consents = contract.get_patient_consents(&patient);
    assert_eq!(patient_consents.len(), 2);
}

#[test]
fn test_get_party_consents() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    contract.initialize();

    let patient1 = Address::generate(&env);
    let patient2 = Address::generate(&env);
    let authorized_party = Address::generate(&env);

    let mut scopes = soroban_sdk::Vec::new(&env);
    scopes.push_back(consent::DataScope::Treatment);

    let purpose = String::from_str(&env, "Care");

    contract.create_consent(&patient1, &authorized_party, &scopes, &purpose, &None);
    contract.create_consent(&patient2, &authorized_party, &scopes, &purpose, &None);

    let party_consents = contract.get_party_consents(&authorized_party);
    assert_eq!(party_consents.len(), 2);
}

#[test]
fn test_validate_consent_params() {
    let env = Env::default();
    let contract = create_test_contract(&env);

    let valid_purpose = String::from_str(&env, "Treatment and care");
    let valid_expiry = Some(env.ledger().timestamp() + 86400);

    assert_eq!(contract.validate_consent_params(&valid_purpose, &valid_expiry), true);

    // Invalid: empty purpose
    let invalid_purpose = String::from_str(&env, "");
    assert_eq!(contract.validate_consent_params(&invalid_purpose, &valid_expiry), false);

    // Invalid: expiry in past (using 0 as a past time)
    let invalid_expiry = Some(0);
    assert_eq!(contract.validate_consent_params(&valid_purpose, &invalid_expiry), false);
}

#[test]
fn test_get_remaining_validity() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    contract.initialize();

    let patient = Address::generate(&env);
    let authorized_party = Address::generate(&env);

    let mut scopes = soroban_sdk::Vec::new(&env);
    scopes.push_back(consent::DataScope::Research);

    let purpose = String::from_str(&env, "Research");
    let expires_at = Some(env.ledger().timestamp() + 86400); // 1 day

    let consent_id = contract.create_consent(
        &patient,
        &authorized_party,
        &scopes,
        &purpose,
        &expires_at,
    );

    let remaining = contract.get_remaining_validity(&consent_id);
    assert!(remaining.is_some());
    assert_eq!(remaining.unwrap(), 86400);

    // No expiration set
    let consent_id2 = contract.create_consent(
        &patient,
        &authorized_party,
        &scopes,
        &purpose,
        &None,
    );

    let remaining2 = contract.get_remaining_validity(&consent_id2);
    assert!(remaining2.is_none());
}

#[test]
fn test_is_expiring_soon() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    contract.initialize();

    let patient = Address::generate(&env);
    let authorized_party = Address::generate(&env);

    let mut scopes = soroban_sdk::Vec::new(&env);
    scopes.push_back(consent::DataScope::Treatment);

    let purpose = String::from_str(&env, "Care");
    let expires_at = Some(env.ledger().timestamp() + 3600); // 1 hour

    let consent_id = contract.create_consent(
        &patient,
        &authorized_party,
        &scopes,
        &purpose,
        &expires_at,
    );

    // Should be expiring soon with threshold of 2 hours
    assert_eq!(contract.is_expiring_soon(&consent_id, &7200), true);

    // Should not be expiring soon with threshold of 30 minutes
    assert_eq!(contract.is_expiring_soon(&consent_id, &1800), false);
}

#[test]
fn test_gdpr_hipaa_compliance() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    contract.initialize();

    let patient = Address::generate(&env);
    let authorized_party = Address::generate(&env);

    let mut scopes = soroban_sdk::Vec::new(&env);
    scopes.push_back(consent::DataScope::Treatment);

    let purpose = String::from_str(&env, "Medical treatment");

    let consent_id = contract.create_consent(
        &patient,
        &authorized_party,
        &scopes,
        &purpose,
        &None,
    );

    // Should be compliant
    assert_eq!(contract.check_gdpr_compliance(&consent_id), true);
    assert_eq!(contract.check_hipaa_compliance(&consent_id), true);
}

#[test]
fn test_full_lifecycle() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    contract.initialize();

    let patient = Address::generate(&env);
    let authorized_party = Address::generate(&env);

    let mut scopes = soroban_sdk::Vec::new(&env);
    scopes.push_back(consent::DataScope::Diagnostics);

    let purpose = String::from_str(&env, "Treatment");
    let expires_at = Some(env.ledger().timestamp() + 86400);

    // Create consent
    let consent_id = contract.create_consent(
        &patient,
        &authorized_party,
        &scopes,
        &purpose,
        &expires_at,
    );

    // Check consent
    assert_eq!(
        contract.check_consent(&consent_id, &authorized_party, &consent::DataScope::Diagnostics),
        true
    );

    // Log access
    contract.log_access(
        &consent_id,
        &authorized_party,
        &consent::DataScope::Diagnostics,
        &String::from_str(&env, "Viewing results"),
    );

    // Update consent
    let new_purpose = String::from_str(&env, "Updated treatment plan");
    contract.update_consent(&consent_id, &patient, &None, &Some(new_purpose), &None);

    // Suspend consent
    contract.suspend_consent(&consent_id, &patient);
    assert_eq!(contract.is_consent_active(&consent_id), false);

    // Resume consent
    contract.resume_consent(&consent_id, &patient);
    assert_eq!(contract.is_consent_active(&consent_id), true);

    // Revoke consent
    contract.revoke_consent(&consent_id, &patient);

    let consent = contract.get_consent(&consent_id).unwrap();
    assert_eq!(consent.status, consent::ConsentStatus::Revoked);

    // Verify audit trail
    let audit_log = contract.audit_log(&consent_id);
    assert!(audit_log.len() >= 5); // Multiple events recorded
}
