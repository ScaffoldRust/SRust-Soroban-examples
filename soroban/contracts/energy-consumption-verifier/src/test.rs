use super::*;
use soroban_sdk::{Address, Env, String as SorobanString};
use soroban_sdk::testutils::{Address as _, Ledger as _};

#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register(EnergyConsumptionVerifier, ());
    let client = EnergyConsumptionVerifierClient::new(&env, &contract_id);

    client.initialize(&admin);

    let initialized: bool = env.as_contract(&contract_id, || {
        env.storage().instance().get(&DataKey::Initialized).unwrap()
    });
    assert!(initialized);

    let stored_admin: Address = env.as_contract(&contract_id, || {
        env.storage().instance().get(&DataKey::Admin).unwrap()
    });
    assert_eq!(stored_admin, admin);
}

#[test]
fn test_initialize_twice_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register(EnergyConsumptionVerifier, ());
    let client = EnergyConsumptionVerifierClient::new(&env, &contract_id);

    client.initialize(&admin);

    let result2 = client.try_initialize(&admin);
    assert_eq!(result2, Err(Ok(ContractError::AlreadyInitialized)));
}

#[test]
fn test_submit_data() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let consumer = Address::generate(&env);
    let contract_id = env.register(EnergyConsumptionVerifier, ());
    let client = EnergyConsumptionVerifierClient::new(&env, &contract_id);

    client.initialize(&admin);

    let meter_id = SorobanString::from_str(&env, "METER_001");
    let record_id = client.submit_data(&consumer, &meter_id, &100, &12345, &Some(25), &Some(230));

    assert!(record_id > 0);
}

#[test]
fn test_submit_data_invalid_consumption() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let consumer = Address::generate(&env);
    let contract_id = env.register(EnergyConsumptionVerifier, ());
    let client = EnergyConsumptionVerifierClient::new(&env, &contract_id);

    client.initialize(&admin);

    let meter_id = SorobanString::from_str(&env, "METER_001");
    let result = client.try_submit_data(&consumer, &meter_id, &200000, &12345, &Some(25), &Some(230));

    assert_eq!(result, Err(Ok(ContractError::InvalidMeterData)));
}

#[test]
fn test_register_verifier() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let verifier = Address::generate(&env);
    let contract_id = env.register(EnergyConsumptionVerifier, ());
    let client = EnergyConsumptionVerifierClient::new(&env, &contract_id);

    client.initialize(&admin);

    client.register_verifier(&admin, &verifier);
}

#[test]
fn test_register_verifier_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let unauthorized = Address::generate(&env);
    let verifier = Address::generate(&env);
    let contract_id = env.register(EnergyConsumptionVerifier, ());
    let client = EnergyConsumptionVerifierClient::new(&env, &contract_id);

    client.initialize(&admin);

    let result = client.try_register_verifier(&unauthorized, &verifier);
    assert_eq!(result, Err(Ok(ContractError::NotAuthorized)));
}

#[test]
fn test_verify_data() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let consumer = Address::generate(&env);
    let verifier = Address::generate(&env);
    let contract_id = env.register(EnergyConsumptionVerifier, ());
    let client = EnergyConsumptionVerifierClient::new(&env, &contract_id);

    client.initialize(&admin);
    client.register_verifier(&admin, &verifier);

    let meter_id = SorobanString::from_str(&env, "METER_001");
    let record_id = client.submit_data(&consumer, &meter_id, &100, &12345, &Some(25), &Some(230));

    let comments = Some(SorobanString::from_str(&env, "Data verified"));
    client.verify_data(&verifier, &record_id, &VerificationStatus::Verified, &comments);
}

#[test]
fn test_verify_data_unauthorized_verifier() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let consumer = Address::generate(&env);
    let unauthorized_verifier = Address::generate(&env);
    let contract_id = env.register(EnergyConsumptionVerifier, ());
    let client = EnergyConsumptionVerifierClient::new(&env, &contract_id);

    client.initialize(&admin);

    let meter_id = SorobanString::from_str(&env, "METER_001");
    let record_id = client.submit_data(&consumer, &meter_id, &100, &12345, &Some(25), &Some(230));

    let comments = Some(SorobanString::from_str(&env, "Data verified"));
    let result = client.try_verify_data(&unauthorized_verifier, &record_id, &VerificationStatus::Verified, &comments);

    assert_eq!(result, Err(Ok(ContractError::VerifierNotRegistered)));
}

