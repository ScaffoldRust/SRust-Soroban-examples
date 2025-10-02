#![no_std]

mod consent;
mod audit;
mod utils;

use soroban_sdk::{contract, contractimpl, Address, Env, String, Vec};

#[contract]
pub struct PatientConsentManagementSystem;

#[contractimpl]
impl PatientConsentManagementSystem {
    /// Initialize the contract (optional - sets up any global config if needed)
    pub fn initialize(env: Env) {
        // Initialize next consent ID counter
        env.storage().instance().set(&consent::NEXT_CONSENT_ID, &1u64);
    }

    /// Create a new consent
    pub fn create_consent(
        env: Env,
        patient: Address,
        authorized_party: Address,
        data_scopes: Vec<consent::DataScope>,
        purpose: String,
        expires_at: Option<u64>,
    ) -> u64 {
        let consent_id = consent::create_consent(
            &env,
            patient.clone(),
            authorized_party,
            data_scopes,
            purpose,
            expires_at,
        );

        // Log audit event
        audit::log_consent_created(&env, consent_id, patient);

        consent_id
    }

    /// Update an existing consent
    pub fn update_consent(
        env: Env,
        consent_id: u64,
        patient: Address,
        new_data_scopes: Option<Vec<consent::DataScope>>,
        new_purpose: Option<String>,
        new_expires_at: Option<Option<u64>>,
    ) {
        consent::update_consent(
            &env,
            consent_id,
            patient.clone(),
            new_data_scopes,
            new_purpose.clone(),
            new_expires_at,
        );

        // Log audit event
        audit::log_consent_updated(&env, consent_id, patient, new_purpose);
    }

    /// Revoke a consent
    pub fn revoke_consent(
        env: Env,
        consent_id: u64,
        patient: Address,
    ) {
        consent::revoke_consent(&env, consent_id, patient.clone());

        // Log audit event
        audit::log_consent_revoked(&env, consent_id, patient);
    }

    /// Suspend a consent temporarily
    pub fn suspend_consent(
        env: Env,
        consent_id: u64,
        patient: Address,
    ) {
        consent::suspend_consent(&env, consent_id, patient.clone());

        // Log audit event
        audit::log_consent_suspended(&env, consent_id, patient);
    }

    /// Resume a suspended consent
    pub fn resume_consent(
        env: Env,
        consent_id: u64,
        patient: Address,
    ) {
        consent::resume_consent(&env, consent_id, patient.clone());

        // Log audit event
        audit::log_consent_resumed(&env, consent_id, patient);
    }

    /// Check if a consent is valid for a specific party and data scope
    pub fn check_consent(
        env: Env,
        consent_id: u64,
        authorized_party: Address,
        data_scope: consent::DataScope,
    ) -> bool {
        consent::check_consent(&env, consent_id, authorized_party, data_scope)
    }

    /// Check if a consent is currently active
    pub fn is_consent_active(
        env: Env,
        consent_id: u64,
    ) -> bool {
        consent::check_consent_active(&env, consent_id)
    }

    /// Get consent details
    pub fn get_consent(
        env: Env,
        consent_id: u64,
    ) -> Option<consent::Consent> {
        consent::get_consent(&env, consent_id)
    }

    /// Get all consents for a patient
    pub fn get_patient_consents(
        env: Env,
        patient: Address,
    ) -> Vec<u64> {
        consent::get_patient_consents(&env, patient)
    }

    /// Get all consents where a party is authorized
    pub fn get_party_consents(
        env: Env,
        party: Address,
    ) -> Vec<u64> {
        consent::get_party_consents(&env, party)
    }

    /// Get audit log for a consent
    pub fn audit_log(
        env: Env,
        consent_id: u64,
    ) -> Vec<audit::AuditEvent> {
        audit::get_audit_log(&env, consent_id)
    }

    /// Get access logs for a consent
    pub fn get_access_logs(
        env: Env,
        consent_id: u64,
    ) -> Vec<audit::ConsentAccessLog> {
        audit::get_access_logs(&env, consent_id)
    }

    /// Log consent access
    pub fn log_access(
        env: Env,
        consent_id: u64,
        accessed_by: Address,
        data_scope: consent::DataScope,
        purpose: String,
    ) {
        audit::log_consent_accessed(&env, consent_id, accessed_by, data_scope, purpose);
    }

    /// Get audit summary for a patient's consent
    pub fn get_audit_summary(
        env: Env,
        patient: Address,
        consent_id: u64,
    ) -> Option<audit::AuditSummary> {
        audit::get_patient_audit_summary(&env, patient, consent_id)
    }

    /// Validate consent parameters before creation
    pub fn validate_consent_params(
        env: Env,
        purpose: String,
        expires_at: Option<u64>,
    ) -> bool {
        utils::validate_consent_parameters(&env, &purpose, &expires_at).is_ok()
    }

    /// Check if consent is expired
    pub fn is_expired(
        env: Env,
        consent_id: u64,
    ) -> bool {
        if let Some(consent) = consent::get_consent(&env, consent_id) {
            utils::is_consent_expired(&env, &consent)
        } else {
            false
        }
    }

    /// Get remaining validity period for consent
    pub fn get_remaining_validity(
        env: Env,
        consent_id: u64,
    ) -> Option<u64> {
        let consent = consent::get_consent(&env, consent_id)?;
        utils::get_remaining_validity(&env, &consent)
    }

    /// Check if consent is expiring soon
    pub fn is_expiring_soon(
        env: Env,
        consent_id: u64,
        warning_threshold: u64,
    ) -> bool {
        if let Some(consent) = consent::get_consent(&env, consent_id) {
            utils::is_expiring_soon(&env, &consent, warning_threshold)
        } else {
            false
        }
    }

    /// Get consent age in seconds
    pub fn get_consent_age(
        env: Env,
        consent_id: u64,
    ) -> u64 {
        if let Some(consent) = consent::get_consent(&env, consent_id) {
            utils::get_consent_age(&env, &consent)
        } else {
            0
        }
    }

    /// Check GDPR compliance for a consent
    pub fn check_gdpr_compliance(
        env: Env,
        consent_id: u64,
    ) -> bool {
        if let Some(consent) = consent::get_consent(&env, consent_id) {
            utils::check_gdpr_compliance(&env, &consent).is_ok()
        } else {
            false
        }
    }

    /// Check HIPAA compliance for a consent
    pub fn check_hipaa_compliance(
        env: Env,
        consent_id: u64,
    ) -> bool {
        if let Some(consent) = consent::get_consent(&env, consent_id) {
            utils::check_hipaa_compliance(&env, &consent).is_ok()
        } else {
            false
        }
    }

    /// Calculate days until expiration
    pub fn days_until_expiration(
        env: Env,
        consent_id: u64,
    ) -> Option<u64> {
        let consent = consent::get_consent(&env, consent_id)?;
        utils::days_until_expiration(&env, &consent)
    }
}

#[cfg(test)]
mod test;
