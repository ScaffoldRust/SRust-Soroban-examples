use soroban_sdk::{symbol_short, testutils::Address as _, String, Vec};

use super::utils::{TestContext, AMOUNT, DONATION_ID};
use crate::escrow::DonationStatus;

#[test]
fn setup_and_valid_deposit() {
    let ctx = TestContext::new(2);
    let client = ctx.client();
    assert_eq!(
        client.get_donation_status(&DONATION_ID),
        Some(DonationStatus::Pending)
    );
    ctx.funded();
    assert_eq!(
        client.get_donation_status(&DONATION_ID),
        Some(DonationStatus::Funded)
    );
}

#[test]
#[should_panic(expected = "Amount or token mismatch")]
fn deposit_rejects_wrong_amount_or_token() {
    let ctx = TestContext::new(1);
    let client = ctx.client();
    client.deposit(
        &DONATION_ID,
        &ctx.donor,
        &(AMOUNT + 1),
        &symbol_short!("USDC"),
    );
}

#[test]
#[should_panic(expected = "Unauthorized donor")]
fn deposit_rejects_wrong_donor() {
    let ctx = TestContext::new(1);
    let other = soroban_sdk::Address::generate(&ctx.env);
    let client = ctx.client();
    client.deposit(&DONATION_ID, &other, &AMOUNT, &symbol_short!("USDC"));
}

#[test]
#[should_panic(expected = "Invalid donation status")]
fn duplicate_deposit_is_rejected() {
    let ctx = TestContext::new(1);
    ctx.funded();
    ctx.funded();
}

#[test]
fn parameter_validation_matches_contract() {
    let ctx = TestContext::new(1);
    let client = ctx.client();
    let mut milestones = Vec::new(&ctx.env);
    milestones.push_back(String::from_str(&ctx.env, "ordered"));
    assert!(client.validate_donation_parameters(&AMOUNT, &symbol_short!("USDC"), &milestones));
    assert!(!client.validate_donation_parameters(&0, &symbol_short!("USDC"), &milestones));
}
