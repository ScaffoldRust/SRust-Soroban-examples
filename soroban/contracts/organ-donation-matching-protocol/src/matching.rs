// matching.rs - Organ Donation Matching Algorithms
// Implements UNOS-inspired compatibility scoring and matching logic

use crate::utils::generate_match_id;
use crate::{
    BloodType, Config, ContractError, DataKey, DonorProfile, HLAProfile, MatchResult, OrganType,
    RecipientProfile, UrgencyLevel,
};
use soroban_sdk::{Env, Vec};

/// Calculate blood type compatibility score
/// Returns 0-100 score, with 0 meaning incompatible (absolute rejection)
pub fn blood_compatibility_score(donor_blood: &BloodType, recipient_blood: &BloodType) -> u32 {
    match (donor_blood, recipient_blood) {
        // Perfect matches - same blood type
        (BloodType::A, BloodType::A) => 100,
        (BloodType::B, BloodType::B) => 100,
        (BloodType::AB, BloodType::AB) => 100,
        (BloodType::O, BloodType::O) => 100,

        // Universal donor O to any recipient
        (BloodType::O, BloodType::A) => 95,
        (BloodType::O, BloodType::B) => 95,
        (BloodType::O, BloodType::AB) => 95,

        // A can donate to AB
        (BloodType::A, BloodType::AB) => 90,

        // B can donate to AB
        (BloodType::B, BloodType::AB) => 90,

        // All other combinations are incompatible
        _ => 0,
    }
}

/// Calculate HLA compatibility score using crossmatch simulation
/// Compares Human Leukocyte Antigen alleles across three loci: A, B, and DR
/// Higher scores indicate better tissue compatibility and lower rejection risk
pub fn hla_compatibility_score(donor_hla: &HLAProfile, recipient_hla: &HLAProfile) -> u32 {
    let mut total_matches = 0u32;
    let mut total_comparisons = 0u32;

    // Compare HLA-A alleles
    for donor_allele in donor_hla.hla_a.iter() {
        total_comparisons += 1;
        if recipient_hla.hla_a.contains(donor_allele) {
            total_matches += 1;
        }
    }

    // Compare HLA-B alleles
    for donor_allele in donor_hla.hla_b.iter() {
        total_comparisons += 1;
        if recipient_hla.hla_b.contains(donor_allele) {
            total_matches += 1;
        }
    }

    // Compare HLA-DR alleles
    for donor_allele in donor_hla.hla_dr.iter() {
        total_comparisons += 1;
        if recipient_hla.hla_dr.contains(donor_allele) {
            total_matches += 1;
        }
    }

    if total_comparisons == 0 {
        return 50; // Default middle score if no HLA data available
    }

    // Calculate percentage match and scale to 0-100
    (total_matches * 100) / total_comparisons
}

/// Calculate age compatibility score based on donor and recipient ages
/// Different organs have different age tolerance thresholds
pub fn age_compatibility_score(donor_age: u32, recipient_age: u32, organ_type: &OrganType) -> u32 {
    let age_diff = if donor_age > recipient_age {
        donor_age - recipient_age
    } else {
        recipient_age - donor_age
    };

    // Different age tolerance based on organ type
    let max_acceptable_diff = match organ_type {
        OrganType::Kidney => 15,    // Most tolerant
        OrganType::Liver => 20,     // Very tolerant
        OrganType::Heart => 10,     // Strict - size and function critical
        OrganType::Lung => 10,      // Strict - size matching important
        OrganType::Pancreas => 15,  // Moderate tolerance
        OrganType::Intestine => 20, // Flexible
    };

    if age_diff == 0 {
        100 // Perfect age match
    } else if age_diff <= max_acceptable_diff / 2 {
        90 - (age_diff * 5) // Good compatibility with slight penalty
    } else if age_diff <= max_acceptable_diff {
        60 - (age_diff * 2) // Acceptable but not ideal
    } else {
        20 // Suboptimal but still possible in emergency cases
    }
}

/// Calculate urgency-based priority score
/// Combines medical urgency level, wait time, and medical priority assessment
pub fn calculate_priority_score(
    recipient: &RecipientProfile,
    wait_time_days: u32,
    urgency_weight: u32,
) -> u32 {
    // Base urgency score from medical assessment
    let urgency_score = match recipient.urgency_level {
        UrgencyLevel::Critical => 1000, // Life-threatening, immediate need
        UrgencyLevel::High => 750,      // Urgent, significant deterioration
        UrgencyLevel::Medium => 500,    // Important but stable
        UrgencyLevel::Low => 250,       // Can wait longer
    };

    // Wait time score - capped at 1 year to prevent extreme values
    let wait_time_score = wait_time_days.min(365) * 2;

    // Medical priority score from healthcare provider
    let medical_score = recipient.medical_priority_score;

    // Weighted combination with configurable urgency weight
    (urgency_score * urgency_weight / 100) + wait_time_score + medical_score
}

