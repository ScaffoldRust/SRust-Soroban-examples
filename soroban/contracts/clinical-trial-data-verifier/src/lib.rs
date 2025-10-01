#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype, 
    Address, Env, String, Vec, Error,
};

mod data;
mod utils;
mod verifier;

pub use data::{DataStatus, VerificationEvent, TrialMetadata, StudyPhase};
pub use verifier::Verifier;

#[cfg(test)]
mod tests; // modular test suite (submission, verification, audit, utils)

#[contracttype]
pub enum DataKey {
    Admin,                  // Contract administrator
    Verifiers,             // List of authorized verifiers
    TrialData,             // This stores Map<String, ClinicalData>
    AuditTrail,            // This stores Vec<VerificationEvent>
    TrialConfig,           // Trial-specific configuration
    ConsentLinks,          // Links data to consent contracts
    SupplyChainLinks,      // Links data to supply chain contracts
}

#[contracttype]
pub struct TrialConfiguration {
    pub trial_id: String,
    pub start_date: u64,
    pub end_date: u64,
    pub required_verifications: u32,
    pub protocol_hash: String,          // Hash of trial protocol document
    pub protocol_version: String,       // ICH-GCP protocol version
    pub ethics_approval_id: String,     // Ethics committee approval ID
    pub study_phase: StudyPhase,        // Clinical trial phase
}

