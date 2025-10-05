#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env,
};

use crate::consent::{ConsentStatus, DataScope};
use super::utils::*;

#[test]
fn test_revoke_suspended_consent() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    let patient = Address::generate(&env);
    let authorized_party = Address::generate(&env);

    let consent_id = create_default_consent(&contract, &env, &patient, &authorized_party);

    // Suspend consent first
    contract.suspend_consent(&consent_id, &patient);
    let consent = contract.get_consent(&consent_id).unwrap();
    assert_eq!(consent.status, ConsentStatus::Suspended);

    // Revoke suspended consent - should work
    contract.revoke_consent(&consent_id, &patient);

    let consent = contract.get_consent(&consent_id).unwrap();
    assert_eq!(consent.status, ConsentStatus::Revoked);
    assert!(consent.revoked_at.is_some());
}

#[test]
#[should_panic(expected = "Can only revoke active or suspended consents")]
fn test_revoke_already_revoked_consent() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    let patient = Address::generate(&env);
    let authorized_party = Address::generate(&env);

    let consent_id = create_default_consent(&contract, &env, &patient, &authorized_party);

    // Revoke once
    contract.revoke_consent(&consent_id, &patient);

    // Try to revoke again - should panic
    contract.revoke_consent(&consent_id, &patient);
}

#[test]
#[should_panic(expected = "Consent not found")]
fn test_revoke_nonexistent_consent() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    let patient = Address::generate(&env);

    // Try to revoke non-existent consent
    contract.revoke_consent(&999, &patient);
}

#[test]
#[should_panic(expected = "Unauthorized: not consent owner")]
fn test_revoke_consent_wrong_patient() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    let patient = Address::generate(&env);
    let other_patient = Address::generate(&env);
    let authorized_party = Address::generate(&env);

    let consent_id = create_default_consent(&contract, &env, &patient, &authorized_party);

    // Try to revoke with different patient - should panic
    contract.revoke_consent(&consent_id, &other_patient);
}

#[test]
fn test_revoked_consent_check_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    let patient = Address::generate(&env);
    let authorized_party = Address::generate(&env);

    let scopes = create_data_scopes(&env, &[DataScope::Treatment]);
    let purpose = create_purpose(&env, "Treatment");

    let consent_id = contract.create_consent(&patient, &authorized_party, &scopes, &purpose, &None);

    // Verify consent works before revocation
    assert!(contract.check_consent(&consent_id, &authorized_party, &DataScope::Treatment));

    // Revoke consent
    contract.revoke_consent(&consent_id, &patient);

    // Verify consent check fails after revocation
    assert!(!contract.check_consent(&consent_id, &authorized_party, &DataScope::Treatment));
    assert!(!contract.is_consent_active(&consent_id));
}

#[test]
fn test_revoke_multiple_consents() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    let patient = Address::generate(&env);
    let party1 = Address::generate(&env);
    let party2 = Address::generate(&env);
    let party3 = Address::generate(&env);

    // Create multiple consents
    let id1 = create_default_consent(&contract, &env, &patient, &party1);
    let id2 = create_default_consent(&contract, &env, &patient, &party2);
    let id3 = create_default_consent(&contract, &env, &patient, &party3);

    // Revoke some consents
    contract.revoke_consent(&id1, &patient);
    contract.revoke_consent(&id3, &patient);

    // Verify correct consents are revoked
    let consent1 = contract.get_consent(&id1).unwrap();
    assert_eq!(consent1.status, ConsentStatus::Revoked);

    let consent2 = contract.get_consent(&id2).unwrap();
    assert_eq!(consent2.status, ConsentStatus::Active);

    let consent3 = contract.get_consent(&id3).unwrap();
    assert_eq!(consent3.status, ConsentStatus::Revoked);
}

#[test]
fn test_suspend_consent() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    let patient = Address::generate(&env);
    let authorized_party = Address::generate(&env);

    let consent_id = create_default_consent(&contract, &env, &patient, &authorized_party);

    // Suspend consent
    contract.suspend_consent(&consent_id, &patient);

    let consent = contract.get_consent(&consent_id).unwrap();
    assert_eq!(consent.status, ConsentStatus::Suspended);
    assert!(!contract.is_consent_active(&consent_id));
}

#[test]
#[should_panic(expected = "Can only suspend active consents")]
fn test_suspend_already_suspended_consent() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    let patient = Address::generate(&env);
    let authorized_party = Address::generate(&env);

    let consent_id = create_default_consent(&contract, &env, &patient, &authorized_party);

    // Suspend once
    contract.suspend_consent(&consent_id, &patient);

    // Try to suspend again - should panic
    contract.suspend_consent(&consent_id, &patient);
}

#[test]
#[should_panic(expected = "Can only suspend active consents")]
fn test_suspend_revoked_consent() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    let patient = Address::generate(&env);
    let authorized_party = Address::generate(&env);

    let consent_id = create_default_consent(&contract, &env, &patient, &authorized_party);

    // Revoke consent
    contract.revoke_consent(&consent_id, &patient);

    // Try to suspend revoked consent - should panic
    contract.suspend_consent(&consent_id, &patient);
}

