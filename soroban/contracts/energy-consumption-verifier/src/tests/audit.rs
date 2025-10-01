
#![cfg(test)]

extern crate std;

use soroban_sdk::{testutils::{Address as _}, Address, Env, String, log};
use crate::tests::utils::setup_test_environment;
use crate::{VerificationStatus};


#[test]
fn test_audit_log_submit_data() {
    let (env, client, admin) = setup_test_environment();
    client.initialize(&admin);

    let consumer = Address::generate(&env);
    let meter_id = String::from_str(&env, "METER_001");
    let consumption_kwh = 200;
    let meter_reading = 1300;
    let temperature = Some(22);
    let voltage = Some(230);
    let _ = client.submit_data(&consumer, &meter_id, &consumption_kwh, &meter_reading, &temperature, &voltage);

    let audit_log = client.audit_log(&0, &10);
    assert!(audit_log.len() > 0);
    assert_eq!(audit_log.get(0).unwrap().id, 0);
    assert_eq!(audit_log.get(0).unwrap().details, Some(String::from_str(&env, "Consumption data submitted for verification")));
}

#[test]
fn test_audit_log_register_verifier() {
    let (env, client, admin) = setup_test_environment();
    client.initialize(&admin);

    let consumer = Address::generate(&env);
    let verifier = Address::generate(&env);
    let meter_id = String::from_str(&env, "METER_001");
    let consumption_kwh = 200;
    let meter_reading = 1300;
    let temperature = Some(22);
    let voltage = Some(230);
    let record_id = client.submit_data(&consumer, &meter_id, &consumption_kwh, &meter_reading, &temperature, &voltage);
    client.register_verifier(&admin, &verifier);

    let comments = Some(String::from_str(&env, "Data verification Pending"));
    client.verify_data(&verifier, &record_id, &VerificationStatus::Pending, &comments);

    let audit_log = client.audit_log(&0, &10);
    assert!(audit_log.len() > 0);
    assert_eq!(audit_log.get(0).unwrap().id, 0);
    assert_eq!(audit_log.get(0).unwrap().details, Some(String::from_str(&env, "Consumption data submitted for verification")));
    assert_eq!(audit_log.get(1).unwrap().details, Some(String::from_str(&env, "New verifier registered")));
}

#[test]
fn test_audit_log_verify_data() {
    let (env, client, admin) = setup_test_environment();
    client.initialize(&admin);

    let consumer = Address::generate(&env);
    let verifier = Address::generate(&env);
    let meter_id = String::from_str(&env, "METER_001");
    let consumption_kwh = 200;
    let meter_reading = 1300;
    let temperature = Some(22);
    let voltage = Some(230);
    let record_id = client.submit_data(&consumer, &meter_id, &consumption_kwh, &meter_reading, &temperature, &voltage);
    client.register_verifier(&admin, &verifier);

    let comments = Some(String::from_str(&env, "Data verification Pending"));
    client.verify_data(&verifier, &record_id, &VerificationStatus::Pending, &comments);

    let audit_log = client.audit_log(&0, &10);
    assert!(audit_log.len() > 0);
    assert_eq!(audit_log.get(0).unwrap().id, 0);
    assert_eq!(audit_log.get(0).unwrap().details, Some(String::from_str(&env, "Consumption data submitted for verification")));
    assert_eq!(audit_log.get(0).unwrap().action, String::from_str(&env, "DATA_SUBMITTED"));
    assert_eq!(audit_log.get(1).unwrap().details, Some(String::from_str(&env, "New verifier registered")));
    assert_eq!(audit_log.get(1).unwrap().action, String::from_str(&env, "VERIFIER_REGISTERED"));
    assert_eq!(audit_log.get(2).unwrap().details, Some(String::from_str(&env, "Data verification Pending")));
    assert_eq!(audit_log.get(2).unwrap().action, String::from_str(&env, "DATA_REVIEWED"));

}

