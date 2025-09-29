// utils.rs - Utility Functions for Organ Donation Matching Protocol
// Provides helper functions for validation, formatting, and data processing

use crate::{
    BloodType, ContractError, DonorProfile, HLAProfile, MatchResult, OrganType, RecipientProfile,
    UrgencyLevel,
};
use soroban_sdk::{Env, String};

/// Convert blood type to string for logging and reporting
pub fn blood_type_to_string(env: &Env, blood_type: &BloodType) -> String {
    match blood_type {
        BloodType::A => String::from_str(env, "A"),
        BloodType::B => String::from_str(env, "B"),
        BloodType::AB => String::from_str(env, "AB"),
        BloodType::O => String::from_str(env, "O"),
    }
}

/// Convert organ type to string for logging and reporting
pub fn organ_type_to_string(env: &Env, organ_type: &OrganType) -> String {
    match organ_type {
        OrganType::Kidney => String::from_str(env, "Kidney"),
        OrganType::Liver => String::from_str(env, "Liver"),
        OrganType::Heart => String::from_str(env, "Heart"),
        OrganType::Lung => String::from_str(env, "Lung"),
        OrganType::Pancreas => String::from_str(env, "Pancreas"),
        OrganType::Intestine => String::from_str(env, "Intestine"),
    }
}

/// Convert urgency level to string for logging and reporting
pub fn urgency_level_to_string(env: &Env, urgency_level: &UrgencyLevel) -> String {
    match urgency_level {
        UrgencyLevel::Low => String::from_str(env, "Low"),
        UrgencyLevel::Medium => String::from_str(env, "Medium"),
        UrgencyLevel::High => String::from_str(env, "High"),
        UrgencyLevel::Critical => String::from_str(env, "Critical"),
    }
}

/// Validate HLA allele format
/// HLA alleles typically follow patterns like "A*02:01" or "DRB1*15:01"
pub fn validate_hla_allele(allele: &String) -> bool {
    // Basic validation - HLA alleles should be 5-10 characters
    let len = allele.len();
    if len < 5 || len > 10 {
        return false;
    }

    true
}

/// Create a comprehensive HLA profile validation
/// Ensures all required loci have valid alleles
pub fn validate_hla_profile(hla_profile: &HLAProfile) -> Result<(), ContractError> {
    // Check if all required loci have at least one allele
    if hla_profile.hla_a.is_empty() {
        return Err(ContractError::InvalidMedicalFacility);
    }
    if hla_profile.hla_b.is_empty() {
        return Err(ContractError::InvalidMedicalFacility);
    }
    if hla_profile.hla_dr.is_empty() {
        return Err(ContractError::InvalidMedicalFacility);
    }

    // Validate each allele format
    for allele in hla_profile.hla_a.iter() {
        if !validate_hla_allele(&allele) {
            return Err(ContractError::InvalidMedicalFacility);
        }
    }
    for allele in hla_profile.hla_b.iter() {
        if !validate_hla_allele(&allele) {
            return Err(ContractError::InvalidMedicalFacility);
        }
    }
    for allele in hla_profile.hla_dr.iter() {
        if !validate_hla_allele(&allele) {
            return Err(ContractError::InvalidMedicalFacility);
        }
    }

    Ok(())
}

/// Calculate time-based urgency multiplier
/// Adjusts urgency based on how long patient has been waiting
pub fn calculate_time_urgency_multiplier(urgency_level: &UrgencyLevel, wait_time_days: u32) -> f64 {
    let base_multiplier = match urgency_level {
        UrgencyLevel::Critical => 4.0,
        UrgencyLevel::High => 3.0,
        UrgencyLevel::Medium => 2.0,
        UrgencyLevel::Low => 1.0,
    };

    // Increase multiplier based on wait time (max 2x additional)
    let time_factor = if wait_time_days > 365 {
        2.0 // Maximum time bonus after 1 year
    } else {
        1.0 + (wait_time_days as f64 / 365.0)
    };

    base_multiplier * time_factor
}

/// Validate age ranges for different organ types
/// Different organs have different acceptable age ranges for donors and recipients
pub fn validate_age_for_organ_type(
    age: u32,
    organ_type: &OrganType,
    is_donor: bool,
) -> Result<(), ContractError> {
    let (min_age, max_age) = if is_donor {
        // Donor age ranges
        match organ_type {
            OrganType::Kidney => (18, 75),
            OrganType::Liver => (18, 70),
            OrganType::Heart => (18, 65),
            OrganType::Lung => (18, 65),
            OrganType::Pancreas => (18, 55),
            OrganType::Intestine => (18, 60),
        }
    } else {
        // Recipient age ranges (typically wider)
        match organ_type {
            OrganType::Kidney => (1, 80),
            OrganType::Liver => (1, 75),
            OrganType::Heart => (1, 70),
            OrganType::Lung => (1, 70),
            OrganType::Pancreas => (1, 65),
            OrganType::Intestine => (1, 60),
        }
    };

    if age < min_age || age > max_age {
        return Err(ContractError::InvalidMedicalFacility);
    }

    Ok(())
}

