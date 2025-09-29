#![no_std]
use soroban_sdk::{contract, contracterror, contractimpl, contracttype, Address, Env, String, Vec};

mod matching;
mod protocol;
mod test;
mod utils;

pub use matching::*;
pub use protocol::*;
pub use utils::*;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Admin,
    DonorCount,
    RecipientCount,
    Donor(Address),
    DonorIndex(u32),
    Recipient(Address),
    Match(u32),
    Config,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Config {
    pub admin: Address,
    pub matching_enabled: bool,
    pub max_donors: u32,
    pub max_recipients: u32,
    pub urgency_weight: u32,
    pub compatibility_threshold: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BloodType {
    A,
    B,
    AB,
    O,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OrganType {
    Kidney,
    Liver,
    Heart,
    Lung,
    Pancreas,
    Intestine,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UrgencyLevel {
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HLAProfile {
    pub hla_a: Vec<String>,
    pub hla_b: Vec<String>,
    pub hla_dr: Vec<String>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DonorProfile {
    pub address: Address,
    pub blood_type: BloodType,
    pub organ_type: OrganType,
    pub hla_profile: HLAProfile,
    pub age: u32,
    pub registered_at: u64,
    pub is_active: bool,
    pub medical_facility: Address,
    pub consent_hash: String,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecipientProfile {
    pub address: Address,
    pub blood_type: BloodType,
    pub organ_type: OrganType,
    pub hla_profile: HLAProfile,
    pub age: u32,
    pub urgency_level: UrgencyLevel,
    pub registered_at: u64,
    pub wait_time_days: u32,
    pub is_active: bool,
    pub medical_facility: Address,
    pub medical_priority_score: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MatchResult {
    pub match_id: u32,
    pub donor: Address,
    pub recipient: Address,
    pub compatibility_score: u32,
    pub priority_score: u32,
    pub matched_at: u64,
    pub confirmed: bool,
    pub medical_facility: Address,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    NotAuthorized = 1,
    AlreadyInitialized = 2,
    NotInitialized = 3,
    DonorNotFound = 4,
    RecipientNotFound = 5,
    InvalidBloodType = 6,
    InvalidOrganType = 7,
    MatchNotFound = 8,
    AlreadyMatched = 9,
    InsufficientCompatibility = 10,
    MaxCapacityReached = 11,
    InvalidUrgencyLevel = 12,
    InactiveProfile = 13,
    InvalidMedicalFacility = 14,
    ConsentNotProvided = 15,
}

#[contract]
pub struct OrganDonationMatchingContract;

#[contractimpl]
impl OrganDonationMatchingContract {
    /// Initialize the organ donation matching protocol
    pub fn initialize(
        env: Env,
        admin: Address,
        max_donors: u32,
        max_recipients: u32,
        urgency_weight: u32,
        compatibility_threshold: u32,
    ) -> Result<(), ContractError> {
        admin.require_auth();

        if env.storage().instance().has(&DataKey::Config) {
            return Err(ContractError::AlreadyInitialized);
        }

        let config = Config {
            admin: admin.clone(),
            matching_enabled: true,
            max_donors,
            max_recipients,
            urgency_weight,
            compatibility_threshold,
        };

        env.storage().instance().set(&DataKey::Config, &config);
        env.storage().instance().set(&DataKey::DonorCount, &0u32);
        env.storage()
            .instance()
            .set(&DataKey::RecipientCount, &0u32);

        Ok(())
    }

    /// Register a new organ donor
    pub fn register_donor(
        env: Env,
        donor_address: Address,
        blood_type: BloodType,
        organ_type: OrganType,
        hla_profile: HLAProfile,
        age: u32,
        medical_facility: Address,
        consent_hash: String,
    ) -> Result<(), ContractError> {
        donor_address.require_auth();
        medical_facility.require_auth();

        let config: Config = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(ContractError::NotInitialized)?;

        let donor_count: u32 = env
            .storage()
            .instance()
            .get(&DataKey::DonorCount)
            .unwrap_or(0);

        if donor_count >= config.max_donors {
            return Err(ContractError::MaxCapacityReached);
        }

        protocol::validate_donor_data(
            &donor_address,
            &blood_type,
            &organ_type,
            &hla_profile,
            age,
            &medical_facility,
            &consent_hash,
        )?;
        let donor = DonorProfile {
            address: donor_address.clone(),
            blood_type,
            organ_type,
            hla_profile,
            age,
            registered_at: env.ledger().timestamp(),
            is_active: true,
            medical_facility,
            consent_hash,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Donor(donor_address.clone()), &donor);
        env.storage()
            .instance()
            .set(&DataKey::DonorIndex(donor_count), &donor_address);
        env.storage()
            .instance()
            .set(&DataKey::DonorCount, &(donor_count + 1));

        Ok(())
    }

    /// Register a new organ recipient
    pub fn register_recipient(
        env: Env,
        recipient_address: Address,
        blood_type: BloodType,
        organ_type: OrganType,
        hla_profile: HLAProfile,
        age: u32,
        urgency_level: UrgencyLevel,
        medical_facility: Address,
        medical_priority_score: u32,
    ) -> Result<(), ContractError> {
        recipient_address.require_auth();
        medical_facility.require_auth();

        let config: Config = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(ContractError::NotInitialized)?;

        let recipient_count: u32 = env
            .storage()
            .instance()
            .get(&DataKey::RecipientCount)
            .unwrap_or(0);

        if recipient_count >= config.max_recipients {
            return Err(ContractError::MaxCapacityReached);
        }

        protocol::validate_recipient_data(
            &recipient_address,
            &blood_type,
            &organ_type,
            &hla_profile,
            age,
            &urgency_level,
            &medical_facility,
            medical_priority_score,
        )?;
        let recipient = RecipientProfile {
            address: recipient_address.clone(),
            blood_type,
            organ_type,
            hla_profile,
            age,
            urgency_level,
            registered_at: env.ledger().timestamp(),
            wait_time_days: 0,
            is_active: true,
            medical_facility,
            medical_priority_score,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Recipient(recipient_address), &recipient);
        env.storage()
            .instance()
            .set(&DataKey::RecipientCount, &(recipient_count + 1));

        Ok(())
    }

    /// Find potential matches for a given recipient
    pub fn find_match(
        env: Env,
        recipient_address: Address,
    ) -> Result<Vec<MatchResult>, ContractError> {
        let config: Config = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(ContractError::NotInitialized)?;

        if !config.matching_enabled {
            return Ok(Vec::new(&env));
        }

        let recipient: RecipientProfile = env
            .storage()
            .persistent()
            .get(&DataKey::Recipient(recipient_address.clone()))
            .ok_or(ContractError::RecipientNotFound)?;

        if !recipient.is_active {
            return Err(ContractError::InactiveProfile);
        }

        matching::find_compatible_donors(&env, &recipient, &config)
    }

    /// Confirm a match between donor and recipient
    pub fn confirm_match(
        env: Env,
        match_id: u32,
        medical_facility: Address,
    ) -> Result<(), ContractError> {
        medical_facility.require_auth();

        let mut match_result: MatchResult = env
            .storage()
            .persistent()
            .get(&DataKey::Match(match_id))
            .ok_or(ContractError::MatchNotFound)?;

        if match_result.confirmed {
            return Err(ContractError::AlreadyMatched);
        }

        if match_result.medical_facility != medical_facility {
            return Err(ContractError::NotAuthorized);
        }

        match_result.confirmed = true;
        env.storage()
            .persistent()
            .set(&DataKey::Match(match_id), &match_result);

        // Deactivate donor and recipient profiles
        protocol::deactivate_matched_profiles(&env, &match_result.donor, &match_result.recipient)?;

        Ok(())
    }

    /// Get donor profile
    pub fn get_donor(env: Env, donor_address: Address) -> Option<DonorProfile> {
        env.storage()
            .persistent()
            .get(&DataKey::Donor(donor_address))
    }

    /// Get recipient profile
    pub fn get_recipient(env: Env, recipient_address: Address) -> Option<RecipientProfile> {
        env.storage()
            .persistent()
            .get(&DataKey::Recipient(recipient_address))
    }

    /// Get match result
    pub fn get_match(env: Env, match_id: u32) -> Option<MatchResult> {
        env.storage().persistent().get(&DataKey::Match(match_id))
    }

    /// Update recipient urgency level (admin or medical facility only)
    pub fn update_recipient_urgency(
        env: Env,
        recipient_address: Address,
        urgency_level: UrgencyLevel,
        medical_priority_score: u32,
        caller: Address,
    ) -> Result<(), ContractError> {
        caller.require_auth();

        let config: Config = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(ContractError::NotInitialized)?;

        let mut recipient: RecipientProfile = env
            .storage()
            .persistent()
            .get(&DataKey::Recipient(recipient_address.clone()))
            .ok_or(ContractError::RecipientNotFound)?;

        // Only admin or registered medical facility can update urgency
        if caller != config.admin && caller != recipient.medical_facility {
            return Err(ContractError::NotAuthorized);
        }

        recipient.urgency_level = urgency_level;
        recipient.medical_priority_score = medical_priority_score;

        env.storage()
            .persistent()
            .set(&DataKey::Recipient(recipient_address), &recipient);

        Ok(())
    }

    /// Deactivate donor profile
    pub fn deactivate_donor(
        env: Env,
        donor_address: Address,
        caller: Address,
    ) -> Result<(), ContractError> {
        caller.require_auth();

        let config: Config = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(ContractError::NotInitialized)?;

        let mut donor: DonorProfile = env
            .storage()
            .persistent()
            .get(&DataKey::Donor(donor_address.clone()))
            .ok_or(ContractError::DonorNotFound)?;

        // Only admin, donor themselves, or their medical facility can deactivate
        if caller != config.admin && caller != donor.address && caller != donor.medical_facility {
            return Err(ContractError::NotAuthorized);
        }

        donor.is_active = false;
        env.storage()
            .persistent()
            .set(&DataKey::Donor(donor_address), &donor);

        Ok(())
    }

    /// Deactivate recipient profile
    pub fn deactivate_recipient(
        env: Env,
        recipient_address: Address,
        caller: Address,
    ) -> Result<(), ContractError> {
        caller.require_auth();

        let config: Config = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(ContractError::NotInitialized)?;

        let mut recipient: RecipientProfile = env
            .storage()
            .persistent()
            .get(&DataKey::Recipient(recipient_address.clone()))
            .ok_or(ContractError::RecipientNotFound)?;

        // Only admin, recipient themselves, or their medical facility can deactivate
        if caller != config.admin
            && caller != recipient.address
            && caller != recipient.medical_facility
        {
            return Err(ContractError::NotAuthorized);
        }

        recipient.is_active = false;
        env.storage()
            .persistent()
            .set(&DataKey::Recipient(recipient_address), &recipient);

        Ok(())
    }
}