/// Calculate overall compatibility score
/// This is the core matching algorithm that combines all factors
pub fn calculate_compatibility_score(donor: &DonorProfile, recipient: &RecipientProfile) -> u32 {
    // Blood type compatibility (40% weight) - CRITICAL FACTOR
    let blood_score = blood_compatibility_score(&donor.blood_type, &recipient.blood_type);
    if blood_score == 0 {
        return 0; // Blood incompatibility is absolute - cannot proceed
    }

    // HLA compatibility (35% weight) - MAJOR FACTOR
    let hla_score = hla_compatibility_score(&donor.hla_profile, &recipient.hla_profile);

    // Age compatibility (25% weight) - SUPPORTING FACTOR
    let age_score = age_compatibility_score(donor.age, recipient.age, &donor.organ_type);

    // Weighted average of all factors
    (blood_score * 40 + hla_score * 35 + age_score * 25) / 100
}

/// Check if donor and recipient have the same organ type
pub fn organ_type_match(donor: &DonorProfile, recipient: &RecipientProfile) -> bool {
    donor.organ_type == recipient.organ_type
}

/// Find all compatible donors for a recipient using UNOS-inspired algorithm
/// This is the main matching function that searches through available donors
pub fn find_compatible_donors(
    env: &Env,
    recipient: &RecipientProfile,
    config: &Config,
) -> Result<Vec<MatchResult>, ContractError> {
    let mut matches = Vec::new(env);
    let donor_count: u32 = env
        .storage()
        .instance()
        .get(&DataKey::DonorCount)
        .unwrap_or(0);
    let current_time = env.ledger().timestamp();

    // Calculate current wait time for recipient
    let wait_time_days = ((current_time - recipient.registered_at) / 86400) as u32;

    for donor_id in 0..donor_count {
        if let Some(donor) = get_donor_by_index(env, donor_id) {
            // Skip inactive donors
            if !donor.is_active {
                continue;
            }

            // Check organ type match - must be exact
            if !organ_type_match(&donor, recipient) {
                continue;
            }

            // Calculate compatibility score
            let compatibility_score = calculate_compatibility_score(&donor, recipient);

            // Check if compatibility meets minimum threshold
            if compatibility_score < config.compatibility_threshold {
                continue;
            }

            // Calculate priority score for this match
            let priority_score =
                calculate_priority_score(recipient, wait_time_days, config.urgency_weight);

            // Generate unique match ID
            let match_id = generate_match_id(env);

            // Create match result
            let match_result = MatchResult {
                match_id,
                donor: donor.address.clone(),
                recipient: recipient.address.clone(),
                compatibility_score,
                priority_score,
                matched_at: current_time,
                confirmed: false,
                medical_facility: recipient.medical_facility.clone(),
            };

            // Store the match for future reference
            env.storage()
                .persistent()
                .set(&DataKey::Match(match_id), &match_result);
            matches.push_back(match_result);
        }
    }

    // Sort matches by priority score (highest first), then by compatibility
    sort_matches_by_priority(env, &mut matches);

    Ok(matches)
}

/// Get donor by index
/// In production, this would use a proper indexing system
fn get_donor_by_index(env: &Env, _index: u32) -> Option<DonorProfile> {
    let donor_address = env
        .storage()
        .instance()
        .get(&DataKey::DonorIndex(_index))
        .unwrap();
    env.storage()
        .persistent()
        .get(&DataKey::Donor(donor_address))
}

/// Sort matches by priority and compatibility scores
/// Uses bubble sort for simplicity in Soroban environment
fn sort_matches_by_priority(_env: &Env, matches: &mut Vec<MatchResult>) {
    let len = matches.len();
    if len <= 1 {
        return;
    }

    // Bubble sort - simple and works well for small datasets
    for i in 0..len {
        for j in 0..(len - i - 1) {
            let current = matches.get(j).unwrap();
            let next = matches.get(j + 1).unwrap();

            // Sort by priority score first (descending), then by compatibility (descending)
            let should_swap = if current.priority_score != next.priority_score {
                current.priority_score < next.priority_score
            } else {
                current.compatibility_score < next.compatibility_score
            };

            if should_swap {
                let temp = current.clone();
                matches.set(j, next.clone());
                matches.set(j + 1, temp);
            }
        }
    }
}

