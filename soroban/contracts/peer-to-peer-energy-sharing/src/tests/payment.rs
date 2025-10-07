#![cfg(test)]

use crate::tests::utils::*;
use crate::utils::*;
use soroban_sdk::testutils::Address as _;

#[test]
fn test_successful_payment_settlement() {
    let setup = TestSetup::new();
    let agreement_id = setup.create_test_agreement().unwrap();
    
    // Deliver energy first
    let transaction_id = setup.client.deliver_energy(
        &agreement_id,
        &100u64,
        &1500u64,
        &setup.provider,
    ).unwrap();
    
    // Check initial balances
    let consumer_balance_before = get_token_balance(&setup.env, &setup.token_contract, &setup.consumer);
    let provider_balance_before = get_token_balance(&setup.env, &setup.token_contract, &setup.provider);
    
    // Settle payment
    setup.client.settle_payment(&transaction_id, &setup.consumer).unwrap();
    
    // Check balances after payment
    let consumer_balance_after = get_token_balance(&setup.env, &setup.token_contract, &setup.consumer);
    let provider_balance_after = get_token_balance(&setup.env, &setup.token_contract, &setup.provider);
    
    // Verify payment was transferred
    assert_eq!(consumer_balance_after, consumer_balance_before - 5000); // 100 * 50
    assert_eq!(provider_balance_after, provider_balance_before + 5000);
    
    // Check transaction status updated
    let transaction = setup.client.get_transaction(&transaction_id).unwrap();
    assert_eq!(transaction.status, TransactionStatus::Settled);
    assert!(transaction.settled_at.is_some());
}

#[test]
fn test_provider_can_settle_payment() {
    let setup = TestSetup::new();
    let agreement_id = setup.create_test_agreement().unwrap();
    
    let transaction_id = setup.client.deliver_energy(
        &agreement_id,
        &100u64,
        &1500u64,
        &setup.provider,
    ).unwrap();
    
    // Provider settles the payment
    setup.client.settle_payment(&transaction_id, &setup.provider).unwrap();
    
    let transaction = setup.client.get_transaction(&transaction_id).unwrap();
    assert_eq!(transaction.status, TransactionStatus::Settled);
}

#[test]
fn test_settle_payment_unauthorized() {
    let setup = TestSetup::new();
    let agreement_id = setup.create_test_agreement().unwrap();
    
    let transaction_id = setup.client.deliver_energy(
        &agreement_id,
        &100u64,
        &1500u64,
        &setup.provider,
    ).unwrap();
    
    // Unauthorized third party tries to settle
    let result = setup.client.settle_payment(&transaction_id, &setup.prosumer3);
    assert_eq!(result.unwrap_err(), SharingError::NotAuthorized);
}

#[test]
fn test_settle_nonexistent_transaction() {
    let setup = TestSetup::new();
    
    let result = setup.client.settle_payment(&999, &setup.consumer);
    assert_eq!(result.unwrap_err(), SharingError::TransactionNotFound);
}

#[test]
fn test_settle_already_settled_transaction() {
    let setup = TestSetup::new();
    let agreement_id = setup.create_test_agreement().unwrap();
    
    let transaction_id = setup.client.deliver_energy(
        &agreement_id,
        &100u64,
        &1500u64,
        &setup.provider,
    ).unwrap();
    
    // First settlement
    setup.client.settle_payment(&transaction_id, &setup.consumer).unwrap();
    
    // Try to settle again
    let result = setup.client.settle_payment(&transaction_id, &setup.consumer);
    assert_eq!(result.unwrap_err(), SharingError::TransactionAlreadySettled);
}

#[test]
fn test_settle_payment_for_pending_transaction() {
    let setup = TestSetup::new();
    
    // Create a transaction ID that doesn't exist (simulating pending state)
    let result = setup.client.settle_payment(&1, &setup.consumer);
    assert_eq!(result.unwrap_err(), SharingError::TransactionNotFound);
}