#[derive(Clone)]
#[contracttype]
pub struct ConsentLink {
    pub data_hash: String,
    pub consent_contract_id: Address,
    pub patient_id_hash: String,        // Hashed patient identifier
    pub consent_timestamp: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct SupplyChainLink {
    pub data_hash: String,
    pub supply_contract_id: Address,
    pub batch_number: String,
    pub link_timestamp: u64,
}

#[contract]
pub struct ClinicalTrialVerifier;

#[contractimpl]
impl ClinicalTrialVerifier {
    /// Initialize the contract with trial configuration
    /// Complies with ICH-GCP standards by requiring ethics approval and protocol version
    pub fn initialize(
        env: Env,
        admin: Address,
        trial_config: TrialConfiguration,
    ) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::from_contract_error(1));
        }

        // Validate ICH-GCP required fields
        if trial_config.ethics_approval_id.len() == 0 {
            return Err(Error::from_contract_error(10));
        }
        if trial_config.protocol_version.len() == 0 {
            return Err(Error::from_contract_error(11));
        }

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::TrialConfig, &trial_config);
        env.storage().instance().set(&DataKey::Verifiers, &Vec::<Address>::new(&env));
        env.storage().instance().set(&DataKey::ConsentLinks, &Vec::<ConsentLink>::new(&env));
        env.storage().instance().set(&DataKey::SupplyChainLinks, &Vec::<SupplyChainLink>::new(&env));

        Ok(())
    }

    /// Add an authorized verifier (researcher or medical professional)
    pub fn add_verifier(env: Env, admin: Address, verifier: Address) -> Result<(), Error> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;
        
        let mut verifiers: Vec<Address> = env.storage().instance()
            .get(&DataKey::Verifiers)
            .unwrap_or_else(|| Vec::new(&env));
        
        if !verifiers.contains(&verifier) {
            verifiers.push_back(verifier);
            env.storage().instance().set(&DataKey::Verifiers, &verifiers);
        }

        Ok(())
    }

    /// Remove a verifier
    pub fn remove_verifier(env: Env, admin: Address, verifier: Address) -> Result<(), Error> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;
        
        let verifiers: Vec<Address> = env.storage().instance()
            .get(&DataKey::Verifiers)
            .unwrap_or_else(|| Vec::new(&env));
        
        let mut new_verifiers = Vec::new(&env);
        for v in verifiers.iter() {
            if v != verifier {
                new_verifiers.push_back(v);
            }
        }
        
        env.storage().instance().set(&DataKey::Verifiers, &new_verifiers);
        Ok(())
    }

    /// Submit clinical trial data with metadata
    /// Researchers submit patient outcomes, measurements, adverse events, etc.
    pub fn submit_data(
        env: Env,
        submitter: Address,
        trial_id: String,
        data_hash: String,
        metadata_hash: String,
        metadata: TrialMetadata,
    ) -> Result<(), Error> {
        submitter.require_auth();
        Verifier::submit_data(&env, submitter, trial_id, data_hash, metadata_hash, metadata)
    }

    /// Verify submitted data (authorized verifiers only)
    /// Marks data as verified after validation against protocol
    pub fn verify_data(
        env: Env,
        verifier: Address,
        data_hash: String,
        approved: bool,
        notes: String,
    ) -> Result<(), Error> {
        verifier.require_auth();
        
        if !Self::is_authorized_verifier(&env, &verifier) {
            return Err(Error::from_contract_error(2));
        }
        
        Verifier::verify_data(&env, verifier, data_hash, approved, notes)
    }

    /// Link data to a patient consent contract
    /// Ensures compliance with informed consent requirements
    pub fn link_consent_contract(
        env: Env,
        admin: Address,
        data_hash: String,
        consent_contract_id: Address,
        patient_id_hash: String,
    ) -> Result<(), Error> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;

        let consent_link = ConsentLink {
            data_hash,
            consent_contract_id,
            patient_id_hash,
            consent_timestamp: env.ledger().timestamp(),
        };

        let mut consent_links: Vec<ConsentLink> = env.storage().instance()
            .get(&DataKey::ConsentLinks)
            .unwrap_or_else(|| Vec::new(&env));
        
        consent_links.push_back(consent_link);
        env.storage().instance().set(&DataKey::ConsentLinks, &consent_links);

        Ok(())
    }

    /// Link data to a supply chain contract
    /// Tracks drug batches, medical supplies used in trial
    pub fn link_supply_chain(
        env: Env,
        admin: Address,
        data_hash: String,
        supply_contract_id: Address,
        batch_number: String,
    ) -> Result<(), Error> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;

        let supply_link = SupplyChainLink {
            data_hash,
            supply_contract_id,
            batch_number,
            link_timestamp: env.ledger().timestamp(),
        };

        let mut supply_links: Vec<SupplyChainLink> = env.storage().instance()
            .get(&DataKey::SupplyChainLinks)
            .unwrap_or_else(|| Vec::new(&env));
        
        supply_links.push_back(supply_link);
        env.storage().instance().set(&DataKey::SupplyChainLinks, &supply_links);

        Ok(())
    }

    /// Get verification status of specific data
    pub fn get_verification_status(
        env: Env,
        data_hash: String,
    ) -> Result<DataStatus, Error> {
        Verifier::get_verification_status(&env, data_hash)
    }

    /// Get complete audit trail for specific data
    /// Returns all verification events for transparency
    pub fn get_audit_trail(
        env: Env,
        data_hash: String,
    ) -> Result<Vec<VerificationEvent>, Error> {
        Verifier::get_audit_trail(&env, data_hash)
    }

    /// Get consent links for specific data
    pub fn get_consent_links(
        env: Env,
        data_hash: String,
    ) -> Result<Vec<ConsentLink>, Error> {
        let consent_links: Vec<ConsentLink> = env.storage().instance()
            .get(&DataKey::ConsentLinks)
            .unwrap_or_else(|| Vec::new(&env));

        let mut result = Vec::new(&env);
        for link in consent_links.iter() {
            if link.data_hash == data_hash {
                result.push_back(link);
            }
        }
        Ok(result)
    }

    /// Get supply chain links for specific data
    pub fn get_supply_chain_links(
        env: Env,
        data_hash: String,
    ) -> Result<Vec<SupplyChainLink>, Error> {
        let supply_links: Vec<SupplyChainLink> = env.storage().instance()
            .get(&DataKey::SupplyChainLinks)
            .unwrap_or_else(|| Vec::new(&env));

        let mut result = Vec::new(&env);
        for link in supply_links.iter() {
            if link.data_hash == data_hash {
                result.push_back(link);
            }
        }
        Ok(result)
    }

    /// Get trial configuration
    pub fn get_trial_config(env: Env) -> Result<TrialConfiguration, Error> {
        env.storage().instance()
            .get(&DataKey::TrialConfig)
            .ok_or(Error::from_contract_error(5))
    }

    /// Internal helper to check admin authorization
    fn require_admin(env: &Env, caller: &Address) -> Result<(), Error> {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != *caller {
            return Err(Error::from_contract_error(3));
        }
        Ok(())
    }

    /// Internal helper to check verifier authorization
    pub(crate) fn is_authorized_verifier(env: &Env, verifier: &Address) -> bool {
        let verifiers: Vec<Address> = env.storage().instance()
            .get(&DataKey::Verifiers)
            .unwrap_or_else(|| Vec::new(&env));
        verifiers.contains(verifier)
    }
}