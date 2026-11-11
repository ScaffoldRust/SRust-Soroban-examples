use soroban_sdk::testutils::Address as _;

use super::utils::{TestContext, DONATION_ID};
use crate::escrow::DonationStatus;

#[test]
fn partial_and_final_milestone_verification() {
    let ctx = TestContext::new(2);
    let client = ctx.client();
    ctx.funded();
    ctx.verify(&ctx.recipient, 0);
    assert_eq!(
        client.get_donation_status(&DONATION_ID),
        Some(DonationStatus::InProgress)
    );
    ctx.verify(&ctx.recipient, 1);
    assert_eq!(
        client.get_donation_status(&DONATION_ID),
        Some(DonationStatus::Completed)
    );
    assert!(client.get_milestone_status(&DONATION_ID, &0));
}

#[test]
#[should_panic(expected = "Invalid milestone index")]
fn invalid_milestone_index_is_rejected() {
    let ctx = TestContext::new(1);
    ctx.funded();
    ctx.verify(&ctx.recipient, 1);
}

#[test]
#[should_panic(expected = "Milestone already verified")]
fn duplicate_milestone_is_rejected() {
    // Use 2 milestones so the status transitions to InProgress after first verification,
    // allowing verify_milestone to reach the duplicate check.
    let ctx = TestContext::new(2);
    ctx.funded();
    ctx.verify(&ctx.recipient, 0);
    ctx.verify(&ctx.recipient, 0);
}

#[test]
#[should_panic(expected = "Unauthorized verifier")]
fn unauthorized_milestone_verification_is_rejected() {
    let ctx = TestContext::new(1);
    let other = soroban_sdk::Address::generate(&ctx.env);
    ctx.funded();
    ctx.verify(&other, 0);
}
