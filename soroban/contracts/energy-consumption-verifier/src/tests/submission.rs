
#![cfg(test)]

extern crate std;

use soroban_sdk::{testutils::{Address as _, Ledger}, Address, Env, String, log, Vec};
use crate::tests::utils::setup_test_environment;
use std::format;

#[test]
#[should_panic(expected = "Error(Contract, #1)")]
fn test_initialize() {
    let (_, client, admin) = setup_test_environment();
    client.initialize(&admin);
    client.initialize(&admin);
}


#[test]
fn test_submit_data_pass() {
    let (env, client, admin) = setup_test_environment();
    client.initialize(&admin);


    let consumer = Address::generate(&env);
    let meter_id = String::from_str(&env, "METER_001");
    let consumption_kwh = 200;
    let meter_reading = 1300;
    let temperature = Some(22);
    let voltage = Some(230);
    let record = client.submit_data(&consumer, &meter_id, &consumption_kwh, &meter_reading, &temperature, &voltage);

    let consumer_records_id = client.get_consumer_records(&consumer, &0, &5);

    let record_id = consumer_records_id.get(0).unwrap();
    assert_eq!(record_id, record);
    assert_eq!(record_id, 1);

    let consumer_records = client.get_consumption_record(&record_id);
    log!(&env, "consumer_records: {}", consumer_records);
    assert_eq!(consumer_records.consumption_kwh, consumption_kwh);
    assert_eq!(consumer_records.meter_reading, meter_reading);
    assert_eq!(consumer_records.temperature, temperature);
    assert_eq!(consumer_records.voltage, voltage);
}


#[test]
fn test_submit_data_multiple() {
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

    let len = consumers.len();
    let mut record = 0;
    for i in 0..len {
        env.ledger().set_timestamp((1 + i).into());
        let consumer = consumers.get(i).unwrap();
        let meter_id = meter_ids.get(i).unwrap();
        let _ = client.submit_data(&consumer, &meter_id, &consumption_kwh, &meter_reading, &temperature, &voltage);
        record += 1;
    }
    assert_eq!(record, 10);
}


#[test]
fn test_submit_data_multiple_same_consumer() {
    let (env, client, admin) = setup_test_environment();
    client.initialize(&admin);

    let consumer = Address::generate(&env);
    let meter_id = String::from_str(&env, "METER_001");
    let consumption_kwh = 100;
    let meter_reading = 200;
    let temperature = Some(22);
    let voltage = Some(230);

    let mut record = 0;
    for i in 1..6 {
        env.ledger().set_timestamp((1 + i).into());
        let _ = client.submit_data(&consumer, &meter_id, &(consumption_kwh * i), &(meter_reading * i), &temperature, &voltage);
        record += 1;
    }
    assert_eq!(record, 5);

    let consumer_records_ids = client.get_consumer_records(&consumer, &0, &10);
    assert_eq!(consumer_records_ids.len(), 5);

    log!(&env, "consumer record: {}", consumer_records_ids);

    for (i, record_id) in consumer_records_ids.iter().enumerate() {
        let consumer_records = client.get_consumption_record(&record_id);
        log!(&env, "consumer records: {}", consumer_records);

        assert_eq!(consumer_records.consumption_kwh, (consumption_kwh * (i+1) as u64));
        assert_eq!(consumer_records.meter_reading, meter_reading * (i+1) as u64);
        assert_eq!(consumer_records.temperature, temperature);
        assert_eq!(consumer_records.voltage, voltage);
    }
}

#[test]
#[should_panic(expected = "Error(Contract, #9)")]
fn test_submit_data_not_initiliazed() {
    let (env, client, _) = setup_test_environment();

    let consumer = Address::generate(&env);
    let meter_id = String::from_str(&env, "METER_001");
    let consumption_kwh = 200;
    let meter_reading = 1300;
    let temperature = Some(22);
    let voltage = Some(230);
    let _ = client.submit_data(&consumer, &meter_id, &consumption_kwh, &meter_reading, &temperature, &voltage);
}
