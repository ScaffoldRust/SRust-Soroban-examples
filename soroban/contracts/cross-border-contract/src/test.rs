#![cfg(test)]

extern crate std;

use crate::types::*;
use crate::{CrossBorderPayment, CrossBorderPaymentClient};
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, String};

#[test]
fn test_initialize_contract() {
    let env = Env::default();
    let admin = Address::generate(&env);

    let contract_address = env.register(CrossBorderPayment, ());
    let client = CrossBorderPaymentClient::new(&env, &contract_address);

    env.mock_all_auths();
    client.initialize(&admin, &100, &100, &150);

    assert!(true, "Initialization completed without errors");
}

#[test]
fn test_initiate_transfer() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let sender = Address::generate(&env);
    let recipient = Address::generate(&env);

    let contract_address = env.register(CrossBorderPayment, ());
    let client = CrossBorderPaymentClient::new(&env, &contract_address);

    env.mock_all_auths();
    client.initialize(&admin, &100, &100, &150);

    let transfer_id = client.initiate_transfer(
        &sender,
        &recipient,
        &1000,
        &String::from_str(&env, "USD"),
        &String::from_str(&env, "SWIFT"),
    );

    let transfer = client.get_transfer_details(&transfer_id);
    assert_eq!(transfer.amount, 1000);
    assert_eq!(transfer.sender, sender);
    assert_eq!(transfer.recipient, recipient);
    assert_eq!(transfer.currency, String::from_str(&env, "USD"));
    assert_eq!(
        transfer.destination_network,
        String::from_str(&env, "SWIFT")
    );
    assert_eq!(
        client.get_transfer_status(&transfer_id),
        SettlementStatus::Pending
    );
}

#[test]
#[should_panic(expected = "Amount must be positive")]
fn test_initiate_transfer_invalid_amount() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let sender = Address::generate(&env);
    let recipient = Address::generate(&env);

    let contract_address = env.register(CrossBorderPayment, ());
    let client = CrossBorderPaymentClient::new(&env, &contract_address);

    env.mock_all_auths();
    client.initialize(&admin, &100, &100, &150);

    client.initiate_transfer(
        &sender,
        &recipient,
        &0, // Invalid amount
        &String::from_str(&env, "USD"),
        &String::from_str(&env, "SWIFT"),
    );
}

#[test]
fn test_compliance_and_settlement() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let sender = Address::generate(&env);
    let recipient = Address::generate(&env);
    let org = Address::generate(&env);

    let contract_address = env.register(CrossBorderPayment, ());
    let client = CrossBorderPaymentClient::new(&env, &contract_address);

    env.mock_all_auths();
    client.initialize(&admin, &100, &100, &150);

    // Initiate transfer
    let transfer_id = client.initiate_transfer(
        &sender,
        &recipient,
        &1000,
        &String::from_str(&env, "USD"),
        &String::from_str(&env, "SWIFT"),
    );

    // Verify compliance
    let documents = BytesN::from_array(&env, &[0; 32]);
    client.verify_compliance(&sender, &documents);
    client.verify_compliance(&recipient, &documents);

    // Execute settlement
    client.execute_settlement(&transfer_id, &org);

    let status = client.get_transfer_status(&transfer_id);
    assert_eq!(status, SettlementStatus::Settled);
}

#[test]
#[should_panic(expected = "Compliance check failed")]
fn test_settlement_without_compliance() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let sender = Address::generate(&env);
    let recipient = Address::generate(&env);
    let org = Address::generate(&env);

    let contract_address = env.register(CrossBorderPayment, ());
    let client = CrossBorderPaymentClient::new(&env, &contract_address);

    env.mock_all_auths();
    client.initialize(&admin, &100, &100, &150);

    // Initiate transfer
    let transfer_id = client.initiate_transfer(
        &sender,
        &recipient,
        &1000,
        &String::from_str(&env, "USD"),
        &String::from_str(&env, "SWIFT"),
    );

    // Attempt settlement without compliance
    client.execute_settlement(&transfer_id, &org);
}

#[test]
fn test_refund_transfer() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let sender = Address::generate(&env);
    let recipient = Address::generate(&env);
    let org = Address::generate(&env);

    let contract_address = env.register(CrossBorderPayment, ());
    let client = CrossBorderPaymentClient::new(&env, &contract_address);

    env.mock_all_auths();
    client.initialize(&admin, &100, &100, &150);

    // Initiate transfer
    let transfer_id = client.initiate_transfer(
        &sender,
        &recipient,
        &1000,
        &String::from_str(&env, "USD"),
        &String::from_str(&env, "SWIFT"),
    );

    // Refund transfer
    client.refund_transfer(&transfer_id, &org);

    let status = client.get_transfer_status(&transfer_id);
    assert_eq!(status, SettlementStatus::Refunded);
}

#[test]
#[should_panic(expected = "Transfer cannot be refunded")]
fn test_refund_settled_transfer() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let sender = Address::generate(&env);
    let recipient = Address::generate(&env);
    let org = Address::generate(&env);

    let contract_address = env.register(CrossBorderPayment, ());
    let client = CrossBorderPaymentClient::new(&env, &contract_address);

    env.mock_all_auths();
    client.initialize(&admin, &100, &100, &150);

    // Initiate transfer
    let transfer_id = client.initiate_transfer(
        &sender,
        &recipient,
        &1000,
        &String::from_str(&env, "USD"),
        &String::from_str(&env, "SWIFT"),
    );

    // Verify compliance
    let documents = BytesN::from_array(&env, &[0; 32]);
    client.verify_compliance(&sender, &documents);
    client.verify_compliance(&recipient, &documents);

    // Settle transfer
    client.execute_settlement(&transfer_id, &org);

    // Attempt to refund settled transfer
    client.refund_transfer(&transfer_id, &org);
}