#[test]
fn test_get_verification() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let consumer = Address::generate(&env);
    let verifier = Address::generate(&env);
    let contract_id = env.register(EnergyConsumptionVerifier, ());
    let client = EnergyConsumptionVerifierClient::new(&env, &contract_id);

    client.initialize(&admin);
    client.register_verifier(&admin, &verifier);

    let meter_id = SorobanString::from_str(&env, "METER_001");
    let record_id = client.submit_data(&consumer, &meter_id, &100, &12345, &Some(25), &Some(230));

    let comments = Some(SorobanString::from_str(&env, "Data verified"));
    client.verify_data(&verifier, &record_id, &VerificationStatus::Verified, &comments);

    let verification_record = client.get_verification(&record_id);
    assert_eq!(verification_record.record_id, record_id);
    assert_eq!(verification_record.verifier, verifier);
    assert_eq!(verification_record.status, VerificationStatus::Verified);
}

#[test]
fn test_get_consumption_record() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let consumer = Address::generate(&env);
    let contract_id = env.register(EnergyConsumptionVerifier, ());
    let client = EnergyConsumptionVerifierClient::new(&env, &contract_id);

    client.initialize(&admin);

    let meter_id = SorobanString::from_str(&env, "METER_001");
    let record_id = client.submit_data(&consumer, &meter_id, &100, &12345, &Some(25), &Some(230));

    let consumption_record = client.get_consumption_record(&record_id);
    assert_eq!(consumption_record.consumer, consumer);
    assert_eq!(consumption_record.meter_id, meter_id);
    assert_eq!(consumption_record.consumption_kwh, 100);
    assert_eq!(consumption_record.meter_reading, 12345);
    assert_eq!(consumption_record.temperature, Some(25));
    assert_eq!(consumption_record.voltage, Some(230));
}

#[test]
fn test_register_meter() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let meter_address = Address::generate(&env);
    let contract_id = env.register(EnergyConsumptionVerifier, ());
    let client = EnergyConsumptionVerifierClient::new(&env, &contract_id);

    client.initialize(&admin);

    let meter_id = SorobanString::from_str(&env, "METER_001");
    client.register_meter(&admin, &meter_id, &meter_address);
}

#[test]
fn test_audit_log() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let consumer = Address::generate(&env);
    let contract_id = env.register(EnergyConsumptionVerifier, ());
    let client = EnergyConsumptionVerifierClient::new(&env, &contract_id);

    client.initialize(&admin);

    let meter_id = SorobanString::from_str(&env, "METER_001");
    client.submit_data(&consumer, &meter_id, &100, &12345, &Some(25), &Some(230));

    let audit_log = client.audit_log(&0, &10);
    assert!(audit_log.len() > 0);
}

#[test]
fn test_get_consumer_records() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let consumer = Address::generate(&env);
    let contract_id = env.register(EnergyConsumptionVerifier, ());
    let client = EnergyConsumptionVerifierClient::new(&env, &contract_id);

    client.initialize(&admin);

    let meter_id = SorobanString::from_str(&env, "METER_001");
    client.submit_data(&consumer, &meter_id, &100, &12345, &Some(25), &Some(230));

    env.ledger().set_timestamp(3601);
    client.submit_data(&consumer, &meter_id, &150, &12400, &Some(26), &Some(235));

    let consumer_records = client.get_consumer_records(&consumer, &0, &10);
    assert_eq!(consumer_records.len(), 2);
}

#[test]
fn test_get_records_by_status() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let consumer = Address::generate(&env);
    let verifier = Address::generate(&env);
    let contract_id = env.register(EnergyConsumptionVerifier, ());
    let client = EnergyConsumptionVerifierClient::new(&env, &contract_id);

    client.initialize(&admin);
    client.register_verifier(&admin, &verifier);

    let meter_id = SorobanString::from_str(&env, "METER_001");
    let record_id = client.submit_data(&consumer, &meter_id, &100, &12345, &Some(25), &Some(230));

    let comments = Some(SorobanString::from_str(&env, "Data verified"));
    client.verify_data(&verifier, &record_id, &VerificationStatus::Verified, &comments);

    let verified_records = client.get_records_by_status(&VerificationStatus::Verified, &0, &10);
    assert_eq!(verified_records.len(), 1);
    assert_eq!(verified_records.get(0).unwrap(), record_id);
}

#[test]
fn test_not_initialized_error() {
    let env = Env::default();
    env.mock_all_auths();

    let consumer = Address::generate(&env);
    let contract_id = env.register(EnergyConsumptionVerifier, ());
    let client = EnergyConsumptionVerifierClient::new(&env, &contract_id);

    let meter_id = SorobanString::from_str(&env, "METER_001");
    let result = client.try_submit_data(&consumer, &meter_id, &100, &12345, &Some(25), &Some(230));

    assert_eq!(result, Err(Ok(ContractError::NotInitialized)));
}