/// Advanced matching algorithm that considers multiple factors
/// This extends the basic matching with additional considerations
pub fn calculate_unos_score(
    donor: &DonorProfile,
    recipient: &RecipientProfile,
    wait_time_days: u32,
    urgency_weight: u32,
) -> u32 {
    let compatibility_score = calculate_compatibility_score(donor, recipient);
    if compatibility_score == 0 {
        return 0; // Incompatible match
    }

    let priority_score = calculate_priority_score(recipient, wait_time_days, urgency_weight);

    // Geographic proximity bonus (simplified)
    // In reality, would calculate actual distance between medical facilities
    let geographic_bonus = if donor.medical_facility == recipient.medical_facility {
        50 // Same facility - faster transport
    } else {
        0
    };

    // Time sensitivity bonus for organs with short viability windows
    let time_bonus = match donor.organ_type {
        OrganType::Heart => 100,    // 4-6 hours - most urgent
        OrganType::Lung => 90,      // 6-8 hours
        OrganType::Liver => 80,     // 12-18 hours
        OrganType::Pancreas => 60,  // 12-20 hours
        OrganType::Intestine => 50, // 8-12 hours
        OrganType::Kidney => 0,     // 24-36 hours - less urgent
    };

    // Final weighted score combining all factors
    compatibility_score + priority_score + geographic_bonus + time_bonus
}

/// Check for crossmatch compatibility (simplified antibody testing)
/// In reality, this would involve complex serological testing
pub fn check_crossmatch_compatibility(donor_hla: &HLAProfile, recipient_hla: &HLAProfile) -> bool {
    // Simplified crossmatch check
    // Real implementation would check for recipient antibodies against donor HLA
    let hla_score = hla_compatibility_score(donor_hla, recipient_hla);
    hla_score >= 60 // Minimum 60% HLA compatibility threshold
}

/// Validate medical compatibility beyond basic matching
/// Comprehensive check before finalizing a match
pub fn validate_medical_compatibility(
    donor: &DonorProfile,
    recipient: &RecipientProfile,
) -> Result<bool, ContractError> {
    // Organ type must match
    if !organ_type_match(donor, recipient) {
        return Ok(false);
    }

    // Blood type compatibility is mandatory
    if blood_compatibility_score(&donor.blood_type, &recipient.blood_type) == 0 {
        return Ok(false);
    }

    // HLA crossmatch must pass
    if !check_crossmatch_compatibility(&donor.hla_profile, &recipient.hla_profile) {
        return Ok(false);
    }

    // Age compatibility check for specific organs
    let age_score = age_compatibility_score(donor.age, recipient.age, &donor.organ_type);
    if age_score < 20 {
        return Ok(false); // Too large age gap
    }

    Ok(true)
}

/// Calculate expected transplant success probability (simplified model)
/// Returns a percentage (0-100) based on compatibility factors
pub fn calculate_success_probability(donor: &DonorProfile, recipient: &RecipientProfile) -> u32 {
    let compatibility = calculate_compatibility_score(donor, recipient);

    // Organ-specific base success rates
    let base_rate = match donor.organ_type {
        OrganType::Kidney => 90,    // Highest success rate
        OrganType::Liver => 85,     // Very good outcomes
        OrganType::Heart => 80,     // Good but complex
        OrganType::Lung => 75,      // More challenging
        OrganType::Pancreas => 85,  // Good outcomes
        OrganType::Intestine => 70, // Most challenging
    };

    // Adjust based on compatibility score
    let compatibility_factor = compatibility / 100;
    (base_rate * compatibility_factor).min(100)
}

/// Estimate organ viability remaining time
/// Returns time in seconds until organ is no longer viable
pub fn estimate_viability_remaining(
    organ_type: &OrganType,
    procurement_time: u64,
    current_time: u64,
) -> u64 {
    let viability_hours = match organ_type {
        OrganType::Heart => 4,
        OrganType::Lung => 6,
        OrganType::Liver => 12,
        OrganType::Kidney => 24,
        OrganType::Pancreas => 12,
        OrganType::Intestine => 8,
    };

    let viability_seconds = viability_hours * 3600;
    let elapsed = current_time - procurement_time;

    if elapsed >= viability_seconds {
        0 // No time remaining
    } else {
        viability_seconds - elapsed
    }
}
