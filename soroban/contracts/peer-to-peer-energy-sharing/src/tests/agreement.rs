#![cfg(test)]

use crate::tests::utils::*;
use crate::utils::*;
use soroban_sdk::testutils::Address as _;

#[test]
fn test_create_valid_agreement() {
    let setup = TestSetup::new();
    
    let agreement_id = setup.create_test_agreement().unwrap();
    assert_eq!(agreement_id, 1);
    
    let agreement = setup.client.get_agreement(&agreement_id).unwrap();
    assert_eq!(agreement.provider, setup.provider);
    assert_eq!(agreement.consumer, setup.consumer);
    assert_eq!(agreement.energy_amount_kwh, 100);
    assert_eq!(agreement.price_per_kwh, 50);
    assert_eq!(agreement.total_amount, 5000); // 100 * 50
    assert_eq!(agreement.status, AgreementStatus::Active);
}

#[test]
fn test_create_agreement_with_zero_energy() {
    let setup = TestSetup::new();
    
    let result = setup.create_custom_agreement(
        &setup.provider,
        &setup.consumer,
        0, // Zero energy
        50,
        setup.get_future_deadline(),
    );
    
    assert_eq!(result.unwrap_err(), SharingError::InvalidInput);
}

#[test]
fn test_create_agreement_with_zero_price() {
    let setup = TestSetup::new();
    
    let result = setup.create_custom_agreement(
        &setup.provider,
        &setup.consumer,
        100,
        0, // Zero price
        setup.get_future_deadline(),
    );
    
    assert_eq!(result.unwrap_err(), SharingError::InvalidInput);
}

#[test]
fn test_create_agreement_with_past_deadline() {
    let setup = TestSetup::new();
    
    let past_deadline = setup.env.ledger().timestamp() - 3600; // 1 hour ago
    
    let result = setup.create_custom_agreement(
        &setup.provider,
        &setup.consumer,
        100,
        50,
        past_deadline,
    );
    
    assert_eq!(result.unwrap_err(), SharingError::DeliveryDeadlinePassed);
}

#[test]
fn test_self_sharing_not_allowed() {
    let setup = TestSetup::new();
    
    let result = setup.create_custom_agreement(
        &setup.provider,
        &setup.provider, // Same as provider
        100,
        50,
        setup.get_future_deadline(),
    );
    
    assert_eq!(result.unwrap_err(), SharingError::SelfSharingNotAllowed);
}

#[test]
fn test_unregistered_prosumer_provider() {
    let setup = TestSetup::new();
    let unregistered = setup.env.generate::<Address>();
    
    let result = setup.create_custom_agreement(
        &unregistered,
        &setup.consumer,
        100,
        50,
        setup.get_future_deadline(),
    );
    
    assert_eq!(result.unwrap_err(), SharingError::ProsumerNotRegistered);
}

#[test]
fn test_unregistered_prosumer_consumer() {
    let setup = TestSetup::new();
    let unregistered = setup.env.generate::<Address>();
    
    let result = setup.create_custom_agreement(
        &setup.provider,
        &unregistered,
        100,
        50,
        setup.get_future_deadline(),
    );
    
    assert_eq!(result.unwrap_err(), SharingError::ProsumerNotRegistered);
}

#[test]
fn test_multiple_agreements_increment_ids() {
    let setup = TestSetup::new();
    
    let id1 = setup.create_test_agreement().unwrap();
    let id2 = setup.create_custom_agreement(
        &setup.prosumer3,
        &setup.consumer,
        200,
        30,
        setup.get_future_deadline(),
    ).unwrap();
    let id3 = setup.create_custom_agreement(
        &setup.provider,
        &setup.prosumer3,
        50,
        100,
        setup.get_future_deadline(),
    ).unwrap();
    
    assert_eq!(id1, 1);
    assert_eq!(id2, 2);
    assert_eq!(id3, 3);
}

#[test]
fn test_agreement_total_amount_calculation() {
    let setup = TestSetup::new();
    
    let agreement_id = setup.create_custom_agreement(
        &setup.provider,
        &setup.consumer,
        250, // energy_amount_kwh
        75,  // price_per_kwh
        setup.get_future_deadline(),
    ).unwrap();
    
    let agreement = setup.client.get_agreement(&agreement_id).unwrap();
    assert_eq!(agreement.total_amount, 18750); // 250 * 75
}

#[test]
fn test_agreement_created_timestamp() {
    let setup = TestSetup::new();
    
    let current_time = setup.env.ledger().timestamp();
    let agreement_id = setup.create_test_agreement().unwrap();
    
    let agreement = setup.client.get_agreement(&agreement_id).unwrap();
    assert_eq!(agreement.created_at, current_time);
}

#[test]
fn test_get_nonexistent_agreement() {
    let setup = TestSetup::new();
    
    let result = setup.client.get_agreement(&999);
    assert_eq!(result.unwrap_err(), SharingError::AgreementNotFound);
}

#[test]
fn test_agreement_data_integrity() {
    let setup = TestSetup::new();
    
    let energy_amount = 150u64;
    let price = 80u64;
    let deadline = setup.get_future_deadline();
    
    let agreement_id = setup.create_custom_agreement(
        &setup.provider,
        &setup.consumer,
        energy_amount,
        price,
        deadline,
    ).unwrap();
    
    let agreement = setup.client.get_agreement(&agreement_id).unwrap();
    
    // Verify all data integrity
    assert_eq!(agreement.agreement_id, agreement_id);
    assert_eq!(agreement.provider, setup.provider);
    assert_eq!(agreement.consumer, setup.consumer);
    assert_eq!(agreement.energy_amount_kwh, energy_amount);
    assert_eq!(agreement.price_per_kwh, price);
    assert_eq!(agreement.total_amount, energy_amount * price);
    assert_eq!(agreement.delivery_deadline, deadline);
    assert_eq!(agreement.status, AgreementStatus::Active);
}

#[test]
fn test_large_energy_amounts() {
    let setup = TestSetup::new();
    
    let large_energy = 1_000_000u64;
    let price = 100u64;
    
    let agreement_id = setup.create_custom_agreement(
        &setup.provider,
        &setup.consumer,
        large_energy,
        price,
        setup.get_future_deadline(),
    ).unwrap();
    
    let agreement = setup.client.get_agreement(&agreement_id).unwrap();
    assert_eq!(agreement.energy_amount_kwh, large_energy);
    assert_eq!(agreement.total_amount, large_energy * price);
}

#[test]
fn test_contract_not_initialized() {
    let env = Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register(PeerToPeerEnergySharing, ());
    let client = PeerToPeerEnergySharingClient::new(&env, &contract_id);
    
    let provider = env.generate::<Address>();
    let consumer = env.generate::<Address>();
    
    // Try to create agreement without initialization
    let result = client.create_agreement(
        &provider,
        &consumer,
        &100u64,
        &50u64,
        &(env.ledger().timestamp() + 86400),
    );
    
    assert_eq!(result.unwrap_err(), SharingError::NotInitialized);
}