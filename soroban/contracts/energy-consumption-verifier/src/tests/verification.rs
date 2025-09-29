
#![cfg(test)]

extern crate std;

use soroban_sdk::{testutils::{Address as _, Ledger}, Address, Env, String, log, Vec};
use crate::tests::utils::setup_test_environment;
use crate::{VerificationStatus};
use std::format;

#[test]
#[should_panic(expected = "Error(Contract, #6)")]
fn test_validate_comsumption_data_max_voltage() {
    let (env, client, admin) = setup_test_environment();
    client.initialize(&admin);


    let consumer = Address::generate(&env);
    let meter_id = String::from_str(&env, "METER_001");
    let consumption_kwh = 200;
    let meter_reading = 1300;
    let temperature = Some(22);
    let voltage = Some(300);
    let _ = client.submit_data(&consumer, &meter_id, &consumption_kwh, &meter_reading, &temperature, &voltage);
}



#[test]
#[should_panic(expected = "Error(Contract, #6)")]
fn test_validate_comsumption_data_min_voltage() {
    let (env, client, admin) = setup_test_environment();
    client.initialize(&admin);


    let consumer = Address::generate(&env);
    let meter_id = String::from_str(&env, "METER_001");
    let consumption_kwh = 200;
    let meter_reading = 1300;
    let temperature = Some(22);
    let voltage = Some(100);
    let _ = client.submit_data(&consumer, &meter_id, &consumption_kwh, &meter_reading, &temperature, &voltage);
}


#[test]
#[should_panic(expected = "Error(Contract, #6)")]
fn test_validate_comsumption_data_max_temperature() {
    let (env, client, admin) = setup_test_environment();
    client.initialize(&admin);


    let consumer = Address::generate(&env);
    let meter_id = String::from_str(&env, "METER_001");
    let consumption_kwh = 200;
    let meter_reading = 1300;
    let temperature = Some(80);
    let voltage = Some(240);
    let _ = client.submit_data(&consumer, &meter_id, &consumption_kwh, &meter_reading, &temperature, &voltage);
}



#[test]
#[should_panic(expected = "Error(Contract, #6)")]
fn test_validate_comsumption_data_max_consumption() {
    let (env, client, admin) = setup_test_environment();
    client.initialize(&admin);


    let consumer = Address::generate(&env);
    let meter_id = String::from_str(&env, "METER_001");
    let consumption_kwh = 100_001;
    let meter_reading = 1300;
    let temperature = Some(27);
    let voltage = Some(240);
    let _ = client.submit_data(&consumer, &meter_id, &consumption_kwh, &meter_reading, &temperature, &voltage);
}

#[test]
#[should_panic(expected = "Error(Contract, #6)")]
fn test_validate_comsumption_data_zero_meter_reading() {
    let (env, client, admin) = setup_test_environment();
    client.initialize(&admin);


    let consumer = Address::generate(&env);
    let meter_id = String::from_str(&env, "METER_001");
    let consumption_kwh = 1000;
    let meter_reading = 0;
    let temperature = Some(27);
    let voltage = Some(240);
    let _ = client.submit_data(&consumer, &meter_id, &consumption_kwh, &meter_reading, &temperature, &voltage);
}

