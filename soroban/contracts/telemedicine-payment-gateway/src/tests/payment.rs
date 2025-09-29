use soroban_sdk::{Address, testutils::Address as _};
use super::utils::{setup_default, register_provider, new_session_id, consent_hash};

#[test]
fn initiate_valid_payment() {
    let ctx = setup_default(200, 15, 240); // 2% fee
    let patient = Address::generate(&ctx.env);
    let provider = register_provider(&ctx, 6000); // 60.00/hr
    let session_id = new_session_id(&ctx.env, 1);
    let consent = consent_hash(&ctx.env, 1);
    let payment_id = ctx.client.initiate_payment(&patient, &provider, &session_id, &30, &consent);
    assert_eq!(ctx.client.get_payment_status(&payment_id), 0); // pending
}

#[test]
#[should_panic(expected = "Provider not found or inactive")]
fn payment_with_unregistered_provider() {
    let ctx = setup_default(200, 15, 240);
    let patient = Address::generate(&ctx.env);
    let provider = Address::generate(&ctx.env); // not registered
    let session_id = new_session_id(&ctx.env, 2);
    let consent = consent_hash(&ctx.env, 2);
    ctx.client.initiate_payment(&patient, &provider, &session_id, &30, &consent);
}

#[test]
#[should_panic(expected = "Payment already exists for this session")]
fn duplicate_payment_same_session() {
    let ctx = setup_default(200, 15, 240);
    let patient = Address::generate(&ctx.env);
    let provider = register_provider(&ctx, 7000);
    let session_id = new_session_id(&ctx.env, 3);
    let consent = consent_hash(&ctx.env, 3);
    ctx.client.initiate_payment(&patient, &provider, &session_id, &45, &consent);
    ctx.client.initiate_payment(&patient, &provider, &session_id, &45, &consent);
}

#[test]
#[should_panic(expected = "Invalid payment amount")]
fn invalid_zero_duration_payment() {
    let ctx = setup_default(200, 15, 240);
    let patient = Address::generate(&ctx.env);
    let provider = register_provider(&ctx, 5000);
    let session_id = new_session_id(&ctx.env, 4);
    let consent = consent_hash(&ctx.env, 4);
    ctx.client.initiate_payment(&patient, &provider, &session_id, &0, &consent);
}
