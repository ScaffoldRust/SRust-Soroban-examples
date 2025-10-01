
#![cfg(test)]

extern crate std;

use soroban_sdk::{testutils::{Address as _, Ledger}, Address, Env, String, log, Vec};
use crate::tests::utils::{setup_test_environment, create_test_hla_profile, create_test_hla_profile_with_alleles};
use crate::{BloodType, OrganType, UrgencyLevel};
use std::format;

#[test]
// #[should_panic(expected = "Error(Contract, #1)")]
fn test_initialize() {
    let (_, client, admin) = setup_test_environment();
    let max_donors = 10;
    let max_recipients = 10;
    let urgency_weight = 10;
    let compatibility_threshold = 10;
    client.initialize(&admin, &max_donors, &max_recipients, &urgency_weight, &compatibility_threshold);
}


#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_already_initialize() {
    let (_, client, admin) = setup_test_environment();
    let max_donors = 10;
    let max_recipients = 10;
    let urgency_weight = 10;
    let compatibility_threshold = 10;
    client.initialize(&admin, &max_donors, &max_recipients, &urgency_weight, &compatibility_threshold);
    client.initialize(&admin, &max_donors, &max_recipients, &urgency_weight, &compatibility_threshold);
}

#[test]
fn test_register_donor() {
    let (env, client, admin) = setup_test_environment();
    let donor = Address::generate(&env);
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
}

