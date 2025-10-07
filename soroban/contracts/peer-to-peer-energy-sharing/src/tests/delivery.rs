#![cfg(test)]

use crate::tests::utils::*;
use crate::utils::*;

#[test]
fn test_valid_energy_delivery() {
    let setup = TestSetup::new();
    let agreement_id = setup.create_test_agreement();

    let transaction_id =
        setup
            .client
            .deliver_energy(&agreement_id, &100u64, &1500u64, &setup.provider);

    let transaction = setup.client.get_transaction(&transaction_id);
    assert_eq!(transaction.energy_delivered_kwh, 100);
    assert_eq!(transaction.meter_reading, 1500);
    assert_eq!(transaction.payment_amount, 5000);
    assert_eq!(transaction.status, TransactionStatus::Delivered);

    // Verify agreement status updated
    let agreement = setup.client.get_agreement(&agreement_id);
    assert_eq!(agreement.status, AgreementStatus::Delivered);
}

#[test]
fn test_delivery_validation_failures() {
    let setup = TestSetup::new();
    let agreement_id = setup.create_test_agreement();

    // Unauthorized provider
    let result = setup
        .client
        .try_deliver_energy(&agreement_id, &100u64, &1500u64, &setup.consumer);
    assert_eq!(result, Err(Ok(SharingError::NotAuthorized)));

    // Excessive energy delivery
    let result = setup
        .client
        .try_deliver_energy(&agreement_id, &150u64, &2250u64, &setup.provider);
    assert_eq!(result, Err(Ok(SharingError::InsufficientEnergy)));

    // Non-existent agreement
    let result = setup
        .client
        .try_deliver_energy(&999u64, &100u64, &1500u64, &setup.provider);
    assert_eq!(result, Err(Ok(SharingError::AgreementNotFound)));
}

#[test]
fn test_delivery_deadline_enforcement() {
    let setup = TestSetup::new();
    let agreement_id = setup.create_test_agreement();

    // Advance time past deadline
    setup.advance_ledger_time(90000);

    let result = setup
        .client
        .try_deliver_energy(&agreement_id, &100u64, &1500u64, &setup.provider);
    assert_eq!(result, Err(Ok(SharingError::DeliveryDeadlinePassed)));
}

#[test]
fn test_partial_delivery_with_meter_verification() {
    let setup = TestSetup::new();
    let agreement_id = setup.create_test_agreement();

    // Partial delivery with specific meter reading
    let meter_reading = 2750u64;
    let transaction_id =
        setup
            .client
            .deliver_energy(&agreement_id, &75u64, &meter_reading, &setup.provider);

    let transaction = setup.client.get_transaction(&transaction_id);
    assert_eq!(transaction.energy_delivered_kwh, 75);
    assert_eq!(transaction.meter_reading, meter_reading);
    assert_eq!(transaction.payment_amount, 3750); // 75 * 50
}