#[test]
fn test_register_verifier() {
    let (env, client, admin) = setup_test_environment();
    client.initialize(&admin);

    let verifier = Address::generate(&env);
    client.register_verifier(&admin, &verifier);
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_register_verifier_unauthorized() {
    let (env, client, admin) = setup_test_environment();
    client.initialize(&admin);

    let verifier = Address::generate(&env);
    let unauthorized = Address::generate(&env);
    client.register_verifier(&unauthorized, &verifier);
}


#[test]
fn test_verify_data() {
    let (env, client, admin) = setup_test_environment();
    client.initialize(&admin);

    let consumer = Address::generate(&env);
    let verifier = Address::generate(&env);
    let meter_id = String::from_str(&env, "METER_001");
    let consumption_kwh = 200;
    let meter_reading = 1300;
    let temperature = Some(22);
    let voltage = Some(230);
    let _ = client.submit_data(&consumer, &meter_id, &consumption_kwh, &meter_reading, &temperature, &voltage);

    client.register_verifier(&admin, &verifier);

    let record_id = client.submit_data(&consumer, &meter_id, &100, &12345, &Some(25), &Some(230));

    let comments = Some(String::from_str(&env, "Data verified"));
    client.verify_data(&verifier, &record_id, &VerificationStatus::Verified, &comments);
}

#[test]
#[should_panic(expected = "Error(Contract, #7)")]
fn test_verify_data_unauthorized() {
    let (env, client, admin) = setup_test_environment();
    client.initialize(&admin);

    let consumer = Address::generate(&env);
    let verifier = Address::generate(&env);
    let unauthorized = Address::generate(&env);
    let meter_id = String::from_str(&env, "METER_001");
    let consumption_kwh = 200;
    let meter_reading = 1300;
    let temperature = Some(22);
    let voltage = Some(230);
    let _ = client.submit_data(&consumer, &meter_id, &consumption_kwh, &meter_reading, &temperature, &voltage);

    client.register_verifier(&admin, &verifier);

    let record_id = client.submit_data(&consumer, &meter_id, &100, &12345, &Some(25), &Some(230));

    let comments = Some(String::from_str(&env, "Data verified"));
    client.verify_data(&unauthorized, &record_id, &VerificationStatus::Verified, &comments);
}

#[test]
#[should_panic(expected = "Error(Contract, #6)")]
fn test_verify_data_meter_reading_inconsistent() {
    let (env, client, admin) = setup_test_environment();
    client.initialize(&admin);

    let consumer = Address::generate(&env);
    let verifier = Address::generate(&env);
    let meter_id = String::from_str(&env, "METER_001");
    let consumption_kwh = 200;
    let meter_reading = 1300;
    let temperature = Some(22);
    let voltage = Some(230);

    for i in 1..3 {
        env.ledger().set_timestamp((1 + i).into());
        let _ = client.submit_data(&consumer, &meter_id, &(consumption_kwh * i), &(meter_reading / i), &temperature, &voltage);
    }

    client.register_verifier(&admin, &verifier);


    let consumer_records_ids = client.get_consumer_records(&consumer, &0, &10);

    let comments = Some(String::from_str(&env, "Data verified"));
    for record_id in consumer_records_ids {
        client.verify_data(&verifier, &record_id, &VerificationStatus::Verified, &comments);
    }
}

#[test]
fn test_verify_data_get_verification() {
    let (env, client, admin) = setup_test_environment();
    client.initialize(&admin);

    let consumer = Address::generate(&env);
    let verifier = Address::generate(&env);
    let meter_id = String::from_str(&env, "METER_001");
    let consumption_kwh = 200;
    let meter_reading = 1300;
    let temperature = Some(22);
    let voltage = Some(230);
    let _ = client.submit_data(&consumer, &meter_id, &consumption_kwh, &meter_reading, &temperature, &voltage);

    client.register_verifier(&admin, &verifier);

    let record_id = client.submit_data(&consumer, &meter_id, &100, &12345, &Some(25), &Some(230));

    let comments = Some(String::from_str(&env, "Data verification Pending"));
    client.verify_data(&verifier, &record_id, &VerificationStatus::Pending, &comments);

    let verification_data = client.get_verification(&record_id);
    log!(&env, "verification_data:{}", verification_data);
    assert_eq!(verification_data.record_id, record_id);
    assert_eq!(verification_data.status, VerificationStatus::Pending);
    assert_eq!(verification_data.comments, comments);
}

#[test]
fn test_get_records_by_status() {
     let (env, client, admin) = setup_test_environment();
    client.initialize(&admin);

    let consumer = Address::generate(&env);
    let verifier = Address::generate(&env);
    let meter_id = String::from_str(&env, "METER_001");
    let consumption_kwh = 200;
    let meter_reading = 1300;
    let temperature = Some(22);
    let voltage = Some(230);
    let comments = Some(String::from_str(&env, "Data verified"));
    let record_id = client.submit_data(&consumer, &meter_id, &consumption_kwh, &meter_reading, &temperature, &voltage);

    client.register_verifier(&admin, &verifier);
    client.verify_data(&verifier, &record_id, &VerificationStatus::Verified, &comments);

    let verified_records = client.get_records_by_status(&VerificationStatus::Verified, &0, &10);
    assert_eq!(verified_records.len(), 1);
    assert_eq!(verified_records.get(0).unwrap(), record_id);
}

#[test]
fn test_get_records_by_status_multiple() {
     let (env, client, admin) = setup_test_environment();
    client.initialize(&admin);

    let mut consumers = Vec::new(&env);
    for _ in 0..10 {
        consumers.push_back(Address::generate(&env));
    }
    let mut meter_ids: Vec<String> = Vec::new(&env);
    for i in 0..=10 {
        let meter_id = String::from_str(&env, &format!("METER_00{:02}", i));
        meter_ids.push_back(meter_id);
    }

    let consumption_kwh = 200;
    let meter_reading = 1300;
    let temperature = Some(22);
    let voltage = Some(230);
    let verifier = Address::generate(&env);
    let comments = Some(String::from_str(&env, "Data verified"));

    client.register_verifier(&admin, &verifier);

    let mut records = Vec::new(&env);
    for i in 0..6 {
        env.ledger().set_timestamp((1 + i).into());
        let consumer = consumers.get(i).unwrap();
        let meter_id = meter_ids.get(i).unwrap();
        let record_id = client.submit_data(&consumer, &meter_id, &consumption_kwh, &meter_reading, &temperature, &voltage);
        records.push_back(record_id);
        client.verify_data(&verifier, &record_id, &VerificationStatus::Verified, &comments);
    }

    for i in 6..10 {
        env.ledger().set_timestamp((1 + i).into());
        let consumer = consumers.get(i).unwrap();
        let meter_id = meter_ids.get(i).unwrap();
        let record_id = client.submit_data(&consumer, &meter_id, &consumption_kwh, &meter_reading, &temperature, &voltage);
        client.verify_data(&verifier, &record_id, &VerificationStatus::Rejected, &comments);
    }

    let verified_records_verified = client.get_records_by_status(&VerificationStatus::Verified, &0, &10);
    assert_eq!(verified_records_verified.len(), 6);
    for i in 0..6 {
        assert_eq!(verified_records_verified.get(i).unwrap(), records.get(i).unwrap());
    }

    let verified_records_rejected = client.get_records_by_status(&VerificationStatus::Rejected, &0, &10);
    assert_eq!(verified_records_rejected.len(), 4);
}