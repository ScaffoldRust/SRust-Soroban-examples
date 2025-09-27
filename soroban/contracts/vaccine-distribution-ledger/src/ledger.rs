use crate::distribution_storage;
use crate::error::ContractError;
use crate::events::*;
use crate::distribution_storage::*;
use crate::utils;
use soroban_sdk::{Address, Env, String, Vec};

pub fn initialize_batch(
    env: &Env,
    batch_id: String,
    manufacturer: &Address,
    vaccine_type: String,
    production_date: u64,
    quantity: u32,
    expiry_date: u64,
) -> Result<(), ContractError> {
    if quantity == 0 {
        return Err(ContractError::InvalidQuantity);
    }

    if get_batch(env, batch_id.clone()).is_ok() {
        return Err(ContractError::BatchAlreadyExists);
    }

    if !utils::is_valid_date(production_date, expiry_date) {
        return Err(ContractError::InvalidDate);
    }

    let timestamp = env.ledger().timestamp();
    
    let batch = VaccineBatch {
        batch_id: batch_id.clone(),
        manufacturer: manufacturer.clone(),
        vaccine_type: vaccine_type.clone(),
        production_date,
        expiry_date,
        initial_quantity: quantity,
        current_quantity: quantity,
        status: BatchStatus::Produced,
        created_at: timestamp,
        last_updated: timestamp,
        notes: None,
    };

    set_batch(env, &batch);
    add_manufacturer_batch(env, manufacturer, &batch_id);
    add_batch_to_status(env, &BatchStatus::Produced, &batch_id);

    // Create initial production event
    let event_id = get_next_event_id(env);
    let production_event = DistributionEvent {
        event_id,
        batch_id: batch_id.clone(),
        event_type: EventType::Production,
        actor: manufacturer.clone(),
        timestamp,
        quantity,
        destination: None,
        temperature_log: None,
        notes: Some(String::from_str(env, "Initial production")),
        location: None,
    };

    set_distribution_event(env, &production_event);
    add_batch_event(env, &batch_id, event_id);

    emit_batch_initialized(
        env,
        batch_id,
        manufacturer.clone(),
        vaccine_type,
        production_date,
        quantity,
        expiry_date,
    );

    Ok(())
}

pub fn update_batch_status(
    env: &Env,
    batch_id: String,
    updater: &Address,
    new_status: BatchStatus,
    notes: Option<String>,
) -> Result<(), ContractError> {
    
    let mut batch = get_batch(env, batch_id.clone())?;

    // Check authorization based on status change
    if !utils::can_update_batch_status(env, updater, &batch, &new_status) {
        return Err(ContractError::Unauthorized);
    }

    let old_status = batch.status.clone();
    let timestamp = env.ledger().timestamp();

    // Update batch status
    remove_batch_from_status(env, &old_status, &batch_id);
    add_batch_to_status(env, &new_status, &batch_id);

    batch.status = new_status.clone();
    batch.last_updated = timestamp;
    batch.notes = notes.clone();

    set_batch(env, &batch);

    // Create status update event
    let event_id = get_next_event_id(env);
    let status_event = DistributionEvent {
        event_id,
        batch_id: batch_id.clone(),
        event_type: EventType::StatusUpdate,
        actor: updater.clone(),
        timestamp,
        quantity: 0,
        destination: None,
        temperature_log: None,
        notes,
        location: None,
    };

    set_distribution_event(env, &status_event);
    add_batch_event(env, &batch_id, event_id);

    emit_batch_status_updated(
        env,
        batch_id,
        updater.clone(),
        old_status,
        new_status,
        timestamp,
    );

    Ok(())
}

pub fn get_batch(env: &Env, batch_id: String) -> Result<VaccineBatch, ContractError> {
    distribution_storage::get_batch(env, &batch_id)
        .ok_or(ContractError::BatchNotFound)
}

pub fn get_batch_history(
    env: &Env,
    batch_id: String,
    offset: u32,
    limit: u32,
) -> Result<Vec<DistributionEvent>, ContractError> {
    let event_ids = get_batch_history_ids(env, &batch_id);
    let mut events = Vec::new(env);

    let start = offset as usize;
    let end = (offset + limit) as usize;
    let total_events = event_ids.len() as usize;

    for i in start..end.min(total_events) {
        if let Some(event_id) = event_ids.get(i as u32) {
            if let Some(event) = get_distribution_event(env, event_id) {
                events.push_back(event);
            }
        }
    }

    Ok(events)
}

pub fn get_batch_inventory(env: &Env, batch_id: String) -> Result<u32, ContractError> {
    let batch = get_batch(env, batch_id)?;
    Ok(batch.current_quantity)
}

pub fn get_manufacturer_batches(
    env: &Env,
    manufacturer: &Address,
    offset: u32,
    limit: u32,
) -> Result<Vec<String>, ContractError> {
    let all_batches = get_manufacturer_batch_ids(env, manufacturer);
    let mut result = Vec::new(env);

    let start = offset as usize;
    let end = (offset + limit) as usize;
    let total_batches = all_batches.len() as usize;

    for i in start..end.min(total_batches) {
        if let Some(batch_id) = all_batches.get(i as u32) {
            result.push_back(batch_id);
        }
    }

    Ok(result)
}

pub fn get_batches_by_status(
    env: &Env,
    status: BatchStatus,
    offset: u32,
    limit: u32,
) -> Result<Vec<String>, ContractError> {
    let all_batches = get_batches_by_status_ids(env, &status);
    let mut result = Vec::new(env);

    let start = offset as usize;
    let end = (offset + limit) as usize;
    let total_batches = all_batches.len() as usize;

    for i in start..end.min(total_batches) {
        if let Some(batch_id) = all_batches.get(i as u32) {
            result.push_back(batch_id);
        }
    }

    Ok(result)
}

pub fn get_administration_records(
    env: &Env,
    batch_id: String,
    offset: u32,
    limit: u32,
) -> Result<Vec<AdministrationRecord>, ContractError> {
    let record_ids = get_batch_administration_ids(env, &batch_id);
    let mut records = Vec::new(env);

    let start = offset as usize;
    let end = (offset + limit) as usize;
    let total_records = record_ids.len() as usize;

    for i in start..end.min(total_records) {
        if let Some(record_id) = record_ids.get(i as u32) {
            if let Some(record) = get_administration_record(env, record_id) {
                records.push_back(record);
            }
        }
    }

    Ok(records)
}