#[test]
fn test_payment_settlement_updates_agreement_status() {
    let setup = TestSetup::new();
    let agreement_id = setup.create_test_agreement().unwrap();
    
    let transaction_id = setup.client.deliver_energy(
        &agreement_id,
        &100u64,
        &1500u64,
        &setup.provider,
    ).unwrap();
    
    // Check agreement status before settlement
    let agreement = setup.client.get_agreement(&agreement_id).unwrap();
    assert_eq!(agreement.status, AgreementStatus::Delivered);
    
    // Settle payment
    setup.client.settle_payment(&transaction_id, &setup.consumer).unwrap();
    
    // Check agreement status after settlement
    let agreement = setup.client.get_agreement(&agreement_id).unwrap();
    assert_eq!(agreement.status, AgreementStatus::Settled);
}

#[test]
fn test_partial_delivery_payment() {
    let setup = TestSetup::new();
    let agreement_id = setup.create_test_agreement().unwrap(); // 100 kWh at 50 per kWh
    
    // Partial delivery
    let transaction_id = setup.client.deliver_energy(
        &agreement_id,
        &75u64, // Only 75 kWh delivered
        &1125u64,
        &setup.provider,
    ).unwrap();
    
    let consumer_balance_before = get_token_balance(&setup.env, &setup.token_contract, &setup.consumer);
    let provider_balance_before = get_token_balance(&setup.env, &setup.token_contract, &setup.provider);
    
    setup.client.settle_payment(&transaction_id, &setup.consumer).unwrap();
    
    let consumer_balance_after = get_token_balance(&setup.env, &setup.token_contract, &setup.consumer);
    let provider_balance_after = get_token_balance(&setup.env, &setup.token_contract, &setup.provider);
    
    // Should only pay for delivered amount: 75 * 50 = 3750
    assert_eq!(consumer_balance_after, consumer_balance_before - 3750);
    assert_eq!(provider_balance_after, provider_balance_before + 3750);
}

#[test]
fn test_zero_energy_delivery_payment() {
    let setup = TestSetup::new();
    let agreement_id = setup.create_test_agreement().unwrap();
    
    let transaction_id = setup.client.deliver_energy(
        &agreement_id,
        &0u64, // Zero energy delivered
        &0u64,
        &setup.provider,
    ).unwrap();
    
    let consumer_balance_before = get_token_balance(&setup.env, &setup.token_contract, &setup.consumer);
    let provider_balance_before = get_token_balance(&setup.env, &setup.token_contract, &setup.provider);
    
    setup.client.settle_payment(&transaction_id, &setup.consumer).unwrap();
    
    let consumer_balance_after = get_token_balance(&setup.env, &setup.token_contract, &setup.consumer);
    let provider_balance_after = get_token_balance(&setup.env, &setup.token_contract, &setup.provider);
    
    // No payment should occur for zero delivery
    assert_eq!(consumer_balance_after, consumer_balance_before);
    assert_eq!(provider_balance_after, provider_balance_before);
}

#[test]
fn test_payment_settlement_timestamp() {
    let setup = TestSetup::new();
    let agreement_id = setup.create_test_agreement().unwrap();
    
    let transaction_id = setup.client.deliver_energy(
        &agreement_id,
        &100u64,
        &1500u64,
        &setup.provider,
    ).unwrap();
    
    let settlement_time = setup.env.ledger().timestamp();
    setup.client.settle_payment(&transaction_id, &setup.consumer).unwrap();
    
    let transaction = setup.client.get_transaction(&transaction_id).unwrap();
    assert_eq!(transaction.settled_at, Some(settlement_time));
}

#[test]
fn test_multiple_payment_settlements() {
    let setup = TestSetup::new();
    
    // Create two agreements
    let agreement_id1 = setup.create_test_agreement().unwrap();
    let agreement_id2 = setup.create_custom_agreement(
        &setup.prosumer3,
        &setup.consumer,
        200,
        30,
        setup.get_future_deadline(),
    ).unwrap();
    
    // Deliver energy for both
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
    
    let consumer_balance_before = get_token_balance(&setup.env, &setup.token_contract, &setup.consumer);
    let provider_balance_before = get_token_balance(&setup.env, &setup.token_contract, &setup.provider);
    let prosumer3_balance_before = get_token_balance(&setup.env, &setup.token_contract, &setup.prosumer3);
    
    // Settle both payments
    setup.client.settle_payment(&transaction_id1, &setup.consumer).unwrap();
    setup.client.settle_payment(&transaction_id2, &setup.consumer).unwrap();
    
    let consumer_balance_after = get_token_balance(&setup.env, &setup.token_contract, &setup.consumer);
    let provider_balance_after = get_token_balance(&setup.env, &setup.token_contract, &setup.provider);
    let prosumer3_balance_after = get_token_balance(&setup.env, &setup.token_contract, &setup.prosumer3);
    
    // Check all balances updated correctly
    assert_eq!(consumer_balance_after, consumer_balance_before - 5000 - 6000); // 5000 + 6000
    assert_eq!(provider_balance_after, provider_balance_before + 5000);
    assert_eq!(prosumer3_balance_after, prosumer3_balance_before + 6000);
}

