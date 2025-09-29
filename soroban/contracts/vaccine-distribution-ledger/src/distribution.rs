use crate::error::ContractError;
use crate::events::*;
use crate::distribution_storage::*;
use crate::utils;
use soroban_sdk::{Address, Env, String};

pub fn log_distribution_event(
    env: &Env,
    batch_id: String,
    distributor: &Address,
    destination: String,
    quantity: u32,
    temperature_log: Option<String>,
) -> Result<(), ContractError> {
    let mut batch = get_batch(env, &batch_id)
        .ok_or(ContractError::BatchNotFound)?;

    if quantity == 0 {
        return Err(ContractError::InvalidQuantity);
    }

    if batch.current_quantity < quantity {
        return Err(ContractError::InsufficientQuantity);
    }

    if batch.status == BatchStatus::Expired || batch.status == BatchStatus::Recalled {
        return Err(ContractError::BatchInactive);
    }

    if utils::is_batch_expired(env, &batch) {
        return Err(ContractError::BatchExpired);
    }

    if !utils::is_valid_destination(&destination) {
        return Err(ContractError::InvalidDestination);
    }

    let timestamp = env.ledger().timestamp();
    
    // Update batch quantity and status
    let old_quantity = batch.current_quantity;
    batch.current_quantity -= quantity;
    batch.last_updated = timestamp;

    if batch.status == BatchStatus::Produced {
        remove_batch_from_status(env, &BatchStatus::Produced, &batch_id);
        batch.status = BatchStatus::InTransit;
        add_batch_to_status(env, &BatchStatus::InTransit, &batch_id);
    }

    set_batch(env, &batch);

    // Create distribution event
    let event_id = get_next_event_id(env);
    let distribution_event = DistributionEvent {
        event_id,
        batch_id: batch_id.clone(),
        event_type: EventType::Distribution,
        actor: distributor.clone(),
        timestamp,
        quantity,
        destination: Some(destination.clone()),
        temperature_log,
        notes: None,
        location: None,
    };

    set_distribution_event(env, &distribution_event);
    add_batch_event(env, &batch_id, event_id);

    emit_distribution_logged(
        env,
        batch_id.clone(),
        distributor.clone(),
        destination,
        quantity,
        timestamp,
    );

    emit_inventory_updated(
        env,
        batch_id,
        old_quantity,
        batch.current_quantity,
        String::from_str(env, "Distribution"),
        timestamp,
    );

    Ok(())
}

pub fn verify_administration(
    env: &Env,
    batch_id: String,
    administrator: &Address,
    patient_id: String,
    administered_quantity: u32,
    location: String,
) -> Result<(), ContractError> {
    let mut batch = get_batch(env, &batch_id)
        .ok_or(ContractError::BatchNotFound)?;

    if administered_quantity == 0 {
        return Err(ContractError::InvalidQuantity);
    }

    if batch.current_quantity < administered_quantity {
        return Err(ContractError::InsufficientQuantity);
    }

    if batch.status == BatchStatus::Expired || batch.status == BatchStatus::Recalled {
        return Err(ContractError::BatchInactive);
    }

    if utils::is_batch_expired(env, &batch) {
        return Err(ContractError::BatchExpired);
    }

    if !utils::is_valid_patient_id(&patient_id) {
        return Err(ContractError::InvalidPatientId);
    }

    if !utils::is_valid_location(&location) {
        return Err(ContractError::InvalidLocation);
    }

    let timestamp = env.ledger().timestamp();

    // Check for duplicate administration to same patient
    if utils::check_duplicate_administration(env, &batch_id, &patient_id) {
        return Err(ContractError::DuplicateAdministration);
    }

    // Update batch quantity and status
    let old_quantity = batch.current_quantity;
    batch.current_quantity -= administered_quantity;
    batch.last_updated = timestamp;

   // Update status to Administered if not already in a terminal state
    if batch.status != BatchStatus::Administered 
        && batch.status != BatchStatus::Expired 
        && batch.status != BatchStatus::Recalled 
        && batch.status != BatchStatus::ColdChainBreach {
        remove_batch_from_status(env, &batch.status, &batch_id);
        batch.status = BatchStatus::Administered;
        add_batch_to_status(env, &BatchStatus::Administered, &batch_id);
    }

    set_batch(env, &batch);

    // Create administration record
    let record_id = get_next_record_id(env);
    let admin_record = AdministrationRecord {
        record_id,
        batch_id: batch_id.clone(),
        administrator: administrator.clone(),
        patient_id: patient_id.clone(),
        administered_quantity,
        location: location.clone(),
        timestamp,
        verified: true,
    };

    set_administration_record(env, &admin_record);
    add_batch_administration(env, &batch_id, record_id);

    // Create administration event
    let event_id = get_next_event_id(env);
    let admin_event = DistributionEvent {
        event_id,
        batch_id: batch_id.clone(),
        event_type: EventType::Administration,
        actor: administrator.clone(),
        timestamp,
        quantity: administered_quantity,
        destination: None,
        temperature_log: None,
        notes: Some(patient_id.clone()),
        location: Some(location.clone()),
    };

    set_distribution_event(env, &admin_event);
    add_batch_event(env, &batch_id, event_id);

    emit_administration_verified(
        env,
        batch_id.clone(),
        administrator.clone(),
        patient_id,
        administered_quantity,
        location,
        timestamp,
    );

    emit_inventory_updated(
        env,
        batch_id,
        old_quantity,
        batch.current_quantity,
        String::from_str(env, "Administration"),
        timestamp,
    );

    Ok(())
}

pub fn report_cold_chain_breach(
    env: &Env,
    batch_id: String,
    reporter: &Address,
    severity: String,
    description: String,
) -> Result<(), ContractError> {
    let mut batch = get_batch(env, &batch_id)
        .ok_or(ContractError::BatchNotFound)?;

    if !utils::is_valid_severity(env, &severity) {
        return Err(ContractError::InvalidInput);
    }

    let timestamp = env.ledger().timestamp();

    // Create cold chain alert
    let alert_id = get_next_alert_id(env);
    let alert = ColdChainAlert {
        alert_id,
        batch_id: batch_id.clone(),
        reporter: reporter.clone(),
        severity: severity.clone(),
        description: description.clone(),
        timestamp,
        resolved: false,
    };

    set_cold_chain_alert(env, &alert);

    // Update batch status if severe breach
    if utils::is_severe_breach(env, &severity) {
        remove_batch_from_status(env, &batch.status, &batch_id);
        batch.status = BatchStatus::ColdChainBreach;
        batch.last_updated = timestamp;
        add_batch_to_status(env, &BatchStatus::ColdChainBreach, &batch_id);
        set_batch(env, &batch);
    }

    // Create cold chain breach event
    let event_id = get_next_event_id(env);
    let breach_event = DistributionEvent {
        event_id,
        batch_id: batch_id.clone(),
        event_type: EventType::ColdChainBreach,
        actor: reporter.clone(),
        timestamp,
        quantity: 0,
        destination: None,
        temperature_log: None,
        notes: Some(description.clone()),
        location: None,
    };

    set_distribution_event(env, &breach_event);
    add_batch_event(env, &batch_id, event_id);

    emit_cold_chain_breach_reported(
        env,
        batch_id,
        reporter.clone(),
        severity,
        description,
        timestamp,
    );

    Ok(())
}