#[test]
fn test_fee_calculation() {
    let env = Env::default();
    let admin = Address::generate(&env);

    let contract_address = env.register(CrossBorderPayment, ());
    let client = CrossBorderPaymentClient::new(&env, &contract_address);

    env.mock_all_auths();
    client.initialize(&admin, &100, &100, &150);

    let fee = client.calculate_fees(&10000, &false);
    assert_eq!(fee, 200); // Base fee (100) + 1% of 10000 (100)

    let urgent_fee = client.calculate_fees(&10000, &true);
    assert_eq!(urgent_fee, 300); // (100 + 100) * 1.5
}

#[test]
fn test_fx_rate() {
    let env = Env::default();
    let admin = Address::generate(&env);

    let contract_address = env.register(CrossBorderPayment, ());
    let client = CrossBorderPaymentClient::new(&env, &contract_address);

    env.mock_all_auths();
    client.initialize(&admin, &100, &100, &150);

    let source = String::from_str(&env, "USD");
    let target = String::from_str(&env, "EUR");
    client.update_fx_rate(&source, &target, &850000); // 0.85 * RATE_SCALE

    let rate = client.get_fx_rate(&source, &target);
    assert_eq!(rate, 850000);
}

#[test]
#[should_panic(expected = "Transfer not found")]
fn test_get_non_existent_transfer() {
    let env = Env::default();
    let admin = Address::generate(&env);

    let contract_address = env.register(CrossBorderPayment, ());
    let client = CrossBorderPaymentClient::new(&env, &contract_address);

    env.mock_all_auths();
    client.initialize(&admin, &100, &100, &150);

    client.get_transfer_details(&1);
}

#[test]
fn test_transfer_history() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let sender = Address::generate(&env);
    let recipient = Address::generate(&env);

    let contract_address = env.register(CrossBorderPayment, ());
    let client = CrossBorderPaymentClient::new(&env, &contract_address);

    env.mock_all_auths();
    client.initialize(&admin, &100, &100, &150);

    // Initiate two transfers
    let transfer_id1 = client.initiate_transfer(
        &sender,
        &recipient,
        &1000,
        &String::from_str(&env, "USD"),
        &String::from_str(&env, "SWIFT"),
    );
    let transfer_id2 = client.initiate_transfer(
        &sender,
        &recipient,
        &2000,
        &String::from_str(&env, "EUR"),
        &String::from_str(&env, "SEPA"),
    );

    // Note: Since get_transfer_history is not implemented, we verify transfers individually
    let transfer1 = client.get_transfer_details(&transfer_id1);
    let transfer2 = client.get_transfer_details(&transfer_id2);
    assert_eq!(transfer1.amount, 1000);
    assert_eq!(transfer2.amount, 2000);
}

#[test]
#[should_panic(expected = "Transfer is not in pending state")]
fn test_settlement_already_settled() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let sender = Address::generate(&env);
    let recipient = Address::generate(&env);
    let org = Address::generate(&env);

    let contract_address = env.register(CrossBorderPayment, ());
    let client = CrossBorderPaymentClient::new(&env, &contract_address);

    env.mock_all_auths();
    client.initialize(&admin, &100, &100, &150);

    // Initiate transfer
    let transfer_id = client.initiate_transfer(
        &sender,
        &recipient,
        &1000,
        &String::from_str(&env, "USD"),
        &String::from_str(&env, "SWIFT"),
    );

    // Verify compliance
    let documents = BytesN::from_array(&env, &[0; 32]);
    client.verify_compliance(&sender, &documents);
    client.verify_compliance(&recipient, &documents);

    // Settle transfer
    client.execute_settlement(&transfer_id, &org);

    // Attempt to settle again
    client.execute_settlement(&transfer_id, &org);
}

#[test]
fn test_compliance_verification() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    let contract_address = env.register(CrossBorderPayment, ());
    let client = CrossBorderPaymentClient::new(&env, &contract_address);

    env.mock_all_auths();
    client.initialize(&admin, &100, &100, &150);

    let documents = BytesN::from_array(&env, &[0; 32]);
    client.verify_compliance(&user, &documents);

    // Note: No direct getter for compliance data, so we rely on settlement to verify
    let sender = user.clone();
    let recipient = Address::generate(&env);
    client.verify_compliance(&recipient, &documents);

    let transfer_id = client.initiate_transfer(
        &sender,
        &recipient,
        &1000,
        &String::from_str(&env, "USD"),
        &String::from_str(&env, "SWIFT"),
    );

    let org = Address::generate(&env);
    client.execute_settlement(&transfer_id, &org); // Should succeed if compliance is verified
    assert_eq!(
        client.get_transfer_status(&transfer_id),
        SettlementStatus::Settled
    );
}

#[test]
fn test_fx_rate_default() {
    let env = Env::default();
    let admin = Address::generate(&env);

    let contract_address = env.register(CrossBorderPayment, ());
    let client = CrossBorderPaymentClient::new(&env, &contract_address);

    env.mock_all_auths();
    client.initialize(&admin, &100, &100, &150);

    let source = String::from_str(&env, "USD");
    let target = String::from_str(&env, "EUR");
    let rate = client.get_fx_rate(&source, &target);
    assert_eq!(rate, RATE_SCALE); // Default rate (1:1)
}
