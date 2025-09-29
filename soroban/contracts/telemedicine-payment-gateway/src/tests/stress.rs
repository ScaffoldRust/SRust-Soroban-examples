use soroban_sdk::{Address, testutils::Address as _};
use super::utils::{setup_default, register_provider, new_session_id, consent_hash};

#[test]
fn high_volume_payments() {
    let ctx = setup_default(200, 15, 240);
    let provider = register_provider(&ctx, 9000);
    let patient = Address::generate(&ctx.env);
    for i in 1..=50u64 { // simulate 50 payment initiations
        let session_id = new_session_id(&ctx.env, 100 + i);
        let consent = consent_hash(&ctx.env, 100 + i);
        let pid = ctx.client.initiate_payment(&patient, &provider, &session_id, &30, &consent);
        assert_eq!(ctx.client.get_payment_status(&pid), 0);
    }
}
