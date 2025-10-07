#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env, String,
};

use crate::consent::DataScope;
use super::utils::*;

#[test]
fn test_create_consent_without_expiration() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    let patient = Address::generate(&env);
    let authorized_party = Address::generate(&env);

    let scopes = create_data_scopes(&env, &[DataScope::Treatment]);
    let purpose = create_purpose(&env, "Ongoing treatment");

    let consent_id = contract.create_consent(
        &patient,
        &authorized_party,
        &scopes,
        &purpose,
        &None,
    );

    let consent = contract.get_consent(&consent_id).unwrap();
    assert!(consent.expires_at.is_none());
}

#[test]
#[should_panic(expected = "At least one data scope required")]
fn test_create_consent_empty_scopes() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    let patient = Address::generate(&env);
    let authorized_party = Address::generate(&env);

    let empty_scopes = soroban_sdk::Vec::new(&env);
    let purpose = create_purpose(&env, "Treatment");

    // Should panic
    contract.create_consent(&patient, &authorized_party, &empty_scopes, &purpose, &None);
}

#[test]
#[should_panic(expected = "Purpose is required")]
fn test_create_consent_empty_purpose() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    let patient = Address::generate(&env);
    let authorized_party = Address::generate(&env);

    let scopes = create_data_scopes(&env, &[DataScope::Treatment]);
    let empty_purpose = String::from_str(&env, "");

    // Should panic
    contract.create_consent(&patient, &authorized_party, &scopes, &empty_purpose, &None);
}

#[test]
#[should_panic(expected = "Expiration must be in the future")]
fn test_create_consent_past_expiration() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    let patient = Address::generate(&env);
    let authorized_party = Address::generate(&env);

    let scopes = create_data_scopes(&env, &[DataScope::Research]);
    let purpose = create_purpose(&env, "Research study");

    // Set ledger timestamp first, then use 0 as past time
    env.ledger().set_timestamp(1000);
    let past_expiry = Some(0); // Past time (0 is before current timestamp of 1000)

    // Should panic
    contract.create_consent(&patient, &authorized_party, &scopes, &purpose, &past_expiry);
}

#[test]
fn test_create_consent_multiple_specific_scopes() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    let patient = Address::generate(&env);
    let authorized_party = Address::generate(&env);

    let scopes = create_data_scopes(
        &env,
        &[
            DataScope::Diagnostics,
            DataScope::LabResults,
            DataScope::Imaging,
            DataScope::Prescriptions,
        ],
    );
    let purpose = create_purpose(&env, "Comprehensive care");

    let consent_id = contract.create_consent(&patient, &authorized_party, &scopes, &purpose, &None);

    // Verify granted scopes work
    assert!(contract.check_consent(&consent_id, &authorized_party, &DataScope::Diagnostics));
    assert!(contract.check_consent(&consent_id, &authorized_party, &DataScope::LabResults));
    assert!(contract.check_consent(&consent_id, &authorized_party, &DataScope::Imaging));
    assert!(contract.check_consent(&consent_id, &authorized_party, &DataScope::Prescriptions));

    // Verify non-granted scopes don't work
    assert!(!contract.check_consent(&consent_id, &authorized_party, &DataScope::Research));
    assert!(!contract.check_consent(&consent_id, &authorized_party, &DataScope::Treatment));
}

#[test]
fn test_check_consent_nonexistent() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    let party = Address::generate(&env);

    // Check non-existent consent ID
    assert!(!contract.check_consent(&999, &party, &DataScope::Treatment));
}

#[test]
fn test_consent_age_tracking() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    let patient = Address::generate(&env);
    let authorized_party = Address::generate(&env);

    let consent_id = create_default_consent(&contract, &env, &patient, &authorized_party);

    // Initial age should be 0
    let age = contract.get_consent_age(&consent_id);
    assert_eq!(age, 0);

    // Fast forward time
    env.ledger().set_timestamp(env.ledger().timestamp() + 3600); // 1 hour

    let age = contract.get_consent_age(&consent_id);
    assert_eq!(age, 3600);
}

#[test]
fn test_consent_days_until_expiration() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    let patient = Address::generate(&env);
    let authorized_party = Address::generate(&env);

    // Consent expiring in 7 days
    let seven_days = 7 * 24 * 60 * 60;
    let consent_id = create_expiring_consent(&contract, &env, &patient, &authorized_party, seven_days);

    let days = contract.days_until_expiration(&consent_id);
    assert!(days.is_some());
    assert_eq!(days.unwrap(), 7);

    // Consent without expiration
    let consent_id2 = create_default_consent(&contract, &env, &patient, &authorized_party);
    let days2 = contract.days_until_expiration(&consent_id2);
    assert!(days2.is_none());
}
