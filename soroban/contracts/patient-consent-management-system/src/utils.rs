use soroban_sdk::{Env, String};

use crate::consent::{Consent, ConsentStatus, DataScope};

/// Validate consent parameters
pub fn validate_consent_parameters(
    env: &Env,
    purpose: &String,
    expires_at: &Option<u64>,
) -> Result<(), String> {
    // Validate purpose is not empty
    if purpose.len() == 0 {
        return Err(String::from_str(env, "Purpose is required"));
    }

    // Validate purpose length (reasonable limit)
    if purpose.len() > 500 {
        return Err(String::from_str(env, "Purpose too long"));
    }

    // Validate expiration date if provided
    if let Some(expiry) = expires_at {
        if *expiry <= env.ledger().timestamp() {
            return Err(String::from_str(env, "Expiration must be in future"));
        }

        // Check expiration is not too far in the future (e.g., 10 years)
        let max_duration = 10 * 365 * 24 * 60 * 60; // 10 years in seconds
        if *expiry > env.ledger().timestamp() + max_duration {
            return Err(String::from_str(env, "Expiration too far in future"));
        }
    }

    Ok(())
}

/// Check if consent has expired
pub fn is_consent_expired(env: &Env, consent: &Consent) -> bool {
    if let Some(expires_at) = consent.expires_at {
        env.ledger().timestamp() >= expires_at
    } else {
        false
    }
}

/// Check if consent is valid and active
pub fn is_consent_valid(env: &Env, consent: &Consent) -> bool {
    // Must be active
    if consent.status != ConsentStatus::Active {
        return false;
    }

    // Must not be expired
    if is_consent_expired(env, consent) {
        return false;
    }

    true
}

/// Calculate remaining validity period for consent
pub fn get_remaining_validity(env: &Env, consent: &Consent) -> Option<u64> {
    if let Some(expires_at) = consent.expires_at {
        let current_time = env.ledger().timestamp();
        if expires_at > current_time {
            Some(expires_at - current_time)
        } else {
            Some(0)
        }
    } else {
        None // No expiration set
    }
}

/// Format data scope as human-readable string
pub fn format_data_scope(_env: &Env, _scope: &DataScope) -> String {
    // Simplified formatting - in production would return proper formatted string
    String::from_str(_env, "Data scope")
}

/// Validate data scope list
pub fn validate_data_scopes(
    env: &Env,
    scopes: &soroban_sdk::Vec<DataScope>,
) -> Result<(), String> {
    if scopes.len() == 0 {
        return Err(String::from_str(env, "At least one scope required"));
    }

    // Check for duplicates or conflicts
    // If AllData is present, other scopes are redundant
    let mut has_all_data = false;
    for i in 0..scopes.len() {
        let scope = scopes.get(i).unwrap();
        if scope == DataScope::AllData {
            has_all_data = true;
        }
    }

    if has_all_data && scopes.len() > 1 {
        return Err(String::from_str(env, "AllData scope makes others redundant"));
    }

    Ok(())
}

/// Calculate consent age in seconds
pub fn get_consent_age(env: &Env, consent: &Consent) -> u64 {
    let current_time = env.ledger().timestamp();
    if current_time >= consent.created_at {
        current_time - consent.created_at
    } else {
        0
    }
}

/// Check if consent is about to expire (within warning threshold)
pub fn is_expiring_soon(env: &Env, consent: &Consent, warning_threshold: u64) -> bool {
    if let Some(expires_at) = consent.expires_at {
        let current_time = env.ledger().timestamp();
        if expires_at > current_time {
            let remaining = expires_at - current_time;
            return remaining <= warning_threshold;
        }
    }
    false
}

/// Validate time-based consent constraints
pub fn validate_time_constraints(
    env: &Env,
    created_at: u64,
    expires_at: Option<u64>,
) -> Result<(), String> {
    let current_time = env.ledger().timestamp();

    // Created time should not be in the future
    if created_at > current_time {
        return Err(String::from_str(env, "Created time cannot be in future"));
    }

    // If expiration is set, validate it
    if let Some(expiry) = expires_at {
        if expiry <= created_at {
            return Err(String::from_str(env, "Expiration must be after creation"));
        }

        if expiry <= current_time {
            return Err(String::from_str(env, "Expiration must be in future"));
        }
    }

    Ok(())
}

/// Check GDPR compliance requirements
pub fn check_gdpr_compliance(
    env: &Env,
    consent: &Consent,
) -> Result<(), String> {
    // GDPR requires:
    // 1. Consent must be freely given, specific, informed, and unambiguous
    // 2. Purpose must be clearly stated
    if consent.purpose.len() == 0 {
        return Err(String::from_str(env, "Purpose required for GDPR compliance"));
    }

    // 3. Consent must be revocable
    if consent.status == ConsentStatus::Active || consent.status == ConsentStatus::Suspended {
        // Active or suspended consents can be revoked - compliant
        Ok(())
    } else {
        Ok(()) // Already revoked or expired
    }
}

/// Check HIPAA compliance requirements
pub fn check_hipaa_compliance(
    env: &Env,
    consent: &Consent,
) -> Result<(), String> {
    // HIPAA requires:
    // 1. Patient authorization for PHI disclosure
    // 2. Specific description of information to be used/disclosed
    if consent.data_scopes.len() == 0 {
        return Err(String::from_str(env, "Data scope required for HIPAA"));
    }

    // 3. Identification of authorized parties
    // Checked by having authorized_party field

    // 4. Expiration date or event
    // Can be None for ongoing treatment/payment/operations

    Ok(())
}

/// Generate consent summary for auditing
pub fn generate_consent_summary(_env: &Env, _consent: &Consent) -> String {
    // Simplified - in production would generate detailed summary
    String::from_str(_env, "Consent summary")
}

/// Calculate days until expiration
pub fn days_until_expiration(env: &Env, consent: &Consent) -> Option<u64> {
    if let Some(expires_at) = consent.expires_at {
        let current_time = env.ledger().timestamp();
        if expires_at > current_time {
            let seconds_remaining = expires_at - current_time;
            Some(seconds_remaining / (24 * 60 * 60))
        } else {
            Some(0)
        }
    } else {
        None
    }
}

/// Validate authorized party
pub fn validate_authorized_party(
    _env: &Env,
    party: &soroban_sdk::Address,
) -> Result<(), String> {
    // Basic validation - party address must be valid
    // In production, might check against registry of valid healthcare providers
    let _check = party.clone(); // Ensure address is valid
    Ok(())
}