#[test]
#[should_panic(expected = "Error(Contract, #3)")]
fn test_not_initialize() {
    let (env, client, admin) = setup_test_environment();
    let donor = Address::generate(&env);
    let medical_facility = Address::generate(&env);
    let max_donors = 100;
    let max_recipients = 500;
    let urgency_weight = 70;
    let compatibility_threshold = 70;
    let hla_profile = create_test_hla_profile(&env);
    let consent_hash = String::from_str(&env, "test_consent_hash_1234567890123456");

    client.register_donor(
        &donor,
        &BloodType::O,
        &OrganType::Kidney,
        &hla_profile,
        &25,
        &medical_facility,
        &consent_hash,
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #11)")]
fn test_donor_max_capacity_reached() {
    let (env, client, admin) = setup_test_environment();
    let donor = Address::generate(&env);
    let medical_facility = Address::generate(&env);
    let max_donors = 2;
    let max_recipients = 500;
    let urgency_weight = 70;
    let compatibility_threshold = 70;
    let hla_profile = create_test_hla_profile(&env);
    let consent_hash = String::from_str(&env, "test_consent_hash_1234567890123456");
    client.initialize(&admin, &max_donors, &max_recipients, &urgency_weight, &compatibility_threshold);

    client.register_donor(
        &donor,
        &BloodType::O,
        &OrganType::Kidney,
        &hla_profile,
        &35,
        &medical_facility,
        &consent_hash,
    );

    client.register_donor(
        &donor,
        &BloodType::O,
        &OrganType::Kidney,
        &hla_profile,
        &35,
        &medical_facility,
        &consent_hash,
    );

    client.register_donor(
        &donor,
        &BloodType::O,
        &OrganType::Kidney,
        &hla_profile,
        &35,
        &medical_facility,
        &consent_hash,
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #14)")]
fn test_hla_incoplete() {
    let (env, client, admin) = setup_test_environment();
    let donor = Address::generate(&env);
    let medical_facility = Address::generate(&env);
    let max_donors = 100;
    let max_recipients = 500;
    let urgency_weight = 70;
    let compatibility_threshold = 70;
    client.initialize(&admin, &max_donors, &max_recipients, &urgency_weight, &compatibility_threshold);

    let mut hla_profile = create_test_hla_profile(&env);
    hla_profile.hla_a.remove(0);
    hla_profile.hla_a.remove(0);
    let consent_hash = String::from_str(&env, "test_consent_hash_1234567890123456");

    client.register_donor(
        &donor,
        &BloodType::O,
        &OrganType::Kidney,
        &hla_profile,
        &40,
        &medical_facility,
        &consent_hash,
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #14)")]
fn test_young_age() {
    let (env, client, admin) = setup_test_environment();
    let donor = Address::generate(&env);
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
        &12,
        &medical_facility,
        &consent_hash,
    );
}

#[test]
fn test_register_donor_and_get_donor() {
    let (env, client, admin) = setup_test_environment();
    let donor = Address::generate(&env);
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
        &20,
        &medical_facility,
        &consent_hash,
    );

    let donor_data = client.get_donor(&donor).unwrap();
    log!(&env, "donor_data: {}", donor_data);
    assert_eq!(donor_data.age, 20);
    assert_eq!(donor_data.blood_type, BloodType::O);
    assert_eq!(donor_data.consent_hash, consent_hash);
    assert_eq!(donor_data.organ_type, OrganType::Kidney);
    assert_eq!(donor_data.hla_profile, hla_profile );
}


#[test]
fn test_register_recipient() {
   let (env, client, admin) = setup_test_environment();
    let recipient = Address::generate(&env);
    let medical_facility = Address::generate(&env);
    let max_donors = 100;
    let max_recipients = 500;
    let urgency_weight = 70;
    let compatibility_threshold = 70;
    let hla_profile = create_test_hla_profile(&env);

    client.initialize(&admin, &max_donors, &max_recipients, &urgency_weight, &compatibility_threshold);

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
}

#[test]
#[should_panic(expected = "Error(Contract, #11)")]
fn test_max_registration() {
   let (env, client, admin) = setup_test_environment();
    let recipient = Address::generate(&env);
    let medical_facility = Address::generate(&env);
    let max_donors = 100;
    let max_recipients = 2;
    let urgency_weight = 70;
    let compatibility_threshold = 70;
    let hla_profile = create_test_hla_profile(&env);

    client.initialize(&admin, &max_donors, &max_recipients, &urgency_weight, &compatibility_threshold);

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
}




#[test]
#[should_panic(expected = "Error(Contract, #14)")]
fn test_max_registration_age() {
   let (env, client, admin) = setup_test_environment();
    let recipient = Address::generate(&env);
    let medical_facility = Address::generate(&env);
    let max_donors = 100;
    let max_recipients = 2;
    let urgency_weight = 70;
    let compatibility_threshold = 70;
    let hla_profile = create_test_hla_profile(&env);

    client.initialize(&admin, &max_donors, &max_recipients, &urgency_weight, &compatibility_threshold);

    client.register_recipient(
        &recipient,
        &BloodType::A,
        &OrganType::Kidney,
        &hla_profile,
        &90,
        &UrgencyLevel::High,
        &medical_facility,
        &750,
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #12)")]
fn test_invalid_urgency_score() {
   let (env, client, admin) = setup_test_environment();
    let recipient = Address::generate(&env);
    let medical_facility = Address::generate(&env);
    let max_donors = 100;
    let max_recipients = 2;
    let urgency_weight = 70;
    let compatibility_threshold = 70;
    let hla_profile = create_test_hla_profile(&env);

    client.initialize(&admin, &max_donors, &max_recipients, &urgency_weight, &compatibility_threshold);

    client.register_recipient(
        &recipient,
        &BloodType::A,
        &OrganType::Kidney,
        &hla_profile,
        &40,
        &UrgencyLevel::High,
        &medical_facility,
        &1200,
    );

}

#[test]
#[should_panic(expected = "Error(Contract, #14)")]
fn test_registration_hla_imcomplete() {
   let (env, client, admin) = setup_test_environment();
    let recipient = Address::generate(&env);
    let medical_facility = Address::generate(&env);
    let max_donors = 100;
    let max_recipients = 2;
    let urgency_weight = 70;
    let compatibility_threshold = 70;
    let mut hla_profile = create_test_hla_profile(&env);
    hla_profile.hla_a.remove(0);
    hla_profile.hla_a.remove(0);

    client.initialize(&admin, &max_donors, &max_recipients, &urgency_weight, &compatibility_threshold);

    client.register_recipient(
        &recipient,
        &BloodType::A,
        &OrganType::Kidney,
        &hla_profile,
        &40,
        &UrgencyLevel::High,
        &medical_facility,
        &700,
    );

}


#[test]
fn test_register_recipient_and_get_recipient() {
   let (env, client, admin) = setup_test_environment();
    let recipient = Address::generate(&env);
    let medical_facility = Address::generate(&env);
    let max_donors = 100;
    let max_recipients = 2;
    let urgency_weight = 70;
    let compatibility_threshold = 70;
    let hla_profile = create_test_hla_profile(&env);
    client.initialize(&admin, &max_donors, &max_recipients, &urgency_weight, &compatibility_threshold);

    client.register_recipient(
        &recipient,
        &BloodType::AB,
        &OrganType::Kidney,
        &hla_profile,
        &40,
        &UrgencyLevel::High,
        &medical_facility,
        &700,
    );

    let recipient_data = client.get_recipient(&recipient).unwrap();
    assert_eq!(recipient_data.age, 40);
    assert_eq!(recipient_data.blood_type, BloodType::AB);
    assert_eq!(recipient_data.urgency_level, UrgencyLevel::High);
    assert_eq!(recipient_data.is_active, true);
}

#[test]
fn test_register_recipient_and_deactivate_recipient() {
   let (env, client, admin) = setup_test_environment();
    let recipient = Address::generate(&env);
    let medical_facility = Address::generate(&env);
    let max_donors = 100;
    let max_recipients = 2;
    let urgency_weight = 70;
    let compatibility_threshold = 70;
    let hla_profile = create_test_hla_profile(&env);
    client.initialize(&admin, &max_donors, &max_recipients, &urgency_weight, &compatibility_threshold);

    client.register_recipient(
        &recipient,
        &BloodType::AB,
        &OrganType::Kidney,
        &hla_profile,
        &40,
        &UrgencyLevel::High,
        &medical_facility,
        &700,
    );

    client.deactivate_recipient(&recipient, &admin);
    let recipient_data = client.get_recipient(&recipient).unwrap();
    assert_eq!(recipient_data.age, 40);
    assert_eq!(recipient_data.blood_type, BloodType::AB);
    assert_eq!(recipient_data.urgency_level, UrgencyLevel::High);
    assert_eq!(recipient_data.is_active, false);
}

#[test]
fn test_register_donor_and_deactivate_donor() {
    let (env, client, admin) = setup_test_environment();
    let donor = Address::generate(&env);
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
        &20,
        &medical_facility,
        &consent_hash,
    );

    client.deactivate_donor(&donor, &admin);

    let donor_data = client.get_donor(&donor).unwrap();
    assert_eq!(donor_data.age, 20);
    assert_eq!(donor_data.blood_type, BloodType::O);
    assert_eq!(donor_data.consent_hash, consent_hash);
    assert_eq!(donor_data.organ_type, OrganType::Kidney);
    assert_eq!(donor_data.hla_profile, hla_profile );
    assert_eq!(donor_data.is_active, false);
}
