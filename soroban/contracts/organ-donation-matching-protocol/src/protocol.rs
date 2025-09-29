use crate::{
    BloodType, Config, ContractError, DataKey, DonorProfile, HLAProfile, MatchResult, OrganType,
    RecipientProfile, UrgencyLevel,
};
use soroban_sdk::{Address, Env, String};

use crate::utils::generate_match_id;

/// Validate donor registration data
pub fn validate_donor_data(
    _donor_address: &Address,
    _blood_type: &BloodType,
    _organ_type: &OrganType,
    hla_profile: &HLAProfile,
    age: u32,
    _medical_facility: &Address,
    consent_hash: &String,
) -> Result<(), ContractError> {
    // Age validation
    if age < 18 || age > 80 {
        return Err(ContractError::InvalidMedicalFacility); // Using closest error
    }

    // Consent validation
    if consent_hash.len() < 32 {
        return Err(ContractError::ConsentNotProvided);
    }

    // HLA profile validation - ensure at least one allele per locus
    if hla_profile.hla_a.is_empty() || hla_profile.hla_b.is_empty() || hla_profile.hla_dr.is_empty()
    {
        return Err(ContractError::InvalidMedicalFacility); // Using closest error
    }

    Ok(())
}

/// Validate recipient registration data
pub fn validate_recipient_data(
    _recipient_address: &Address,
    _blood_type: &BloodType,
    _organ_type: &OrganType,
    hla_profile: &HLAProfile,
    age: u32,
    _urgency_level: &UrgencyLevel,
    _medical_facility: &Address,
    medical_priority_score: u32,
) -> Result<(), ContractError> {
    // Age validation
    if age < 1 || age > 85 {
        return Err(ContractError::InvalidMedicalFacility); // Using closest error
    }

    // Medical priority score validation
    if medical_priority_score > 1000 {
        return Err(ContractError::InvalidUrgencyLevel);
    }

    // HLA profile validation
    if hla_profile.hla_a.is_empty() || hla_profile.hla_b.is_empty() || hla_profile.hla_dr.is_empty()
    {
        return Err(ContractError::InvalidMedicalFacility); // Using closest error
    }

    Ok(())
}

/// Deactivate matched donor and recipient profiles
pub fn deactivate_matched_profiles(
    env: &Env,
    donor_address: &Address,
    recipient_address: &Address,
) -> Result<(), ContractError> {
    // Deactivate donor
    let mut donor: DonorProfile = env
        .storage()
        .persistent()
        .get(&DataKey::Donor(donor_address.clone()))
        .ok_or(ContractError::DonorNotFound)?;

    donor.is_active = false;
    env.storage()
        .persistent()
        .set(&DataKey::Donor(donor_address.clone()), &donor);

    // Deactivate recipient
    let mut recipient: RecipientProfile = env
        .storage()
        .persistent()
        .get(&DataKey::Recipient(recipient_address.clone()))
        .ok_or(ContractError::RecipientNotFound)?;

    recipient.is_active = false;
    env.storage()
        .persistent()
        .set(&DataKey::Recipient(recipient_address.clone()), &recipient);

    Ok(())
}

/// Validate organ-specific medical criteria
pub fn validate_organ_specific_criteria(
    donor: &DonorProfile,
    recipient: &RecipientProfile,
    organ_type: &OrganType,
) -> Result<bool, ContractError> {
    match organ_type {
        OrganType::Kidney => validate_kidney_criteria(donor, recipient),
        OrganType::Liver => validate_liver_criteria(donor, recipient),
        OrganType::Heart => validate_heart_criteria(donor, recipient),
        OrganType::Lung => validate_lung_criteria(donor, recipient),
        OrganType::Pancreas => validate_pancreas_criteria(donor, recipient),
        OrganType::Intestine => validate_intestine_criteria(donor, recipient),
    }
}

/// Validate kidney-specific matching criteria
fn validate_kidney_criteria(
    donor: &DonorProfile,
    recipient: &RecipientProfile,
) -> Result<bool, ContractError> {
    // Age considerations for kidney transplant
    let age_diff = if donor.age > recipient.age {
        donor.age - recipient.age
    } else {
        recipient.age - donor.age
    };

    // Kidney transplants are more tolerant of age differences
    if age_diff > 20 && recipient.urgency_level != UrgencyLevel::Critical {
        return Ok(false);
    }

    Ok(true)
}

