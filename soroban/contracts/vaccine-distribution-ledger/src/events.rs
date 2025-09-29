use soroban_sdk::{contracttype, Address, Env, String};
use crate::distribution_storage::BatchStatus;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BatchInitializedEvent {
    pub batch_id: String,
    pub manufacturer: Address,
    pub vaccine_type: String,
    pub production_date: u64,
    pub quantity: u32,
    pub expiry_date: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DistributionLoggedEvent {
    pub batch_id: String,
    pub distributor: Address,
    pub destination: String,
    pub quantity: u32,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdministrationVerifiedEvent {
    pub batch_id: String,
    pub administrator: Address,
    pub patient_id: String,
    pub quantity: u32,
    pub location: String,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BatchStatusUpdatedEvent {
    pub batch_id: String,
    pub updater: Address,
    pub old_status: BatchStatus,
    pub new_status: BatchStatus,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ColdChainBreachReportedEvent {
    pub batch_id: String,
    pub reporter: Address,
    pub severity: String,
    pub description: String,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InventoryUpdatedEvent {
    pub batch_id: String,
    pub old_quantity: u32,
    pub new_quantity: u32,
    pub change_reason: String,
    pub timestamp: u64,
}

pub fn emit_batch_initialized(
    env: &Env,
    batch_id: String,
    manufacturer: Address,
    vaccine_type: String,
    production_date: u64,
    quantity: u32,
    expiry_date: u64,
) {
    let event = BatchInitializedEvent {
        batch_id,
        manufacturer,
        vaccine_type,
        production_date,
        quantity,
        expiry_date,
    };
    env.events().publish(("batch_initialized",), event);
}

pub fn emit_distribution_logged(
    env: &Env,
    batch_id: String,
    distributor: Address,
    destination: String,
    quantity: u32,
    timestamp: u64,
) {
    let event = DistributionLoggedEvent {
        batch_id,
        distributor,
        destination,
        quantity,
        timestamp,
    };
    env.events().publish(("distribution_logged",), event);
}

pub fn emit_administration_verified(
    env: &Env,
    batch_id: String,
    administrator: Address,
    patient_id: String,
    quantity: u32,
    location: String,
    timestamp: u64,
) {
    let event = AdministrationVerifiedEvent {
        batch_id,
        administrator,
        patient_id,
        quantity,
        location,
        timestamp,
    };
    env.events().publish(("administration_verified",), event);
}

pub fn emit_batch_status_updated(
    env: &Env,
    batch_id: String,
    updater: Address,
    old_status: BatchStatus,
    new_status: BatchStatus,
    timestamp: u64,
) {
    let event = BatchStatusUpdatedEvent {
        batch_id,
        updater,
        old_status,
        new_status,
        timestamp,
    };
    env.events().publish(("batch_status_updated",), event);
}

pub fn emit_cold_chain_breach_reported(
    env: &Env,
    batch_id: String,
    reporter: Address,
    severity: String,
    description: String,
    timestamp: u64,
) {
    let event = ColdChainBreachReportedEvent {
        batch_id,
        reporter,
        severity,
        description,
        timestamp,
    };
    env.events().publish(("cold_chain_breach_reported",), event);
}

pub fn emit_inventory_updated(
    env: &Env,
    batch_id: String,
    old_quantity: u32,
    new_quantity: u32,
    change_reason: String,
    timestamp: u64,
) {
    let event = InventoryUpdatedEvent {
        batch_id,
        old_quantity,
        new_quantity,
        change_reason,
        timestamp,
    };
    env.events().publish(("inventory_updated",), event);
}