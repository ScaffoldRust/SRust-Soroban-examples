#[cfg(test)]
mod test {
    use soroban_sdk::{testutils::Address as _, Address, Env, String, Vec};
    use crate::{SecureMedicalRecordsSharing, SecureMedicalRecordsSharingClient};


    #[test]
    fn test_add_and_get_record() {
        let env = Env::default();
        let id = env.register(SecureMedicalRecordsSharing, ());
        let client = SecureMedicalRecordsSharingClient::new(&env, &id);
        client.initialize();
        let patient = Address::generate(&env);
        let data_type = String::from_str(&env, "lab");
        let pointer = String::from_str(&env, "ipfs://QmLabHash");
        let record_id = client.add_record(&patient, &data_type, &pointer);
        assert_eq!(record_id, 1);
        let rec = client.get_record(&patient, &patient, &record_id);
        assert_eq!(rec.pointer, pointer);
    }

    #[test]
    fn test_grant_and_verify_access() {
        let env = Env::default();
        let id = env.register(SecureMedicalRecordsSharing, ());
        let client = SecureMedicalRecordsSharingClient::new(&env, &id);
        client.initialize();
        let patient = Address::generate(&env);
        let provider = Address::generate(&env);
        let data_type = String::from_str(&env, "imaging");
        let pointer = String::from_str(&env, "hash://img1");
        client.add_record(&patient, &data_type, &pointer);

        let mut types = Vec::new(&env);
        types.push_back(String::from_str(&env, "imaging"));
        let expiry = env.ledger().timestamp() + 3600;
        client.grant_access(&patient, &provider, &types, &expiry);
        assert!(client.verify_access(&patient, &provider, &String::from_str(&env, "imaging")));
    }

    #[test]
    fn test_revoke_access() {
        let env = Env::default();
        let id = env.register(SecureMedicalRecordsSharing, ());
        let client = SecureMedicalRecordsSharingClient::new(&env, &id);
        client.initialize();
        let patient = Address::generate(&env);
        let provider = Address::generate(&env);
        let mut types = Vec::new(&env);
        types.push_back(String::from_str(&env, "lab"));
        client.grant_access(&patient, &provider, &types, &0);
        assert!(client.verify_access(&patient, &provider, &String::from_str(&env, "lab")));
        client.revoke_access(&patient, &provider);
        assert!(!client.verify_access(&patient, &provider, &String::from_str(&env, "lab")));
    }

    #[test]
    fn test_emergency_read() {
        let env = Env::default();
        let id = env.register(SecureMedicalRecordsSharing, ());
        let client = SecureMedicalRecordsSharingClient::new(&env, &id);
        client.initialize();
        let patient = Address::generate(&env);
        let provider = Address::generate(&env);
        let data_type = String::from_str(&env, "rx");
        let pointer = String::from_str(&env, "cid://rx1");
        let rec_id = client.add_record(&patient, &data_type, &pointer);
        client.add_emergency_provider(&patient, &provider);
        let rec = client.emergency_read(&provider, &patient, &rec_id, &String::from_str(&env, "ER"));
        assert_eq!(rec.record_id, rec_id);
    }

    #[test]
    fn test_audit_log_growth_and_capping() {
        let env = Env::default();
        let id = env.register(SecureMedicalRecordsSharing, ());
        let client = SecureMedicalRecordsSharingClient::new(&env, &id);
        client.initialize();
        let patient = Address::generate(&env);
        let dt = String::from_str(&env, "lab");
        let base = String::from_str(&env, "hash://static");
        for _ in 0..205u32 { client.add_record(&patient, &dt, &base); }
        let log = client.get_audit_log(&patient, &patient);
        assert!(log.len() <= 200);
    }
}