#[test]
#[should_panic(expected = "Unauthorized: not consent owner")]
fn test_suspend_consent_wrong_patient() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    let patient = Address::generate(&env);
    let other_patient = Address::generate(&env);
    let authorized_party = Address::generate(&env);

    let consent_id = create_default_consent(&contract, &env, &patient, &authorized_party);

    // Try to suspend with different patient - should panic
    contract.suspend_consent(&consent_id, &other_patient);
}

#[test]
#[should_panic(expected = "Can only resume suspended consents")]
fn test_resume_active_consent() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    let patient = Address::generate(&env);
    let authorized_party = Address::generate(&env);

    let consent_id = create_default_consent(&contract, &env, &patient, &authorized_party);

    // Try to resume already active consent - should panic
    contract.resume_consent(&consent_id, &patient);
}

#[test]
#[should_panic(expected = "Can only resume suspended consents")]
fn test_resume_revoked_consent() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    let patient = Address::generate(&env);
    let authorized_party = Address::generate(&env);

    let consent_id = create_default_consent(&contract, &env, &patient, &authorized_party);

    // Revoke consent
    contract.revoke_consent(&consent_id, &patient);

    // Try to resume revoked consent - should panic
    contract.resume_consent(&consent_id, &patient);
}

#[test]
#[should_panic(expected = "Cannot resume expired consent")]
fn test_resume_expired_consent() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    let patient = Address::generate(&env);
    let authorized_party = Address::generate(&env);

    // Create consent expiring in 1 hour
    let consent_id = create_expiring_consent(&contract, &env, &patient, &authorized_party, 3600);

    // Suspend consent
    contract.suspend_consent(&consent_id, &patient);

    // Fast forward past expiration
    env.ledger().set_timestamp(env.ledger().timestamp() + 7200);

    // Try to resume expired consent - should panic
    contract.resume_consent(&consent_id, &patient);
}

#[test]
#[should_panic(expected = "Unauthorized: not consent owner")]
fn test_resume_consent_wrong_patient() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    let patient = Address::generate(&env);
    let other_patient = Address::generate(&env);
    let authorized_party = Address::generate(&env);

    let consent_id = create_default_consent(&contract, &env, &patient, &authorized_party);

    // Suspend consent
    contract.suspend_consent(&consent_id, &patient);

    // Try to resume with different patient - should panic
    contract.resume_consent(&consent_id, &other_patient);
}

#[test]
fn test_suspended_consent_check_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    let patient = Address::generate(&env);
    let authorized_party = Address::generate(&env);

    let scopes = create_data_scopes(&env, &[DataScope::Treatment]);
    let purpose = create_purpose(&env, "Treatment");

    let consent_id = contract.create_consent(&patient, &authorized_party, &scopes, &purpose, &None);

    // Verify consent works before suspension
    assert!(contract.check_consent(&consent_id, &authorized_party, &DataScope::Treatment));

    // Suspend consent
    contract.suspend_consent(&consent_id, &patient);

    // Verify consent check fails during suspension
    assert!(!contract.check_consent(&consent_id, &authorized_party, &DataScope::Treatment));
    assert!(!contract.is_consent_active(&consent_id));

    // Resume and verify it works again
    contract.resume_consent(&consent_id, &patient);
    assert!(contract.check_consent(&consent_id, &authorized_party, &DataScope::Treatment));
    assert!(contract.is_consent_active(&consent_id));
}

#[test]
fn test_consent_lifecycle_transitions() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    let patient = Address::generate(&env);
    let authorized_party = Address::generate(&env);

    let consent_id = create_default_consent(&contract, &env, &patient, &authorized_party);

    // Active -> Suspended -> Active -> Revoked
    assert_eq!(
        contract.get_consent(&consent_id).unwrap().status,
        ConsentStatus::Active
    );

    contract.suspend_consent(&consent_id, &patient);
    assert_eq!(
        contract.get_consent(&consent_id).unwrap().status,
        ConsentStatus::Suspended
    );

    contract.resume_consent(&consent_id, &patient);
    assert_eq!(
        contract.get_consent(&consent_id).unwrap().status,
        ConsentStatus::Active
    );

    contract.revoke_consent(&consent_id, &patient);
    assert_eq!(
        contract.get_consent(&consent_id).unwrap().status,
        ConsentStatus::Revoked
    );
}

#[test]
fn test_expiration_auto_status_change() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    let patient = Address::generate(&env);
    let authorized_party = Address::generate(&env);

    // Create consent expiring in 1 hour
    let consent_id = create_expiring_consent(&contract, &env, &patient, &authorized_party, 3600);

    // Initially active
    assert!(contract.is_consent_active(&consent_id));
    assert!(!contract.is_expired(&consent_id));

    // Fast forward past expiration
    env.ledger().set_timestamp(env.ledger().timestamp() + 7200);

    // Should be expired now
    assert!(!contract.is_consent_active(&consent_id));
    assert!(contract.is_expired(&consent_id));
}
