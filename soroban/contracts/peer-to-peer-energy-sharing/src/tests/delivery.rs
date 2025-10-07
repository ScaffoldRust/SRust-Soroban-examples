#![cfg(test)]

use crate::tests::utils::*;
use crate::utils::*;
use soroban_sdk::testutils::Address as _;

#[test]
fn test_successful_energy_delivery() {
    let setup = TestSetup::new();
    let agreement_id = setup.create_test_agreement().unwrap();
    
    let transaction_id = setup.client.deliver_energy(
        &agreement_id,
        &100u64,    // Full energy amount
        &1500u64,   // Meter reading
        &setup.provider,
    ).unwrap();
    
    assert_eq!(transaction_id, 1);
    
    let transaction = setup.client.get_transaction(&transaction_id).unwrap();
    assert_eq!(transaction.agreement_id, agreement_id);
    assert_eq!(transaction.provider, setup.provider);
    assert_eq!(transaction.consumer, setup.consumer);
    assert_eq!(transaction.energy_delivered_kwh, 100);
    assert_eq!(transaction.meter_reading, 1500);
    assert_eq!(transaction.payment_amount, 5000); // 100 * 50
    assert_eq!(transaction.status, TransactionStatus::Delivered);
    assert!(transaction.settled_at.is_none());
}

#[test]
fn test_partial_energy_delivery() {
    let setup = TestSetup::new();
    let agreement_id = setup.create_test_agreement().unwrap();
    
    let transaction_id = setup.client.deliver_energy(
        &agreement_id,
        &75u64,     // Partial delivery (75 out of 100 kWh)
        &1125u64,   // Meter reading
        &setup.provider,
    ).unwrap();
    
    let transaction = setup.client.get_transaction(&transaction_id).unwrap();
    assert_eq!(transaction.energy_delivered_kwh, 75);
    assert_eq!(transaction.payment_amount, 3750); // 75 * 50
}

#[test]
fn test_deliver_energy_unauthorized_provider() {
    let setup = TestSetup::new();
    let agreement_id = setup.create_test_agreement().unwrap();
    
    let result = setup.client.deliver_energy(
        &agreement_id,
        &100u64,
        &1500u64,
        &setup.consumer, // Wrong address - should be provider
    );
    
    assert_eq!(result.unwrap_err(), SharingError::NotAuthorized);
}

#[test]
fn test_deliver_energy_nonexistent_agreement() {
    let setup = TestSetup::new();
    
    let result = setup.client.deliver_energy(
        &999u64, // Non-existent agreement
        &100u64,
        &1500u64,
        &setup.provider,
    );
    
    assert_eq!(result.unwrap_err(), SharingError::AgreementNotFound);
}

#[test]
fn test_deliver_more_energy_than_agreed() {
    let setup = TestSetup::new();
    let agreement_id = setup.create_test_agreement().unwrap(); // 100 kWh agreed
    
    let result = setup.client.deliver_energy(
        &agreement_id,
        &150u64, // More than agreed amount
        &2250u64,
        &setup.provider,
    );
    
    assert_eq!(result.unwrap_err(), SharingError::InsufficientEnergy);
}

#[test]
fn test_deliver_energy_after_deadline() {
    let setup = TestSetup::new();
    let agreement_id = setup.create_test_agreement().unwrap();
    
    // Advance time beyond deadline
    setup.advance_ledger_time(90000); // More than 1 day
    
    let result = setup.client.deliver_energy(
        &agreement_id,
        &100u64,
        &1500u64,
        &setup.provider,
    );
    
    assert_eq!(result.unwrap_err(), SharingError::DeliveryDeadlinePassed);
    
    // Verify agreement status was updated to expired
    let agreement = setup.client.get_agreement(&agreement_id).unwrap();
    assert_eq!(agreement.status, AgreementStatus::Expired);
}

#[test]
fn test_deliver_energy_to_inactive_agreement() {
    let setup = TestSetup::new();
    let agreement_id = setup.create_test_agreement().unwrap();
    
    // First delivery
    setup.client.deliver_energy(
        &agreement_id,
        &100u64,
        &1500u64,
        &setup.provider,
    ).unwrap();
    
    // Try to deliver again - agreement is now in "Delivered" status
    let result = setup.client.deliver_energy(
        &agreement_id,
        &50u64,
        &750u64,
        &setup.provider,
    );
    
    assert_eq!(result.unwrap_err(), SharingError::AgreementNotActive);
}

