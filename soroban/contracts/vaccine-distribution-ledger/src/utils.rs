use crate::distribution_storage::*;
use crate::storage;
use soroban_sdk::{Address, Env, String}; 

// Date validation
pub fn is_valid_date(production_date: u64, expiry_date: u64) -> bool {
    production_date < expiry_date
}

// Check if batch is expired
pub fn is_batch_expired(env: &Env, batch: &VaccineBatch) -> bool {
    let current_time = env.ledger().timestamp();
    current_time > batch.expiry_date
}

// Validate destination string
pub fn is_valid_destination(destination: &String) -> bool {
    destination.len() > 0 && destination.len() <= 100
}

// Validate patient ID
pub fn is_valid_patient_id(patient_id: &String) -> bool {
    patient_id.len() > 0 && patient_id.len() <= 50
}

// Validate location
pub fn is_valid_location(location: &String) -> bool {
    location.len() > 0 && location.len() <= 100
}

// Validate cold chain breach severity
pub fn is_valid_severity(env: &Env, severity: &String) -> bool {
    *severity == String::from_str(env, "LOW")
        || *severity == String::from_str(env, "MEDIUM")
        || *severity == String::from_str(env, "HIGH")
        || *severity == String::from_str(env, "CRITICAL")
}

// Check if breach is severe enough to change batch status
pub fn is_severe_breach(env: &Env, severity: &String) -> bool {
    *severity == String::from_str(env, "HIGH") 
        || *severity == String::from_str(env, "CRITICAL")
}

// Check authorization for batch status updates
pub fn can_update_batch_status(
    env: &Env,
    updater: &Address,
    batch: &VaccineBatch,
    new_status: &BatchStatus,
) -> bool {
    // Admin can update any status
    if storage::is_admin(env, updater) {
        return true;
    }

    // Manufacturer can update their own batches
    if batch.manufacturer == *updater {
        match new_status {
            BatchStatus::Produced | BatchStatus::InTransit | BatchStatus::Distributed => true,
            _ => false,
        }
    } else {
        // Others can only mark as distributed or administered
        match new_status {
            BatchStatus::Distributed | BatchStatus::Administered => true,
            _ => false,
        }
    }
}

// Check for duplicate administration
pub fn check_duplicate_administration(
    env: &Env,
    batch_id: &String,
    patient_id: &String,
) -> bool {
    let record_ids = get_batch_administration_ids(env, batch_id);
    
    for record_id in record_ids.iter() {
        if let Some(record) = get_administration_record(env, record_id) {
            if record.patient_id == *patient_id {
                return true;
            }
        }
    }
    
    false
}

// Validate temperature log format
pub fn _is_valid_temperature_log(temperature_log: &Option<String>) -> bool {
    match temperature_log {
        Some(log) => {
            log.len() > 0 && log.len() <= 500
        }
        None => true,
    }
}

// Check if quantity is within valid range
pub fn _is_valid_quantity(quantity: u32) -> bool {
    quantity > 0 && quantity <= 1_000_000
}

// Generate batch verification hash (simplified)
pub fn _generate_batch_hash(batch: &VaccineBatch) -> String {
    batch.batch_id.clone()
}

// Validate WHO compliance (simplified check)
pub fn _check_who_compliance(batch: &VaccineBatch) -> bool {
    let shelf_life = batch.expiry_date - batch.production_date;
    
    // Check if shelf life is reasonable (between 6 months to 5 years in seconds)
    let min_shelf_life = 6 * 30 * 24 * 60 * 60; // 6 months in seconds
    let max_shelf_life = 5 * 365 * 24 * 60 * 60; // 5 years in seconds
    
    shelf_life >= min_shelf_life && shelf_life <= max_shelf_life
}

// Calculate batch utilization percentage
pub fn _calculate_utilization(batch: &VaccineBatch) -> u32 {
    if batch.initial_quantity == 0 {
        return 0;
    }
    
    let used_quantity = batch.initial_quantity - batch.current_quantity;
    (used_quantity * 100) / batch.initial_quantity
}

// Check if batch requires urgent attention
pub fn _requires_urgent_attention(env: &Env, batch: &VaccineBatch) -> bool {
    let current_time = env.ledger().timestamp();
    let time_to_expiry = batch.expiry_date - current_time;
    let one_week_in_seconds = 7 * 24 * 60 * 60;
    
    time_to_expiry < one_week_in_seconds || batch.status == BatchStatus::ColdChainBreach
}