use soroban_sdk::{Env, Address, String, testutils::Address as _};
use crate::{ClinicalTrialVerifier, ClinicalTrialVerifierClient, TrialConfiguration, StudyPhase, TrialMetadata, data::{DataType}};

pub struct TestContext {
    pub env: Env,
    pub client: ClinicalTrialVerifierClient<'static>,
    pub admin: Address,
    pub verifier: Address,
}

pub fn default_trial_config(env: &Env) -> TrialConfiguration {
    TrialConfiguration {
        trial_id: String::from_str(env, "TRIAL-001"),
        start_date: 1,
        end_date: 999999,
        required_verifications: 2,
        protocol_hash: String::from_str(env, "protohashv1"),
        protocol_version: String::from_str(env, "1.0"),
        ethics_approval_id: String::from_str(env, "ETH-APPROVED"),
        study_phase: StudyPhase::Phase2,
    }
}

pub fn setup() -> TestContext {
    let env = Env::default();
    // Allow all addresses to satisfy require_auth() in tests
    env.mock_all_auths();
    let id = env.register(ClinicalTrialVerifier, ());
    let client = ClinicalTrialVerifierClient::new(&env, &id);
    let admin = Address::generate(&env);
    let verifier = Address::generate(&env);
    let cfg = default_trial_config(&env);
    client.initialize(&admin, &cfg); // returns ()
    client.add_verifier(&admin, &verifier); // returns ()
    TestContext { env, client, admin, verifier }
}

pub fn random_hash(env: &Env, seed: u64) -> String {
    // produce deterministic 64 hex chars using seed
    let mut bytes = [0u8;32];
    bytes[0..8].copy_from_slice(&seed.to_le_bytes());
    let hex = b"0123456789abcdef";
    let mut out = [0u8;64];
    for (i,b) in bytes.iter().enumerate() { out[i*2]=hex[(b>>4) as usize]; out[i*2+1]=hex[(b & 0xF) as usize]; }
    String::from_bytes(env, &out)
}

pub fn sample_metadata(env: &Env, patient_seed: u64) -> TrialMetadata {
    TrialMetadata {
        data_type: DataType::LabMeasurement,
        patient_id_hash: random_hash(env, 10 + patient_seed),
        visit_number: 1,
        measurement_date: 123456 + patient_seed,
        protocol_deviation: false,
        source_verified: true,
    }
}
