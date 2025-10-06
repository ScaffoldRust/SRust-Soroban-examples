#![no_std]

mod payment;
mod sharing;
mod utils;

#[cfg(test)]
mod test;

use soroban_sdk::{contract, contractimpl, Address, Env, Map, Vec};

use crate::utils::*;

#[contract]
pub struct PeerToPeerEnergySharing;

#[contractimpl]
impl PeerToPeerEnergySharing {
    /// Initialize the peer-to-peer energy sharing contract
    pub fn initialize(
        env: Env,
        admin: Address,
        token_contract: Address,
    ) -> Result<(), SharingError> {
        if env.storage().instance().has(&DataKey::Initialized) {
            return Err(SharingError::AlreadyInitialized);
        }

        admin.require_auth();

        env.storage().instance().set(&DataKey::Initialized, &true);
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::TokenContract, &token_contract);
        env.storage().instance().set(&DataKey::NextAgreementId, &1u64);
        env.storage().instance().set(&DataKey::NextTransactionId, &1u64);

        // Initialize empty maps
        let prosumers: Map<Address, bool> = Map::new(&env);
        let agreements: Map<u64, EnergyAgreement> = Map::new(&env);
        let transactions: Map<u64, EnergyTransaction> = Map::new(&env);

        env.storage().instance().set(&DataKey::Prosumers, &prosumers);
        env.storage()
            .instance()
            .set(&DataKey::Agreements, &agreements);
        env.storage()
            .instance()
            .set(&DataKey::Transactions, &transactions);

        Ok(())
    }

    /// Register a prosumer (both producer and consumer)
    pub fn register_prosumer(env: Env, prosumer: Address) -> Result<(), SharingError> {
        Self::check_initialized(&env)?;
        prosumer.require_auth();

        let mut prosumers: Map<Address, bool> = env
            .storage()
            .instance()
            .get(&DataKey::Prosumers)
            .unwrap_or_else(|| Map::new(&env));

        prosumers.set(prosumer, true);
        env.storage().instance().set(&DataKey::Prosumers, &prosumers);

        Ok(())
    }

    /// Create an energy sharing agreement between prosumers
    pub fn create_agreement(
        env: Env,
        provider: Address,
        consumer: Address,
        energy_amount_kwh: u64,
        price_per_kwh: u64,
        delivery_deadline: u64,
    ) -> Result<u64, SharingError> {
        Self::check_initialized(&env)?;
        provider.require_auth();

        Self::validate_prosumers(&env, &provider, &consumer)?;
        Self::validate_agreement_params(energy_amount_kwh, price_per_kwh, delivery_deadline)?;

        if provider == consumer {
            return Err(SharingError::SelfSharingNotAllowed);
        }

        sharing::create_agreement(
            &env,
            provider,
            consumer,
            energy_amount_kwh,
            price_per_kwh,
            delivery_deadline,
        )
    }

    /// Deliver energy and record meter data
    pub fn deliver_energy(
        env: Env,
        agreement_id: u64,
        energy_delivered_kwh: u64,
        meter_reading: u64,
        provider: Address,
    ) -> Result<u64, SharingError> {
        Self::check_initialized(&env)?;
        provider.require_auth();

        sharing::deliver_energy(&env, agreement_id, energy_delivered_kwh, meter_reading, provider)
    }

    /// Settle payment for delivered energy
    pub fn settle_payment(
        env: Env,
        transaction_id: u64,
        settler: Address,
    ) -> Result<(), SharingError> {
        Self::check_initialized(&env)?;
        settler.require_auth();

        payment::settle_payment(&env, transaction_id, settler)
    }

    /// Get agreement details
    pub fn get_agreement(env: Env, agreement_id: u64) -> Result<EnergyAgreement, SharingError> {
        Self::check_initialized(&env)?;
        sharing::get_agreement(&env, agreement_id)
    }

    /// Get transaction details
    pub fn get_transaction(
        env: Env,
        transaction_id: u64,
    ) -> Result<EnergyTransaction, SharingError> {
        Self::check_initialized(&env)?;
        payment::get_transaction(&env, transaction_id)
    }

    /// Get transaction history for a prosumer
    pub fn get_transaction_history(
        env: Env,
        prosumer: Address,
    ) -> Result<Vec<EnergyTransaction>, SharingError> {
        Self::check_initialized(&env)?;
        payment::get_transaction_history(&env, prosumer)
    }

    fn check_initialized(env: &Env) -> Result<(), SharingError> {
        if !env.storage().instance().has(&DataKey::Initialized) {
            return Err(SharingError::NotInitialized);
        }
        Ok(())
    }

    fn validate_prosumers(
        env: &Env,
        provider: &Address,
        consumer: &Address,
    ) -> Result<(), SharingError> {
        let prosumers: Map<Address, bool> = env
            .storage()
            .instance()
            .get(&DataKey::Prosumers)
            .unwrap_or_else(|| Map::new(env));

        if !prosumers.contains_key(provider.clone())
            || !prosumers.contains_key(consumer.clone())
        {
            return Err(SharingError::ProsumerNotRegistered);
        }

        Ok(())
    }

    fn validate_agreement_params(
        energy_amount_kwh: u64,
        price_per_kwh: u64,
        delivery_deadline: u64,
    ) -> Result<(), SharingError> {
        if energy_amount_kwh == 0 || price_per_kwh == 0 || delivery_deadline == 0 {
            return Err(SharingError::InvalidInput);
        }
        Ok(())
    }
}