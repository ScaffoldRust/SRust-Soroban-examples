use soroban_sdk::String;
use super::utils::{setup, random_hash, sample_metadata};
use crate::{DataStatus, TrialMetadata, data::DataType};

#[test]
fn submit_valid_data() {
    let ctx = setup();
    let data_hash = random_hash(&ctx.env, 1);
    let meta_hash = random_hash(&ctx.env, 1001);
    let metadata = sample_metadata(&ctx.env, 7);
    ctx.client.submit_data(&ctx.verifier, &String::from_str(&ctx.env, "TRIAL-001"), &data_hash, &meta_hash, &metadata);
    let status = ctx.client.get_verification_status(&data_hash);
    assert!(matches!(status, DataStatus::Submitted));
}

#[test]
#[should_panic]
fn duplicate_submission_fails() {
    let ctx = setup();
    let data_hash = random_hash(&ctx.env, 2);
    let meta_hash = random_hash(&ctx.env, 1002);
    let metadata = sample_metadata(&ctx.env, 8);
    ctx.client.submit_data(&ctx.verifier, &String::from_str(&ctx.env, "TRIAL-001"), &data_hash, &meta_hash, &metadata);
    // second triggers panic in contract code (duplicate)
    ctx.client.submit_data(&ctx.verifier, &String::from_str(&ctx.env, "TRIAL-001"), &data_hash, &meta_hash, &metadata);
}

#[test]
#[should_panic]
fn invalid_hash_rejected() {
    let ctx = setup();
    let bad_hash = String::from_str(&ctx.env, "abc");
    let meta_hash = random_hash(&ctx.env, 2000);
    let metadata = sample_metadata(&ctx.env, 9);
    // Should panic due to invalid hash length
    ctx.client.submit_data(&ctx.verifier, &String::from_str(&ctx.env, "TRIAL-001"), &bad_hash, &meta_hash, &metadata);
}

#[test]
#[should_panic]
fn gcp_non_compliant_metadata_rejected() {
    let ctx = setup();
    let data_hash = random_hash(&ctx.env, 3);
    let meta_hash = random_hash(&ctx.env, 3003);
    // Create metadata that violates GCP compliance (source_verified = false)
    let bad_meta = TrialMetadata {
        data_type: DataType::LabMeasurement,
        patient_id_hash: random_hash(&ctx.env, 9999),
        visit_number: 1,
        measurement_date: 1234567,
        protocol_deviation: false,
        source_verified: false, // triggers error 13
    };
    ctx.client.submit_data(&ctx.verifier, &String::from_str(&ctx.env, "TRIAL-001"), &data_hash, &meta_hash, &bad_meta);
}

#[test]
fn high_volume_submissions() {
    let ctx = setup();
    for i in 0..50u64 { // stress
        let data_hash = random_hash(&ctx.env, 4000 + i);
        let meta_hash = random_hash(&ctx.env, 8000 + i);
        let metadata = sample_metadata(&ctx.env, i);
    ctx.client.submit_data(&ctx.verifier, &String::from_str(&ctx.env, "TRIAL-001"), &data_hash, &meta_hash, &metadata);
    }
}
