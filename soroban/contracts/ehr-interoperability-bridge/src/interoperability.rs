use soroban_sdk::{Address, BytesN, Env, String, Vec, Symbol};
use crate::{DataKey, DataRequest, ConsentRecord, RequestStatus};

/// Verify patient consent for data request
pub fn verify_consent(
    env: &Env,
    patient: Address,
    request_id: BytesN<32>,
    consent_signature: BytesN<64>,
) -> bool {
    patient.require_auth();

    // Get the data request
    let mut request: DataRequest = env.storage()
        .persistent()
        .get(&DataKey::DataRequest(request_id.clone()))
        .expect("Request not found");

    // Get patient consent record
    let consent: ConsentRecord = env.storage()
        .persistent()
        .get(&DataKey::ConsentRecord(request.patient_id.clone()))
        .expect("Patient consent record not found");

    // Verify consent is not revoked and not expired
    if consent.revoked {
        panic!("Patient consent has been revoked");
    }

    if env.ledger().timestamp() > consent.consent_expiry {
        panic!("Patient consent has expired");
    }

    // Verify patient identity
    if consent.patient_address != patient {
        panic!("Patient identity mismatch");
    }

    // Check if both systems are authorized
    let sender_authorized = consent.authorized_systems.contains(&request.sender_system);
    let receiver_authorized = consent.authorized_systems.contains(&request.receiver_system);

    if !sender_authorized || !receiver_authorized {
        panic!("One or both systems not authorized by patient");
    }

    // Check if requested data types are permitted
    for data_type in request.data_types.iter() {
        if !consent.data_types_permitted.contains(&data_type) {
            panic!("Data type not permitted by patient consent");
        }
    }

    // Verify consent signature (simplified - in production would use cryptographic verification)
    let _expected_message = "consent_verification";
    
    // In a real implementation, this would verify the signature cryptographically
    // For now, we'll assume the signature is valid if provided
    if consent_signature.to_array().len() != 64 {
        panic!("Invalid consent signature format");
    }

    // Update request to mark consent as verified
    request.consent_verified = true;
    request.status = RequestStatus::ConsentVerified;
    env.storage().persistent().set(&DataKey::DataRequest(request_id.clone()), &request);

    // Emit consent verification event
    env.events().publish(
        (Symbol::new(env, "consent_verified"),),
        (request_id, patient, env.ledger().timestamp()),
    );

    true
}

/// Set patient consent for data sharing
pub fn set_patient_consent(
    env: &Env,
    patient: Address,
    patient_id: String,
    authorized_systems: Vec<String>,
    data_types_permitted: Vec<String>,
    consent_duration_hours: u64,
) -> bool {
    patient.require_auth();

    let current_time = env.ledger().timestamp();
    let consent_expiry = current_time + (consent_duration_hours * 3600);

    let consent_record = ConsentRecord {
        patient_id: patient_id.clone(),
        patient_address: patient.clone(),
        authorized_systems,
        data_types_permitted,
        consent_expiry,
        revoked: false,
    };

    // Store consent record
    env.storage().persistent().set(&DataKey::ConsentRecord(patient_id.clone()), &consent_record);

    // Emit consent set event
    env.events().publish(
        (Symbol::new(env, "consent_set"),),
        (patient_id, patient, consent_expiry),
    );

    true
}

/// Revoke patient consent
pub fn revoke_consent(env: &Env, patient: Address, patient_id: String) -> bool {
    patient.require_auth();

    // Get existing consent record
    let mut consent: ConsentRecord = env.storage()
        .persistent()
        .get(&DataKey::ConsentRecord(patient_id.clone()))
        .expect("Consent record not found");

    // Verify patient identity
    if consent.patient_address != patient {
        panic!("Unauthorized to revoke consent");
    }

    // Revoke consent
    consent.revoked = true;
    env.storage().persistent().set(&DataKey::ConsentRecord(patient_id.clone()), &consent);

    // Emit consent revocation event
    env.events().publish(
        (Symbol::new(env, "consent_revoked"),),
        (patient_id, patient, env.ledger().timestamp()),
    );

    true
}

