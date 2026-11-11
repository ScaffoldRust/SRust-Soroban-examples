use soroban_sdk::testutils::Address as _;

use super::utils::{TestContext, DONATION_ID};

#[test]
#[should_panic(expected = "Invalid donation status")]
fn release_before_completion_is_rejected() {
    let ctx = TestContext::new(1);
    ctx.funded();
    let client = ctx.client();
    client.release_funds(&DONATION_ID, &ctx.recipient);
}

#[test]
#[should_panic(expected = "Unauthorized releaser")]
fn unauthorized_release_is_rejected() {
    let ctx = TestContext::new(1);
    let other = soroban_sdk::Address::generate(&ctx.env);
    ctx.funded();
    ctx.verify(&ctx.recipient, 0);
    let client = ctx.client();
    client.release_funds(&DONATION_ID, &other);
}

#[test]
fn authorized_release_removes_item_from_queue() {
    let ctx = TestContext::new(1);
    ctx.funded();
    ctx.verify(&ctx.recipient, 0);
    let client = ctx.client();
    assert!(client.get_release_queue().contains(DONATION_ID));
    client.release_funds(&DONATION_ID, &ctx.recipient);
    assert!(!client.get_release_queue().contains(DONATION_ID));
}

#[test]
#[should_panic(expected = "Not in release queue")]
fn duplicate_release_is_rejected() {
    let ctx = TestContext::new(1);
    ctx.funded();
    ctx.verify(&ctx.recipient, 0);
    let client = ctx.client();
    client.release_funds(&DONATION_ID, &ctx.recipient);
    client.release_funds(&DONATION_ID, &ctx.recipient);
}

#[test]
fn refund_success_and_queue() {
    let ctx = TestContext::new(1);
    ctx.funded();
    let client = ctx.client();
    client.refund(&DONATION_ID, &ctx.donor);
    assert!(client.get_refund_queue().contains(DONATION_ID));
}

#[test]
#[should_panic(expected = "Unauthorized refund processor")]
fn unauthorized_refund_is_rejected() {
    let ctx = TestContext::new(1);
    let other = soroban_sdk::Address::generate(&ctx.env);
    ctx.funded();
    let client = ctx.client();
    client.refund(&DONATION_ID, &other);
}

#[test]
#[should_panic(expected = "Cannot refund donation")]
fn refund_of_completed_donation_is_rejected() {
    let ctx = TestContext::new(1);
    ctx.funded();
    ctx.verify(&ctx.recipient, 0);
    let client = ctx.client();
    client.refund(&DONATION_ID, &ctx.donor);
}

#[test]
#[should_panic(expected = "Cannot refund donation")]
fn duplicate_refund_is_rejected() {
    let ctx = TestContext::new(1);
    ctx.funded();
    let client = ctx.client();
    client.refund(&DONATION_ID, &ctx.donor);
    client.refund(&DONATION_ID, &ctx.donor);
}