/// Calculate organ viability time windows (in hours)
/// Different organs have different preservation times
pub fn get_organ_viability_hours(organ_type: &OrganType) -> u32 {
    match organ_type {
        OrganType::Heart => 4,     // 4-6 hours - most urgent
        OrganType::Lung => 6,      // 6-8 hours
        OrganType::Liver => 12,    // 12-18 hours
        OrganType::Kidney => 24,   // 24-36 hours - most flexible
        OrganType::Pancreas => 12, // 12-20 hours
        OrganType::Intestine => 8, // 8-12 hours
    }
}

/// Check if a match is still within viable time window
/// Returns true if the organ can still be successfully transplanted
pub fn is_match_viable(
    match_result: &MatchResult,
    organ_type: &OrganType,
    current_time: u64,
) -> bool {
    let viability_hours = get_organ_viability_hours(organ_type);
    let viability_seconds = viability_hours as u64 * 3600;
    let time_elapsed = current_time - match_result.matched_at;

    time_elapsed <= viability_seconds
}

/// Calculate statistical compatibility metrics based on population data
/// Returns (blood_type_frequency, organ_demand_score)
pub fn calculate_population_statistics(
    _env: &Env,
    blood_type: &BloodType,
    organ_type: &OrganType,
) -> (u32, u32) {
    // Blood type population frequencies (approximate global distribution)
    let blood_type_frequency = match blood_type {
        BloodType::O => 45, // 45% of population
        BloodType::A => 40, // 40% of population
        BloodType::B => 11, // 11% of population
        BloodType::AB => 4, // 4% of population
    };

    // Organ demand scores based on typical waiting lists
    let organ_demand_score = match organ_type {
        OrganType::Kidney => 85, // Highest demand
        OrganType::Liver => 75,
        OrganType::Heart => 60,
        OrganType::Lung => 50,
        OrganType::Pancreas => 30,
        OrganType::Intestine => 10, // Lowest demand (rarest)
    };

    (blood_type_frequency, organ_demand_score)
}

/// Validate consent string format
/// Ensures consent hashes meet minimum security requirements
pub fn validate_consent_format(consent_hash: &String) -> bool {
    // Consent hash should be at least 32 characters for security
    // Maximum 128 characters to prevent storage issues
    consent_hash.len() >= 32 && consent_hash.len() <= 128
}

/// Calculate risk score for a potential match
/// Lower scores indicate lower risk and better outcomes
pub fn calculate_risk_score(donor: &DonorProfile, recipient: &RecipientProfile) -> u32 {
    let mut risk_score = 0u32;

    // Age difference risk
    let age_diff = if donor.age > recipient.age {
        donor.age - recipient.age
    } else {
        recipient.age - donor.age
    };
    risk_score += age_diff / 5; // 1 point per 5 years difference

    // Blood type mismatch risk (even compatible mismatches have some risk)
    if donor.blood_type != recipient.blood_type {
        risk_score += 10;
    }

    // Organ-specific risk factors
    risk_score += match donor.organ_type {
        OrganType::Heart => 20,     // High complexity
        OrganType::Lung => 25,      // Highest complexity
        OrganType::Liver => 15,     // Moderate complexity
        OrganType::Kidney => 10,    // Lowest complexity
        OrganType::Pancreas => 18,  // Moderate-high complexity
        OrganType::Intestine => 30, // Very high complexity
    };

    risk_score
}

/// Check if donor and recipient are at the same medical facility
pub fn is_same_facility(donor: &DonorProfile, recipient: &RecipientProfile) -> bool {
    donor.medical_facility == recipient.medical_facility
}

/// Calculate compatibility percentage for display
/// Converts compatibility score to human-readable percentage
pub fn calculate_compatibility_percentage(compatibility_score: u32) -> u32 {
    // Compatibility score is already 0-100
    compatibility_score
}

/// Validate that all required profile fields are present and valid
pub fn validate_donor_profile_completeness(donor: &DonorProfile) -> Result<(), ContractError> {
    // Age validation
    if donor.age < 18 || donor.age > 80 {
        return Err(ContractError::InvalidMedicalFacility);
    }

    // Consent hash validation
    if !validate_consent_format(&donor.consent_hash) {
        return Err(ContractError::ConsentNotProvided);
    }

    // HLA profile validation
    validate_hla_profile(&donor.hla_profile)?;

    Ok(())
}

