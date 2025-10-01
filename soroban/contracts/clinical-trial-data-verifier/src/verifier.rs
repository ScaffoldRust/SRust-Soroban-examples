use soroban_sdk::{
    Env, Address, String, Map, Vec, Error,
};
use soroban_sdk::IntoVal;
use crate::{
    data::{ClinicalData, DataStatus, VerificationEvent, TrialMetadata},
    DataKey,
};

pub struct Verifier;

impl Verifier {
    /// Submit new clinical trial data for verification
    /// Validates ICH-GCP compliance before accepting submission
    pub fn submit_data(
        env: &Env,
        submitter: Address,
        trial_id: String,
        data_hash: String,
        metadata_hash: String,
        metadata: TrialMetadata,
    ) -> Result<(), Error> {
        // Validate data hash format (should be 64-char hex)
        if !crate::utils::validate_hash(env, &data_hash) {
            return Err(Error::from_contract_error(12));
        }

        let data = ClinicalData::new(
            env,
            trial_id.clone(),
            data_hash.clone(),
            metadata_hash,
            metadata,
            submitter.clone(),
        );

        // Validate GCP compliance
        if !data.validate_gcp_compliance() {
            return Err(Error::from_contract_error(13));
        }

        let mut trial_data: Map<String, ClinicalData> = env.storage().instance()
            .get(&DataKey::TrialData)
            .unwrap_or_else(|| Map::new(env));

        if trial_data.contains_key(data_hash.clone()) {
            return Err(Error::from_contract_error(1));
        }

        trial_data.set(data_hash.clone(), data);
        env.storage().instance().set(&DataKey::TrialData, &trial_data);

        // Emit submission event
        env.events().publish(
            (String::from_str(env, "data_submitted"), trial_id),
            (submitter, data_hash),
        );

        Ok(())
    }

    /// Verify submitted clinical trial data
    /// Checks data against protocol requirements and GCP standards
    pub fn verify_data(
        env: &Env,
        verifier: Address,
        data_hash: String,
        approved: bool,
        notes: String,
    ) -> Result<(), Error> {
        if !super::ClinicalTrialVerifier::is_authorized_verifier(env, &verifier) {
            return Err(Error::from_contract_error(2));
        }

        let mut trial_data: Map<String, ClinicalData> = env.storage().instance()
            .get(&DataKey::TrialData)
            .ok_or(Error::from_contract_error(4))?;

        let mut data = trial_data.get(data_hash.clone())
            .ok_or(Error::from_contract_error(4))?;

        // Cannot verify already rejected data
        if matches!(data.status, DataStatus::Rejected) {
            return Err(Error::from_contract_error(14));
        }

        data.add_verification();
        
        let config: super::TrialConfiguration = env.storage().instance()
            .get(&DataKey::TrialConfig)
            .ok_or(Error::from_contract_error(5))?;

        let new_status = if approved {
            if data.verifications >= config.required_verifications {
                DataStatus::Verified
            } else {
                DataStatus::UnderVerification
            }
        } else {
            DataStatus::Rejected
        };

        data.update_status(env, new_status.clone());
        trial_data.set(data_hash.clone(), data.clone());
        env.storage().instance().set(&DataKey::TrialData, &trial_data);

        // Record verification with GCP compliance flag
        let gcp_compliant = data.validate_gcp_compliance() && approved;
        Self::record_verification_event(
            env,
            data_hash,
            verifier,
            new_status,
            notes,
            gcp_compliant,
        );

        Ok(())
    }

    /// Record a verification event in the audit trail
    /// Maintains complete audit trail for regulatory compliance
    fn record_verification_event(
        env: &Env,
        data_hash: String,
        verifier: Address,
        status: DataStatus,
        notes: String,
        gcp_compliant: bool,
    ) {
        let event = VerificationEvent {
            data_hash: data_hash.clone(),
            verifier: verifier.clone(),
            timestamp: env.ledger().timestamp(),
            status: status.clone(),
            notes: notes.clone(),
            gcp_compliant,
        };

        let mut audit_trail: Vec<VerificationEvent> = env.storage().instance()
            .get(&DataKey::AuditTrail)
            .unwrap_or_else(|| Vec::new(env));
        
        audit_trail.push_back(event);
        env.storage().instance().set(&DataKey::AuditTrail, &audit_trail);

        // Emit verification event for transparency
        env.events().publish(
            (String::from_str(env, "data_verified"), data_hash),
            (verifier, status, gcp_compliant),
        );
    }

    /// Get verification status of specific data
    pub fn get_verification_status(
        env: &Env,
        data_hash: String,
    ) -> Result<DataStatus, Error> {
        let trial_data: Map<String, ClinicalData> = env.storage().instance()
            .get(&DataKey::TrialData)
            .ok_or(Error::from_contract_error(4))?;

        let data = trial_data.get(data_hash)
            .ok_or(Error::from_contract_error(4))?;

        Ok(data.status)
    }

    /// Get audit trail for specific data
    /// Returns complete verification history for regulatory audits
    pub fn get_audit_trail(
        env: &Env,
        data_hash: String,
    ) -> Result<Vec<VerificationEvent>, Error> {
        let audit_trail: Vec<VerificationEvent> = env.storage().instance()
            .get(&DataKey::AuditTrail)
            .unwrap_or_else(|| Vec::new(env));

        let mut result = Vec::new(env);
        for event in audit_trail.iter() {
            if event.data_hash == data_hash {
                result.push_back(event.clone());
            }
        }
        Ok(result)
    }

    /// Get all data submissions for a trial
    pub fn get_trial_submissions(
        env: &Env,
        trial_id: String,
    ) -> Result<Vec<ClinicalData>, Error> {
        let trial_data: Map<String, ClinicalData> = env.storage().instance()
            .get(&DataKey::TrialData)
            .unwrap_or_else(|| Map::new(env));

        let mut result = Vec::new(env);
        for data in trial_data.values() {
            if data.trial_id == trial_id {
                result.push_back(data);
            }
        }
        Ok(result)
    }
}