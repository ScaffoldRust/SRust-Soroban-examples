use soroban_sdk::{Address, String, testutils::Address as _};
use super::utils::{setup_default, register_provider, new_session_id, consent_hash};

#[test]
fn refund_pending_payment() {
    let ctx = setup_default(200, 15, 240);
    let patient = Address::generate(&ctx.env);
    let provider = register_provider(&ctx, 6500);
    let session_id = new_session_id(&ctx.env, 20);
    let consent = consent_hash(&ctx.env, 20);
    let payment_id = ctx.client.initiate_payment(&patient, &provider, &session_id, &30, &consent);
    ctx.client.refund_payment(&patient, &payment_id, &String::from_str(&ctx.env, "Cancelled"));
    assert_eq!(ctx.client.get_payment_status(&payment_id), 3); // refunded
}

#[test]
#[should_panic(expected = "Payment already refunded")]
fn refund_already_refunded() {
    let ctx = setup_default(200, 15, 240);
    let patient = Address::generate(&ctx.env);
    let provider = register_provider(&ctx, 6500);
    let session_id = new_session_id(&ctx.env, 21);
    let consent = consent_hash(&ctx.env, 21);
    let payment_id = ctx.client.initiate_payment(&patient, &provider, &session_id, &30, &consent);
    ctx.client.refund_payment(&patient, &payment_id, &String::from_str(&ctx.env, "Cancelled"));
    ctx.client.refund_payment(&patient, &payment_id, &String::from_str(&ctx.env, "Cancelled again"));
}

#[test]
#[should_panic(expected = "Payment not found")]
fn refund_nonexistent_payment() {
    let ctx = setup_default(200, 15, 240);
    let patient = Address::generate(&ctx.env);
    let fake_payment = new_session_id(&ctx.env, 9999);
    ctx.client.refund_payment(&patient, &fake_payment, &String::from_str(&ctx.env, "Invalid"));
}
