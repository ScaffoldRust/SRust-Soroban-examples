use soroban_sdk::{contracttype, symbol_short, Address, Env, String, Symbol, Vec};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConsentStatus {
    Active,      // Consent is currently active
    Revoked,     // Consent has been revoked by patient
    Expired,     // Consent has expired (time-bound)
    Suspended,   // Temporarily suspended
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataScope {
    Diagnostics,      // Diagnostic test results
    Imaging,          // Medical imaging (X-rays, MRI, etc.)
    LabResults,       // Laboratory test results
    Prescriptions,    // Medication and prescription data
    Research,         // Data for research purposes
    Treatment,        // Treatment and procedure data
    AllData,          // All medical data
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Consent {
    pub consent_id: u64,
    pub patient: Address,
    pub authorized_party: Address,
    pub data_scopes: Vec<DataScope>,
    pub purpose: String,
    pub status: ConsentStatus,
    pub created_at: u64,
    pub expires_at: Option<u64>,
    pub revoked_at: Option<u64>,
    pub last_updated: u64,
}

// Storage keys
const CONSENTS: Symbol = symbol_short!("CONSENTS");
const PATIENT_CONSENTS: Symbol = symbol_short!("PAT_CONS");
const PARTY_CONSENTS: Symbol = symbol_short!("PTY_CONS");
pub const NEXT_CONSENT_ID: Symbol = symbol_short!("NEXT_CID");

pub fn create_consent(
    env: &Env,
    patient: Address,
    authorized_party: Address,
    data_scopes: Vec<DataScope>,
    purpose: String,
    expires_at: Option<u64>,
) -> u64 {
    // Validate inputs
    patient.require_auth();

    if data_scopes.len() == 0 {
        panic!("At least one data scope required");
    }

    if purpose.len() == 0 {
        panic!("Purpose is required");
    }

    // Validate expiration date if provided
    if let Some(expiry) = expires_at {
        if expiry <= env.ledger().timestamp() {
            panic!("Expiration must be in the future");
        }
    }

    // Get next consent ID
    let consent_id = get_next_consent_id(env);

    let consent = Consent {
        consent_id,
        patient: patient.clone(),
        authorized_party: authorized_party.clone(),
        data_scopes,
        purpose,
        status: ConsentStatus::Active,
        created_at: env.ledger().timestamp(),
        expires_at,
        revoked_at: None,
        last_updated: env.ledger().timestamp(),
    };

    // Store consent
    env.storage()
        .persistent()
        .set(&storage_key_consent(consent_id), &consent);

    // Add to patient's consent list
    add_patient_consent(env, &patient, consent_id);

    // Add to authorized party's consent list
    add_party_consent(env, &authorized_party, consent_id);

    consent_id
}

pub fn update_consent(
    env: &Env,
    consent_id: u64,
    patient: Address,
    new_data_scopes: Option<Vec<DataScope>>,
    new_purpose: Option<String>,
    new_expires_at: Option<Option<u64>>,
) {
    patient.require_auth();

    let mut consent = get_consent(env, consent_id)
        .unwrap_or_else(|| panic!("Consent not found"));

    // Verify patient owns this consent
    if consent.patient != patient {
        panic!("Unauthorized: not consent owner");
    }

    // Can only update active consents
    if consent.status != ConsentStatus::Active {
        panic!("Can only update active consents");
    }

    // Update fields if provided
    if let Some(scopes) = new_data_scopes {
        if scopes.len() == 0 {
            panic!("At least one data scope required");
        }
        consent.data_scopes = scopes;
    }

    if let Some(purpose) = new_purpose {
        if purpose.len() == 0 {
            panic!("Purpose cannot be empty");
        }
        consent.purpose = purpose;
    }

    if let Some(expiry) = new_expires_at {
        if let Some(exp_time) = expiry {
            if exp_time <= env.ledger().timestamp() {
                panic!("Expiration must be in the future");
            }
        }
        consent.expires_at = expiry;
    }

    consent.last_updated = env.ledger().timestamp();

    // Store updated consent
    env.storage()
        .persistent()
        .set(&storage_key_consent(consent_id), &consent);
}

pub fn revoke_consent(
    env: &Env,
    consent_id: u64,
    patient: Address,
) {
    patient.require_auth();

    let mut consent = get_consent(env, consent_id)
        .unwrap_or_else(|| panic!("Consent not found"));

    // Verify patient owns this consent
    if consent.patient != patient {
        panic!("Unauthorized: not consent owner");
    }

    // Can only revoke active or suspended consents
    if consent.status != ConsentStatus::Active && consent.status != ConsentStatus::Suspended {
        panic!("Can only revoke active or suspended consents");
    }

    consent.status = ConsentStatus::Revoked;
    consent.revoked_at = Some(env.ledger().timestamp());
    consent.last_updated = env.ledger().timestamp();

    // Store updated consent
    env.storage()
        .persistent()
        .set(&storage_key_consent(consent_id), &consent);
}

pub fn suspend_consent(
    env: &Env,
    consent_id: u64,
    patient: Address,
) {
    patient.require_auth();

    let mut consent = get_consent(env, consent_id)
        .unwrap_or_else(|| panic!("Consent not found"));

    // Verify patient owns this consent
    if consent.patient != patient {
        panic!("Unauthorized: not consent owner");
    }

    // Can only suspend active consents
    if consent.status != ConsentStatus::Active {
        panic!("Can only suspend active consents");
    }

    consent.status = ConsentStatus::Suspended;
    consent.last_updated = env.ledger().timestamp();

    // Store updated consent
    env.storage()
        .persistent()
        .set(&storage_key_consent(consent_id), &consent);
}

pub fn resume_consent(
    env: &Env,
    consent_id: u64,
    patient: Address,
) {
    patient.require_auth();

    let mut consent = get_consent(env, consent_id)
        .unwrap_or_else(|| panic!("Consent not found"));

    // Verify patient owns this consent
    if consent.patient != patient {
        panic!("Unauthorized: not consent owner");
    }

    // Can only resume suspended consents
    if consent.status != ConsentStatus::Suspended {
        panic!("Can only resume suspended consents");
    }

    // Check if consent has expired
    if let Some(expires_at) = consent.expires_at {
        if env.ledger().timestamp() >= expires_at {
            consent.status = ConsentStatus::Expired;
            consent.last_updated = env.ledger().timestamp();
            env.storage()
                .persistent()
                .set(&storage_key_consent(consent_id), &consent);
            panic!("Cannot resume expired consent");
        }
    }

    consent.status = ConsentStatus::Active;
    consent.last_updated = env.ledger().timestamp();

    // Store updated consent
    env.storage()
        .persistent()
        .set(&storage_key_consent(consent_id), &consent);
}

pub fn check_consent(
    env: &Env,
    consent_id: u64,
    authorized_party: Address,
    data_scope: DataScope,
) -> bool {
    let consent = match get_consent(env, consent_id) {
        Some(c) => c,
        None => return false,
    };

    // Check if party matches
    if consent.authorized_party != authorized_party {
        return false;
    }

    // Check status
    if consent.status != ConsentStatus::Active {
        return false;
    }

    // Check expiration
    if let Some(expires_at) = consent.expires_at {
        if env.ledger().timestamp() >= expires_at {
            return false;
        }
    }

    // Check data scope
    check_data_scope(&consent.data_scopes, &data_scope)
}

pub fn check_consent_active(
    env: &Env,
    consent_id: u64,
) -> bool {
    let mut consent = match get_consent(env, consent_id) {
        Some(c) => c,
        None => return false,
    };

    // Check if expired
    if consent.status == ConsentStatus::Active {
        if let Some(expires_at) = consent.expires_at {
            if env.ledger().timestamp() >= expires_at {
                // Auto-expire the consent
                consent.status = ConsentStatus::Expired;
                consent.last_updated = env.ledger().timestamp();
                env.storage()
                    .persistent()
                    .set(&storage_key_consent(consent_id), &consent);
                return false;
            }
        }
    }

    consent.status == ConsentStatus::Active
}

pub fn get_consent(env: &Env, consent_id: u64) -> Option<Consent> {
    env.storage()
        .persistent()
        .get(&storage_key_consent(consent_id))
}

pub fn get_patient_consents(env: &Env, patient: Address) -> Vec<u64> {
    env.storage()
        .persistent()
        .get(&storage_key_patient_consents(&patient))
        .unwrap_or_else(|| Vec::new(&env))
}

pub fn get_party_consents(env: &Env, party: Address) -> Vec<u64> {
    env.storage()
        .persistent()
        .get(&storage_key_party_consents(&party))
        .unwrap_or_else(|| Vec::new(&env))
}

fn check_data_scope(consent_scopes: &Vec<DataScope>, requested_scope: &DataScope) -> bool {
    // Check if AllData is granted
    for i in 0..consent_scopes.len() {
        let scope = consent_scopes.get(i).unwrap();
        if scope == DataScope::AllData {
            return true;
        }
        if &scope == requested_scope {
            return true;
        }
    }
    false
}

fn get_next_consent_id(env: &Env) -> u64 {
    let current_id: u64 = env
        .storage()
        .instance()
        .get(&NEXT_CONSENT_ID)
        .unwrap_or(1);

    env.storage().instance().set(&NEXT_CONSENT_ID, &(current_id + 1));

    current_id
}

fn add_patient_consent(env: &Env, patient: &Address, consent_id: u64) {
    let mut patient_consents = get_patient_consents(env, patient.clone());
    patient_consents.push_back(consent_id);
    env.storage()
        .persistent()
        .set(&storage_key_patient_consents(patient), &patient_consents);
}

fn add_party_consent(env: &Env, party: &Address, consent_id: u64) {
    let mut party_consents = get_party_consents(env, party.clone());
    party_consents.push_back(consent_id);
    env.storage()
        .persistent()
        .set(&storage_key_party_consents(party), &party_consents);
}

pub fn storage_key_consent(consent_id: u64) -> (Symbol, u64) {
    (CONSENTS, consent_id)
}

fn storage_key_patient_consents(patient: &Address) -> (Symbol, Address) {
    (PATIENT_CONSENTS, patient.clone())
}

fn storage_key_party_consents(party: &Address) -> (Symbol, Address) {
    (PARTY_CONSENTS, party.clone())
}
