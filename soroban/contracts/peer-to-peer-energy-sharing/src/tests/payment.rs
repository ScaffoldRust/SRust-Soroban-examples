#![cfg(test)]

use crate::tests::utils::*;
use crate::utils::*;

#[test]
fn test_successful_payment_settlement() {
    let setup = TestSetup::new();
    let agreement_id = setup.create_test_agreement();

    // Deliver energy
    let transaction_id =
        setup
            .client
            .deliver_energy(&agreement_id, &100u64, &1500u64, &setup.provider);

    // Check balances before settlement
    let consumer_balance_before =
        get_token_balance(&setup.env, &setup.token_contract, &setup.consumer);
    let provider_balance_before =
        get_token_balance(&setup.env, &setup.token_contract, &setup.provider);

    // Settle payment
    setup
        .client
        .settle_payment(&transaction_id, &setup.consumer);

    // Verify payment transferred correctly
    let consumer_balance_after =
        get_token_balance(&setup.env, &setup.token_contract, &setup.consumer);
    let provider_balance_after =
        get_token_balance(&setup.env, &setup.token_contract, &setup.provider);

    assert_eq!(consumer_balance_after, consumer_balance_before - 5000);
    assert_eq!(provider_balance_after, provider_balance_before + 5000);

    // Verify transaction and agreement status updated
    let transaction = setup.client.get_transaction(&transaction_id);
    assert_eq!(transaction.status, TransactionStatus::Settled);
    assert!(transaction.settled_at.is_some());

    let agreement = setup.client.get_agreement(&agreement_id);
    assert_eq!(agreement.status, AgreementStatus::Settled);
}

#[test]
fn test_payment_authorization_failures() {
    let setup = TestSetup::new();
    let agreement_id = setup.create_test_agreement();

    let transaction_id =
        setup
            .client
            .deliver_energy(&agreement_id, &100u64, &1500u64, &setup.provider);

    // Unauthorized settlement attempt
    let result = setup
        .client
        .try_settle_payment(&transaction_id, &setup.prosumer3);
    assert_eq!(result, Err(Ok(SharingError::NotAuthorized)));

    // Non-existent transaction
    let result = setup.client.try_settle_payment(&999, &setup.consumer);
    assert_eq!(result, Err(Ok(SharingError::TransactionNotFound)));

    // Settle successfully first time
    setup
        .client
        .settle_payment(&transaction_id, &setup.consumer);

    // Duplicate settlement attempt
    let result = setup
        .client
        .try_settle_payment(&transaction_id, &setup.consumer);
    assert_eq!(result, Err(Ok(SharingError::TransactionAlreadySettled)));
}

#[test]
fn test_payment_accuracy_partial_delivery() {
    let setup = TestSetup::new();
    let agreement_id = setup.create_custom_agreement(
        &setup.provider,
        &setup.consumer,
        250,
        120,
        setup.get_future_deadline(),
    );

    // Partial delivery: 200 out of 250 kWh at 120 per kWh
    let transaction_id =
        setup
            .client
            .deliver_energy(&agreement_id, &200u64, &3000u64, &setup.provider);

    let consumer_balance_before =
        get_token_balance(&setup.env, &setup.token_contract, &setup.consumer);
    let provider_balance_before =
        get_token_balance(&setup.env, &setup.token_contract, &setup.provider);

    setup
        .client
        .settle_payment(&transaction_id, &setup.provider);

    let consumer_balance_after =
        get_token_balance(&setup.env, &setup.token_contract, &setup.consumer);
    let provider_balance_after =
        get_token_balance(&setup.env, &setup.token_contract, &setup.provider);

    // Should pay exactly: 200 * 120 = 24000
    assert_eq!(consumer_balance_after, consumer_balance_before - 24000);
    assert_eq!(provider_balance_after, provider_balance_before + 24000);
}

#[test]
fn test_transaction_history_tracking() {
    let setup = TestSetup::new();
    let agreement_id = setup.create_test_agreement();

    // Initially empty
    let history = setup.client.get_transaction_history(&setup.provider);
    assert_eq!(history.len(), 0);

    // Create transaction
    let transaction_id =
        setup
            .client
            .deliver_energy(&agreement_id, &100u64, &1500u64, &setup.provider);

    // Verify transaction appears in both provider and consumer history
    let provider_history = setup.client.get_transaction_history(&setup.provider);
    let consumer_history = setup.client.get_transaction_history(&setup.consumer);

    assert_eq!(provider_history.len(), 1);
    assert_eq!(consumer_history.len(), 1);
    assert_eq!(
        provider_history.get(0).unwrap().transaction_id,
        transaction_id
    );
    assert_eq!(
        consumer_history.get(0).unwrap().transaction_id,
        transaction_id
    );
}
