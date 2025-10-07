#![cfg(test)]

use crate::tests::utils::*;
use crate::utils::*;
use crate::{PeerToPeerEnergySharing, PeerToPeerEnergySharingClient};
use soroban_sdk::{testutils::Address as _, Address, Env};

#[test]
fn test_valid_agreement_creation() {
    let setup = TestSetup::new();
    
    let agreement_id = setup.create_test_agreement();
    let agreement = setup.client.get_agreement(&agreement_id);
    
    // Verify data integrity
    assert_eq!(agreement.provider, setup.provider);
    assert_eq!(agreement.consumer, setup.consumer);
    assert_eq!(agreement.energy_amount_kwh, 100);
    assert_eq!(agreement.price_per_kwh, 50);
    assert_eq!(agreement.total_amount, 5000);
    assert_eq!(agreement.status, AgreementStatus::Active);
}

#[test]
fn test_agreement_invalid_inputs() {
    let setup = TestSetup::new();
    
    // Zero energy amount
    let result = setup.client.try_create_agreement(
        &setup.provider, &setup.consumer, &0u64, &50u64, &setup.get_future_deadline()
    );
    assert_eq!(result, Err(Ok(SharingError::InvalidInput)));
    
    // Zero price
    let result = setup.client.try_create_agreement(
        &setup.provider, &setup.consumer, &100u64, &0u64, &setup.get_future_deadline()
    );
    assert_eq!(result, Err(Ok(SharingError::InvalidInput)));
    
    // Invalid deadline
    let result = setup.client.try_create_agreement(
        &setup.provider, &setup.consumer, &100u64, &50u64, &0u64
    );
    assert_eq!(result, Err(Ok(SharingError::InvalidInput)));
}

#[test]
fn test_agreement_authorization_failures() {
    let setup = TestSetup::new();
    let unregistered = Address::generate(&setup.env);
    
    // Unregistered provider
    let result = setup.client.try_create_agreement(
        &unregistered, &setup.consumer, &100u64, &50u64, &setup.get_future_deadline()
    );
    assert_eq!(result, Err(Ok(SharingError::ProsumerNotRegistered)));
    
    // Self-sharing not allowed
    let result = setup.client.try_create_agreement(
        &setup.provider, &setup.provider, &100u64, &50u64, &setup.get_future_deadline()
    );
    assert_eq!(result, Err(Ok(SharingError::SelfSharingNotAllowed)));
    
    // Contract not initialized
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(PeerToPeerEnergySharing, ());
    let client = PeerToPeerEnergySharingClient::new(&env, &contract_id);
    
    let result = client.try_create_agreement(
        &Address::generate(&env), &Address::generate(&env), 
        &100u64, &50u64, &(env.ledger().timestamp() + 86400)
    );
    assert_eq!(result, Err(Ok(SharingError::NotInitialized)));
}