/// Validate that all required recipient profile fields are present and valid
pub fn validate_recipient_profile_completeness(
    recipient: &RecipientProfile,
) -> Result<(), ContractError> {
    // Age validation
    if recipient.age < 1 || recipient.age > 85 {
        return Err(ContractError::InvalidMedicalFacility);
    }

    // Medical priority score validation
    if recipient.medical_priority_score > 1000 {
        return Err(ContractError::InvalidUrgencyLevel);
    }

    // HLA profile validation
    validate_hla_profile(&recipient.hla_profile)?;

    Ok(())
}

/// Calculate wait list position based on priority score
/// Higher priority = lower position number (closer to front of line)
pub fn calculate_waitlist_position(recipient_priority: u32, total_recipients: u32) -> u32 {
    if recipient_priority > 1000 {
        1 // Critical cases at front
    } else if recipient_priority > 750 {
        total_recipients / 10 // Top 10%
    } else if recipient_priority > 500 {
        total_recipients / 4 // Top 25%
    } else {
        total_recipients / 2 // Middle of list
    }
}

/// Generate a medical urgency description
pub fn get_urgency_description(env: &Env, urgency_level: &UrgencyLevel) -> String {
    let description = match urgency_level {
        UrgencyLevel::Critical => "Life-threatening condition requiring immediate transplant",
        UrgencyLevel::High => "Serious condition with significant deterioration",
        UrgencyLevel::Medium => "Stable but declining health requiring transplant",
        UrgencyLevel::Low => "Stable condition, can wait for optimal match",
    };

    String::from_str(env, description)
}

/// Calculate expected wait time based on organ type and blood type
/// Returns estimated wait time in days
pub fn estimate_wait_time(
    organ_type: &OrganType,
    blood_type: &BloodType,
    urgency_level: &UrgencyLevel,
) -> u32 {
    // Base wait times by organ type (in days)
    let base_wait = match organ_type {
        OrganType::Kidney => 1095,   // ~3 years average
        OrganType::Liver => 365,     // ~1 year average
        OrganType::Heart => 180,     // ~6 months average
        OrganType::Lung => 150,      // ~5 months average
        OrganType::Pancreas => 730,  // ~2 years average
        OrganType::Intestine => 270, // ~9 months average
    };

    // Blood type modifier (universal donors wait longer)
    let blood_modifier = match blood_type {
        BloodType::O => 1.2,  // Wait 20% longer
        BloodType::AB => 0.7, // Wait 30% less (universal recipient)
        BloodType::A => 1.0,  // Average
        BloodType::B => 0.9,  // Slightly less
    };

    // Urgency modifier
    let urgency_modifier = match urgency_level {
        UrgencyLevel::Critical => 0.3, // Much faster (70% reduction)
        UrgencyLevel::High => 0.6,     // Faster (40% reduction)
        UrgencyLevel::Medium => 1.0,   // Average
        UrgencyLevel::Low => 1.4,      // Slower (40% increase)
    };

    ((base_wait as f64) * blood_modifier * urgency_modifier) as u32
}

pub fn meets_minimum_requirements(
    compatibility_score: u32,
    priority_score: u32,
    min_compatibility: u32,
) -> bool {
    compatibility_score >= min_compatibility && priority_score > 0
}

/// Calculate organ allocation fairness score
/// Measures how fairly organs are being distributed
pub fn calculate_fairness_score(
    wait_time_days: u32,
    medical_priority: u32,
    urgency_level: &UrgencyLevel,
) -> u32 {
    let wait_component = wait_time_days.min(365) / 10; // Max 36 points for 1 year
    let medical_component = medical_priority / 25; // Max 40 points
    let urgency_component = match urgency_level {
        UrgencyLevel::Critical => 24,
        UrgencyLevel::High => 18,
        UrgencyLevel::Medium => 12,
        UrgencyLevel::Low => 6,
    };

    wait_component + medical_component + urgency_component
}

/// Validate that a match has not expired
pub fn is_match_valid(match_result: &MatchResult, current_time: u64, max_age_seconds: u64) -> bool {
    let age = current_time - match_result.matched_at;
    age <= max_age_seconds && !match_result.confirmed
}

/// Calculate the total number of potential recipients for an organ
pub fn count_potential_recipients(
    env: &Env,
    organ_type: &OrganType,
    blood_type: &BloodType,
) -> u32 {
    // Simplified calculation based on population statistics
    let (blood_freq, organ_demand) = calculate_population_statistics(env, blood_type, organ_type);

    // Estimate potential recipients
    (blood_freq * organ_demand) / 100
}

/// Generate a unique match ID based on timestamp and ledger sequence
pub fn generate_match_id(env: &Env) -> u32 {
    let timestamp = env.ledger().timestamp();
    let sequence: u32 = env.ledger().sequence();
    let time_stamp_mod: u32 = (timestamp % 1000000).try_into().unwrap();
    sequence + time_stamp_mod
}