#[test]
fn test_get_transaction_history_empty() {
    let setup = TestSetup::new();
    
    let history = setup.client.get_transaction_history(&setup.provider).unwrap();
    assert_eq!(history.len(), 0);
}

#[test]
fn test_get_transaction_history_provider() {
    let setup = TestSetup::new();
    let agreement_id = setup.create_test_agreement().unwrap();
    
    let transaction_id = setup.client.deliver_energy(
        &agreement_id,
        &100u64,
        &1500u64,
        &setup.provider,
    ).unwrap();
    
    let history = setup.client.get_transaction_history(&setup.provider).unwrap();
    assert_eq!(history.len(), 1);
    assert_eq!(history.get(0).unwrap().transaction_id, transaction_id);
}

#[test]
fn test_get_transaction_history_consumer() {
    let setup = TestSetup::new();
    let agreement_id = setup.create_test_agreement().unwrap();
    
    let transaction_id = setup.client.deliver_energy(
        &agreement_id,
        &100u64,
        &1500u64,
        &setup.provider,
    ).unwrap();
    
    let history = setup.client.get_transaction_history(&setup.consumer).unwrap();
    assert_eq!(history.len(), 1);
    assert_eq!(history.get(0).unwrap().transaction_id, transaction_id);
}

#[test]
fn test_get_transaction_history_multiple_transactions() {
    let setup = TestSetup::new();
    
    // Create multiple agreements and transactions
    let agreement_id1 = setup.create_test_agreement().unwrap();
    let agreement_id2 = setup.create_custom_agreement(
        &setup.provider,
        &setup.prosumer3,
        150,
        40,
        setup.get_future_deadline(),
    ).unwrap();
    
    setup.client.deliver_energy(
        &agreement_id1,
        &100u64,
        &1500u64,
        &setup.provider,
    ).unwrap();
    
    setup.client.deliver_energy(
        &agreement_id2,
        &150u64,
        &2250u64,
        &setup.provider,
    ).unwrap();
    
    let history = setup.client.get_transaction_history(&setup.provider).unwrap();
    assert_eq!(history.len(), 2);
    
    // Consumer should only see their transaction
    let consumer_history = setup.client.get_transaction_history(&setup.consumer).unwrap();
    assert_eq!(consumer_history.len(), 1);
    
    // Prosumer3 should only see their transaction
    let prosumer3_history = setup.client.get_transaction_history(&setup.prosumer3).unwrap();
    assert_eq!(prosumer3_history.len(), 1);
}

#[test]
fn test_payment_accuracy_with_different_prices() {
    let setup = TestSetup::new();
    let agreement_id = setup.create_custom_agreement(
        &setup.provider,
        &setup.consumer,
        250,
        120, // Higher price per kWh
        setup.get_future_deadline(),
    ).unwrap();
    
    let transaction_id = setup.client.deliver_energy(
        &agreement_id,
        &200u64, // 200 out of 250 kWh
        &3000u64,
        &setup.provider,
    ).unwrap();
    
    let consumer_balance_before = get_token_balance(&setup.env, &setup.token_contract, &setup.consumer);
    let provider_balance_before = get_token_balance(&setup.env, &setup.token_contract, &setup.provider);
    
    setup.client.settle_payment(&transaction_id, &setup.consumer).unwrap();
    
    let consumer_balance_after = get_token_balance(&setup.env, &setup.token_contract, &setup.consumer);
    let provider_balance_after = get_token_balance(&setup.env, &setup.token_contract, &setup.provider);
    
    // Payment should be: 200 * 120 = 24000
    assert_eq!(consumer_balance_after, consumer_balance_before - 24000);
    assert_eq!(provider_balance_after, provider_balance_before + 24000);
}