/// Validate liver-specific matching criteria
fn validate_liver_criteria(
    donor: &DonorProfile,
    recipient: &RecipientProfile,
) -> Result<bool, ContractError> {
    // Liver transplants are time-sensitive and have different age considerations
    let age_compatible = donor.age <= 70 && recipient.age <= 70;

    if !age_compatible && recipient.urgency_level != UrgencyLevel::Critical {
        return Ok(false);
    }

    Ok(true)
}

/// Validate heart-specific matching criteria
fn validate_heart_criteria(
    donor: &DonorProfile,
    recipient: &RecipientProfile,
) -> Result<bool, ContractError> {
    // Heart transplants require stricter age and size matching
    let age_diff = if donor.age > recipient.age {
        donor.age - recipient.age
    } else {
        recipient.age - donor.age
    };

    if age_diff > 15 {
        return Ok(false);
    }

    Ok(true)
}

/// Validate lung-specific matching criteria
fn validate_lung_criteria(
    donor: &DonorProfile,
    recipient: &RecipientProfile,
) -> Result<bool, ContractError> {
    // Lung transplants require very strict matching
    let age_diff = if donor.age > recipient.age {
        donor.age - recipient.age
    } else {
        recipient.age - donor.age
    };

    if age_diff > 10 {
        return Ok(false);
    }

    Ok(true)
}

/// Validate pancreas-specific matching criteria
fn validate_pancreas_criteria(
    donor: &DonorProfile,
    recipient: &RecipientProfile,
) -> Result<bool, ContractError> {
    // Pancreas transplants typically for diabetes patients
    if donor.age > 50 && recipient.urgency_level == UrgencyLevel::Low {
        return Ok(false);
    }

    Ok(true)
}

/// Validate intestine-specific matching criteria
fn validate_intestine_criteria(
    donor: &DonorProfile,
    recipient: &RecipientProfile,
) -> Result<bool, ContractError> {
    // Intestinal transplants are rare and require specialized matching
    let age_diff = if donor.age > recipient.age {
        donor.age - recipient.age
    } else {
        recipient.age - donor.age
    };

    if age_diff > 15 {
        return Ok(false);
    }

    Ok(true)
}

/// Check consent validity
pub fn verify_consent(
    env: &Env,
    donor_address: &Address,
    consent_hash: &String,
) -> Result<bool, ContractError> {
    let donor: DonorProfile = env
        .storage()
        .persistent()
        .get(&DataKey::Donor(donor_address.clone()))
        .ok_or(ContractError::DonorNotFound)?;

    Ok(donor.consent_hash == *consent_hash)
}

/// Emergency override function for critical cases
pub fn emergency_override_match(
    env: &Env,
    admin: &Address,
    donor_address: &Address,
    recipient_address: &Address,
    _justification: &String,
) -> Result<MatchResult, ContractError> {
    admin.require_auth();

    let config: Config = env
        .storage()
        .instance()
        .get(&DataKey::Config)
        .ok_or(ContractError::NotInitialized)?;

    if *admin != config.admin {
        return Err(ContractError::NotAuthorized);
    }

    let donor: DonorProfile = env
        .storage()
        .persistent()
        .get(&DataKey::Donor(donor_address.clone()))
        .ok_or(ContractError::DonorNotFound)?;

    let recipient: RecipientProfile = env
        .storage()
        .persistent()
        .get(&DataKey::Recipient(recipient_address.clone()))
        .ok_or(ContractError::RecipientNotFound)?;

    // Ensure both profiles are active
    if !donor.is_active || !recipient.is_active {
        return Err(ContractError::InactiveProfile);
    }

    // Create emergency match with high priority
    let match_id = generate_match_id(env);
    let match_result = MatchResult {
        match_id,
        donor: donor_address.clone(),
        recipient: recipient_address.clone(),
        compatibility_score: 999, // Emergency override score
        priority_score: 9999,     // Highest priority
        matched_at: env.ledger().timestamp(),
        confirmed: false,
        medical_facility: recipient.medical_facility.clone(),
    };

    env.storage()
        .persistent()
        .set(&DataKey::Match(match_id), &match_result);

    Ok(match_result)
}
