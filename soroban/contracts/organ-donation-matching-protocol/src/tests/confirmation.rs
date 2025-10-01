
#![cfg(test)]

extern crate std;

use soroban_sdk::{testutils::{Address as _, Ledger}, Address, Env, String, log, Vec};
use crate::tests::utils::{setup_test_environment, create_test_hla_profile};
use crate::{BloodType, OrganType, UrgencyLevel};

#[test]
fn test_find_match_confirm_match() {
    let (env, client, admin) = setup_test_environment();
    let donor = Address::generate(&env);
    let recipient = Address::generate(&env);
    let medical_facility = Address::generate(&env);
    let max_donors = 100;
    let max_recipients = 500;
    let urgency_weight = 70;
    let compatibility_threshold = 70;
    client.initialize(&admin, &max_donors, &max_recipients, &urgency_weight, &compatibility_threshold);

    let hla_profile = create_test_hla_profile(&env);
    let consent_hash = String::from_str(&env, "test_consent_hash_1234567890123456");

    env.ledger().set_timestamp(10);
    env.ledger().set_sequence_number(1);

    client.register_donor(
        &donor,
        &BloodType::O,
        &OrganType::Kidney,
        &hla_profile,
        &35,
        &medical_facility,
        &consent_hash,
    );

    env.ledger().set_timestamp(20);
    env.ledger().set_sequence_number(2);

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

    env.ledger().set_timestamp(40);
    env.ledger().set_sequence_number(4);

    let matches = client.find_match(&recipient);
    assert_eq!(matches.len(), 1);

    let donor_data = client.get_donor(&donor).unwrap();
    let recipient_data = client.get_recipient(&recipient).unwrap();

    assert_eq!(donor_data.is_active, true);
    assert_eq!(recipient_data.is_active, true);

    let match_id = matches.get(0).unwrap().match_id;
    client.confirm_match(&match_id, &medical_facility);

    let donor_data = client.get_donor(&donor).unwrap();
    let recipient_data = client.get_recipient(&recipient).unwrap();

    assert_eq!(donor_data.is_active, false);
    assert_eq!(recipient_data.is_active, false);

    let match_data = client.get_match(&match_id).unwrap();
    assert_eq!(match_data.donor, donor);
    assert_eq!(match_data.medical_facility, medical_facility);
    assert_eq!(match_data.recipient, recipient);
}
