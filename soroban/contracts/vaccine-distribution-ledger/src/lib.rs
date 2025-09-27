#![no_std]

mod error;
mod events;
mod storage;
mod distribution_storage;
mod ledger;
mod distribution;
mod utils;

#[cfg(test)]
mod test;

use soroban_sdk::{
    contract, contractimpl, Address, Env, String, Vec,
};

pub use error::*;
pub use events::*;
pub use ledger::*;
pub use distribution::*;

#[contract]
pub struct VaccineDistributionLedger;

#[contractimpl]
impl VaccineDistributionLedger {
    /// Initialize the contract with admin
    pub fn initialize(env: Env, admin: Address) -> Result<(), ContractError> {
        if storage::has_admin(&env) {
            return Err(ContractError::AlreadyInitialized);
        }
        admin.require_auth();
        storage::set_admin(&env, &admin);
        Ok(())
    }

    /// Initialize a new vaccine batch
    pub fn initialize_batch(
        env: Env,
        batch_id: String,
        manufacturer: Address,
        vaccine_type: String,
        production_date: u64,
        quantity: u32,
        expiry_date: u64,
    ) -> Result<(), ContractError> {
        manufacturer.require_auth();
        ledger::initialize_batch(&env, batch_id, &manufacturer, vaccine_type, production_date, quantity, expiry_date)
    }

    /// Log a distribution event
    pub fn log_distribution(
        env: Env,
        batch_id: String,
        distributor: Address,
        destination: String,
        quantity: u32,
        temperature_log: Option<String>,
    ) -> Result<(), ContractError> {
        distributor.require_auth();
        distribution::log_distribution_event(&env, batch_id, &distributor, destination, quantity, temperature_log)
    }

    /// Verify and log vaccine administration
    pub fn verify_administration(
        env: Env,
        batch_id: String,
        administrator: Address,
        patient_id: String,
        administered_quantity: u32,
        location: String,
    ) -> Result<(), ContractError> {
        administrator.require_auth();
        distribution::verify_administration(&env, batch_id, &administrator, patient_id, administered_quantity, location)
    }

    /// Update batch status
    pub fn update_batch_status(
        env: Env,
        batch_id: String,
        updater: Address,
        new_status: distribution_storage::BatchStatus,
        notes: Option<String>,
    ) -> Result<(), ContractError> {
        updater.require_auth();
        ledger::update_batch_status(&env, batch_id, &updater, new_status, notes)
    }

    /// Get batch history
    pub fn get_history(
        env: Env,
        batch_id: String,
        offset: u32,
        limit: u32,
    ) -> Result<Vec<distribution_storage::DistributionEvent>, ContractError> {
        ledger::get_batch_history(&env, batch_id, offset, limit)
    }

    /// Get batch details
    pub fn get_batch(
        env: Env,
        batch_id: String,
    ) -> Result<distribution_storage::VaccineBatch, ContractError> {
        ledger::get_batch(&env, batch_id)
    }

    /// Check current inventory for a batch
    pub fn inventory_check(
        env: Env,
        batch_id: String,
    ) -> Result<u32, ContractError> {
        ledger::get_batch_inventory(&env, batch_id)
    }

    /// Get batches by manufacturer
    pub fn get_manufacturer_batches(
        env: Env,
        manufacturer: Address,
        offset: u32,
        limit: u32,
    ) -> Result<Vec<String>, ContractError> {
        ledger::get_manufacturer_batches(&env, &manufacturer, offset, limit)
    }

    /// Get batches by status
    pub fn get_batches_by_status(
        env: Env,
        status: distribution_storage::BatchStatus,
        offset: u32,
        limit: u32,
    ) -> Result<Vec<String>, ContractError> {
        ledger::get_batches_by_status(&env, status, offset, limit)
    }

    /// Report cold chain breach
    pub fn report_cold_chain_breach(
        env: Env,
        batch_id: String,
        reporter: Address,
        severity: String,
        description: String,
    ) -> Result<(), ContractError> {
        reporter.require_auth();
        distribution::report_cold_chain_breach(&env, batch_id, &reporter, severity, description)
    }

    /// Get administration records for a batch
    pub fn get_administration_records(
        env: Env,
        batch_id: String,
        offset: u32,
        limit: u32,
    ) -> Result<Vec<distribution_storage::AdministrationRecord>, ContractError> {
        ledger::get_administration_records(&env, batch_id, offset, limit)
    }
}