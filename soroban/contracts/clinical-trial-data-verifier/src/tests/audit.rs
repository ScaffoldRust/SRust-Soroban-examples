use soroban_sdk::{testutils::Address as _, Address, String};
use super::utils::{setup, random_hash, sample_metadata};
use crate::{DataStatus};

#[test]
fn audit_trail_records_events_in_order() {
    let ctx = setup();
    let data_hash = random_hash(&ctx.env, 90);
    let meta_hash = random_hash(&ctx.env, 9090);
    let metadata = sample_metadata(&ctx.env, 31);
    ctx.client.submit_data(&ctx.verifier, &String::from_str(&ctx.env, "TRIAL-001"), &data_hash, &meta_hash, &metadata);
    let second = Address::generate(&ctx.env);
    ctx.client.add_verifier(&ctx.admin, &second);
    ctx.client.verify_data(&ctx.verifier, &data_hash, &true, &String::from_str(&ctx.env, "Initial"));
    ctx.client.verify_data(&second, &data_hash, &true, &String::from_str(&ctx.env, "Final"));
    let trail = ctx.client.get_audit_trail(&data_hash);
    assert_eq!(trail.len(), 2);
    let first = trail.get(0).unwrap();
    let last = trail.get(1).unwrap();
    assert!(matches!(first.status, DataStatus::UnderVerification));
    assert!(matches!(last.status, DataStatus::Verified));
}

#[test]
fn audit_trail_empty_for_unknown_data() {
    let ctx = setup();
    let trail = ctx.client.get_audit_trail(&String::from_str(&ctx.env, "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"));
    assert_eq!(trail.len(), 0);
}
