use soroban_sdk::{Address, String, testutils::Address as _};
use super::utils::{setup_default, register_provider, new_session_id, consent_hash};

#[test]
fn confirm_valid_session() {
    let ctx = setup_default(250, 15, 240); // 2.5% fee
    let patient = Address::generate(&ctx.env);
    let provider = register_provider(&ctx, 8000);
    let session_id = new_session_id(&ctx.env, 10);
    let consent = consent_hash(&ctx.env, 10);
    let payment_id = ctx.client.initiate_payment(&patient, &provider, &session_id, &60, &consent);
    ctx.client.confirm_session(&provider, &payment_id, &55, &String::from_str(&ctx.env, "Session ok"));
    assert_eq!(ctx.client.get_payment_status(&payment_id), 2); // completed
}

#[test]
#[should_panic(expected = "Payment not found")]
fn confirm_nonexistent_payment() {
    let ctx = setup_default(200, 15, 240);
    let provider = register_provider(&ctx, 6000);
    let fake_payment = new_session_id(&ctx.env, 999);
    ctx.client.confirm_session(&provider, &fake_payment, &30, &String::from_str(&ctx.env, "NA"));
}

#[test]
#[should_panic(expected = "Unauthorized: Only the assigned provider can confirm session")]
fn unauthorized_provider_confirmation() {
    let ctx = setup_default(200, 15, 240);
    let patient = Address::generate(&ctx.env);
    let provider = register_provider(&ctx, 6000);
    let other_provider = register_provider(&ctx, 7000);
    let session_id = new_session_id(&ctx.env, 11);
    let consent = consent_hash(&ctx.env, 11);
    let payment_id = ctx.client.initiate_payment(&patient, &provider, &session_id, &30, &consent);
    ctx.client.confirm_session(&other_provider, &payment_id, &25, &String::from_str(&ctx.env, "Wrong"));
}