#[test]
fn test_audit_log_register_meter() {
    let (env, client, admin) = setup_test_environment();
    client.initialize(&admin);

    let consumer = Address::generate(&env);

    let meter_id = String::from_str(&env, "METER_001");
    client.register_meter(&admin, &meter_id, &consumer);

    let audit_log = client.audit_log(&0, &10);
    assert!(audit_log.len() > 0);
    log!(&env, "audit_log:{}", audit_log);
    assert_eq!(audit_log.get(0).unwrap().id, 0);
    assert_eq!(audit_log.get(0).unwrap().details, Some(String::from_str(&env, "New meter registered")));
    assert_eq!(audit_log.get(0).unwrap().action, String::from_str(&env, "METER_REGISTERED"));
}

#[test]
fn test_audit_log_register_verifier_verified() {
    let (env, client, admin) = setup_test_environment();
    client.initialize(&admin);

    let consumer = Address::generate(&env);
    let verifier = Address::generate(&env);
    let meter_id = String::from_str(&env, "METER_001");
    let consumption_kwh = 200;
    let meter_reading = 1300;
    let temperature = Some(22);
    let voltage = Some(230);
    let record_id = client.submit_data(&consumer, &meter_id, &consumption_kwh, &meter_reading, &temperature, &voltage);
    client.register_verifier(&admin, &verifier);

    let comments = Some(String::from_str(&env, "Data Verified"));
    client.verify_data(&verifier, &record_id, &VerificationStatus::Verified, &comments);

    let audit_log = client.audit_log(&0, &10);
    log!(&env, "audit_log:{}", audit_log);
    assert!(audit_log.len() > 0);
    assert_eq!(audit_log.get(2).unwrap().id, 0);
    assert_eq!(audit_log.get(2).unwrap().details, comments);
    assert_eq!(audit_log.get(2).unwrap().action, String::from_str(&env, "DATA_VERIFIED"));
}

#[test]
fn test_audit_log_register_verifier_rejected() {
    let (env, client, admin) = setup_test_environment();
    client.initialize(&admin);

    let consumer = Address::generate(&env);
    let verifier = Address::generate(&env);
    let meter_id = String::from_str(&env, "METER_001");
    let consumption_kwh = 200;
    let meter_reading = 1300;
    let temperature = Some(22);
    let voltage = Some(230);
    let record_id = client.submit_data(&consumer, &meter_id, &consumption_kwh, &meter_reading, &temperature, &voltage);
    client.register_verifier(&admin, &verifier);

    let comments = Some(String::from_str(&env, "Data Rejected"));
    client.verify_data(&verifier, &record_id, &VerificationStatus::Rejected , &comments);

    let audit_log = client.audit_log(&0, &10);
    assert!(audit_log.len() > 0);
    assert_eq!(audit_log.get(2).unwrap().id, 0);
    assert_eq!(audit_log.get(2).unwrap().details, comments);
    assert_eq!(audit_log.get(2).unwrap().action, String::from_str(&env, "DATA_REJECTED"));
}

#[test]
fn test_audit_log_register_verifier_flagged() {
    let (env, client, admin) = setup_test_environment();
    client.initialize(&admin);

    let consumer = Address::generate(&env);
    let verifier = Address::generate(&env);
    let meter_id = String::from_str(&env, "METER_001");
    let consumption_kwh = 200;
    let meter_reading = 1300;
    let temperature = Some(22);
    let voltage = Some(230);
    let record_id = client.submit_data(&consumer, &meter_id, &consumption_kwh, &meter_reading, &temperature, &voltage);
    client.register_verifier(&admin, &verifier);

    let comments = Some(String::from_str(&env, "Data Rejected"));
    client.verify_data(&verifier, &record_id, &VerificationStatus::Flagged , &comments);

    let audit_log = client.audit_log(&0, &10);
    assert!(audit_log.len() > 0);
    assert_eq!(audit_log.get(2).unwrap().id, 0);
    assert_eq!(audit_log.get(2).unwrap().details, comments);
    assert_eq!(audit_log.get(2).unwrap().action, String::from_str(&env, "DATA_FLAGGED"));
}