/// Check if patient has given consent for specific data sharing
pub fn check_patient_consent(
    env: &Env,
    patient_id: String,
    sender_system: String,
    receiver_system: String,
    data_types: Vec<String>,
) -> bool {
    // Get patient consent record
    let consent: ConsentRecord = match env.storage()
        .persistent()
        .get(&DataKey::ConsentRecord(patient_id)) {
        Some(record) => record,
        None => return false,
    };

    // Check if consent is revoked or expired
    if consent.revoked || env.ledger().timestamp() > consent.consent_expiry {
        return false;
    }

    // Check if both systems are authorized
    if !consent.authorized_systems.contains(&sender_system) 
        || !consent.authorized_systems.contains(&receiver_system) {
        return false;
    }

    // Check if all requested data types are permitted
    for data_type in data_types.iter() {
        if !consent.data_types_permitted.contains(&data_type) {
            return false;
        }
    }

    true
}

/// Get patient consent details
pub fn get_patient_consent(env: &Env, patient_id: String) -> Option<ConsentRecord> {
    env.storage().persistent().get(&DataKey::ConsentRecord(patient_id))
}

/// Validate FHIR compliance for data request
pub fn validate_fhir_compliance(
    env: &Env,
    request_id: BytesN<32>,
    fhir_resource_types: Vec<String>,
) -> bool {
    let _request: DataRequest = env.storage()
        .persistent()
        .get(&DataKey::DataRequest(request_id.clone()))
        .expect("Request not found");

    // Define valid FHIR resource types
    let mut valid_fhir_types = Vec::new(env);
    valid_fhir_types.push_back(String::from_str(env, "Patient"));
    valid_fhir_types.push_back(String::from_str(env, "Observation"));
    valid_fhir_types.push_back(String::from_str(env, "Condition"));
    valid_fhir_types.push_back(String::from_str(env, "Medication"));
    valid_fhir_types.push_back(String::from_str(env, "Procedure"));
    valid_fhir_types.push_back(String::from_str(env, "DiagnosticReport"));
    valid_fhir_types.push_back(String::from_str(env, "Encounter"));
    valid_fhir_types.push_back(String::from_str(env, "AllergyIntolerance"));
    valid_fhir_types.push_back(String::from_str(env, "Immunization"));

    // Validate that all requested FHIR resource types are valid
    for resource_type in fhir_resource_types.iter() {
        if !valid_fhir_types.contains(&resource_type) {
            return false;
        }
    }

    // Emit FHIR validation event
    env.events().publish(
        (Symbol::new(env, "fhir_validated"),),
        (request_id, fhir_resource_types.len()),
    );

    true
}

/// Validate SMART on FHIR authorization
pub fn validate_smart_auth(
    env: &Env,
    requester: Address,
    patient_id: String,
    scope: String,
    access_token_hash: BytesN<32>,
) -> bool {
    requester.require_auth();

    // Validate scope format (simplified)
    let mut valid_scopes = Vec::new(env);
    valid_scopes.push_back(String::from_str(env, "patient/*.read"));
    valid_scopes.push_back(String::from_str(env, "patient/Patient.read"));
    valid_scopes.push_back(String::from_str(env, "patient/Observation.read"));
    valid_scopes.push_back(String::from_str(env, "user/*.read"));
    valid_scopes.push_back(String::from_str(env, "system/*.read"));

    if !valid_scopes.contains(&scope) {
        return false;
    }

    // In a real implementation, this would validate the OAuth2/SMART token
    // For now, we check that a token hash is provided
    if access_token_hash.to_array().len() != 32 {
        return false;
    }

    // Emit SMART authorization event
    env.events().publish(
        (Symbol::new(env, "smart_auth_validated"),),
        (requester, patient_id, scope),
    );

    true
}