#[test]
fn test_delivery_updates_agreement_status() {
    let setup = TestSetup::new();
    let agreement_id = setup.create_test_agreement().unwrap();
    
    // Check initial status
    let agreement = setup.client.get_agreement(&agreement_id).unwrap();
    assert_eq!(agreement.status, AgreementStatus::Active);
    
    // Deliver energy
    setup.client.deliver_energy(
        &agreement_id,
        &100u64,
        &1500u64,
        &setup.provider,
    ).unwrap();
    
    // Check updated status
    let agreement = setup.client.get_agreement(&agreement_id).unwrap();
    assert_eq!(agreement.status, AgreementStatus::Delivered);
}

#[test]
fn test_multiple_deliveries_different_agreements() {
    let setup = TestSetup::new();
    let agreement_id1 = setup.create_test_agreement().unwrap();
    let agreement_id2 = setup.create_custom_agreement(
        &setup.prosumer3,
        &setup.consumer,
        200,
        30,
        setup.get_future_deadline(),
    ).unwrap();
    
    let transaction_id1 = setup.client.deliver_energy(
        &agreement_id1,
        &100u64,
        &1500u64,
        &setup.provider,
    ).unwrap();
    
    let transaction_id2 = setup.client.deliver_energy(
        &agreement_id2,
        &200u64,
        &3000u64,
        &setup.prosumer3,
    ).unwrap();
    
    assert_eq!(transaction_id1, 1);
    assert_eq!(transaction_id2, 2);
    
    let transaction1 = setup.client.get_transaction(&transaction_id1).unwrap();
    let transaction2 = setup.client.get_transaction(&transaction_id2).unwrap();
    
    assert_eq!(transaction1.agreement_id, agreement_id1);
    assert_eq!(transaction2.agreement_id, agreement_id2);
    assert_eq!(transaction1.energy_delivered_kwh, 100);
    assert_eq!(transaction2.energy_delivered_kwh, 200);
}

#[test]
fn test_delivery_timestamp_recorded() {
    let setup = TestSetup::new();
    let agreement_id = setup.create_test_agreement().unwrap();
    
    let delivery_time = setup.env.ledger().timestamp();
    let transaction_id = setup.client.deliver_energy(
        &agreement_id,
        &100u64,
        &1500u64,
        &setup.provider,
    ).unwrap();
    
    let transaction = setup.client.get_transaction(&transaction_id).unwrap();
    assert_eq!(transaction.delivered_at, delivery_time);
}

#[test]
fn test_meter_reading_verification() {
    let setup = TestSetup::new();
    let agreement_id = setup.create_test_agreement().unwrap();
    
    let meter_reading = 2750u64;
    let transaction_id = setup.client.deliver_energy(
        &agreement_id,
        &100u64,
        &meter_reading,
        &setup.provider,
    ).unwrap();
    
    let transaction = setup.client.get_transaction(&transaction_id).unwrap();
    assert_eq!(transaction.meter_reading, meter_reading);
}

#[test]
fn test_zero_energy_delivery() {
    let setup = TestSetup::new();
    let agreement_id = setup.create_test_agreement().unwrap();
    
    let transaction_id = setup.client.deliver_energy(
        &agreement_id,
        &0u64, // Zero energy delivery
        &0u64,
        &setup.provider,
    ).unwrap();
    
    let transaction = setup.client.get_transaction(&transaction_id).unwrap();
    assert_eq!(transaction.energy_delivered_kwh, 0);
    assert_eq!(transaction.payment_amount, 0);
}

#[test]
fn test_get_nonexistent_transaction() {
    let setup = TestSetup::new();
    
    let result = setup.client.get_transaction(&999);
    assert_eq!(result.unwrap_err(), SharingError::TransactionNotFound);
}

#[test]
fn test_transaction_data_integrity() {
    let setup = TestSetup::new();
    let agreement_id = setup.create_custom_agreement(
        &setup.provider,
        &setup.consumer,
        250,
        80,
        setup.get_future_deadline(),
    ).unwrap();
    
    let energy_delivered = 200u64;
    let meter_reading = 4000u64;
    let expected_payment = energy_delivered * 80; // price_per_kwh
    
    let transaction_id = setup.client.deliver_energy(
        &agreement_id,
        &energy_delivered,
        &meter_reading,
        &setup.provider,
    ).unwrap();
    
    let transaction = setup.client.get_transaction(&transaction_id).unwrap();
    
    assert_eq!(transaction.transaction_id, transaction_id);
    assert_eq!(transaction.agreement_id, agreement_id);
    assert_eq!(transaction.provider, setup.provider);
    assert_eq!(transaction.consumer, setup.consumer);
    assert_eq!(transaction.energy_delivered_kwh, energy_delivered);
    assert_eq!(transaction.meter_reading, meter_reading);
    assert_eq!(transaction.payment_amount, expected_payment);
    assert_eq!(transaction.status, TransactionStatus::Delivered);
    assert!(transaction.settled_at.is_none());
}