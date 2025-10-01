use soroban_sdk::{testutils::Address as _, Address, String};
use super::utils::{setup, random_hash, sample_metadata};
use crate::{DataStatus};

#[test]
fn multi_step_verification_to_verified() {
    let ctx = setup();
    let data_hash = random_hash(&ctx.env, 50);
    let meta_hash = random_hash(&ctx.env, 1050);
    let metadata = sample_metadata(&ctx.env, 21);
    ctx.client.submit_data(&ctx.verifier, &String::from_str(&ctx.env, "TRIAL-001"), &data_hash, &meta_hash, &metadata);
    // First verification -> UnderVerification
    ctx.client.verify_data(&ctx.verifier, &data_hash, &true, &String::from_str(&ctx.env, "Looks ok"));
    let status1 = ctx.client.get_verification_status(&data_hash);
    assert!(matches!(status1, DataStatus::UnderVerification));
    // Add second verifier
    let second = Address::generate(&ctx.env);
    ctx.client.add_verifier(&ctx.admin, &second);
    ctx.client.verify_data(&second, &data_hash, &true, &String::from_str(&ctx.env, "Confirm"));
    let status2 = ctx.client.get_verification_status(&data_hash);
    assert!(matches!(status2, DataStatus::Verified));
}

#[test]
fn rejection_sets_status_rejected() {
    let ctx = setup();
    let data_hash = random_hash(&ctx.env, 51);
    let meta_hash = random_hash(&ctx.env, 2051);
    let metadata = sample_metadata(&ctx.env, 22);
    ctx.client.submit_data(&ctx.verifier, &String::from_str(&ctx.env, "TRIAL-001"), &data_hash, &meta_hash, &metadata);
    ctx.client.verify_data(&ctx.verifier, &data_hash, &false, &String::from_str(&ctx.env, "Deviation"));
    let status = ctx.client.get_verification_status(&data_hash);
    assert!(matches!(status, DataStatus::Rejected));
}

#[test]
#[should_panic]
fn verify_nonexistent_data_fails() {
    let ctx = setup();
    // Should panic: data not found
    ctx.client.verify_data(&ctx.verifier, &String::from_str(&ctx.env, "nonexistenthash00000000000000000000000000000000000000000000000000000000"), &true, &String::from_str(&ctx.env, "NA"));
}

#[test]
#[should_panic]
fn unauthorized_verifier_fails() {
    let ctx = setup();
    let data_hash = random_hash(&ctx.env, 52);
    let meta_hash = random_hash(&ctx.env, 3052);
    let metadata = super::utils::sample_metadata(&ctx.env, 23);
    ctx.client.submit_data(&ctx.verifier, &String::from_str(&ctx.env, "TRIAL-001"), &data_hash, &meta_hash, &metadata);
    let outsider = Address::generate(&ctx.env);
    // Should panic unauthorized
    ctx.client.verify_data(&outsider, &data_hash, &true, &String::from_str(&ctx.env, "No rights"));
}
