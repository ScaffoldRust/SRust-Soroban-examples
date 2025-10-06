#![no_std]

mod demand;
mod incentives;
mod utils;

#[cfg(test)]
mod test;

use soroban_sdk::{contract, contractimpl, Address, Env, Map, Vec};

use crate::utils::*;

#[contract]
pub struct GridLoadBalancingIncentives;

#[contractimpl]
impl GridLoadBalancingIncentives {
    /// Initialize the grid load balancing incentives contract
    pub fn initialize(
        env: Env,
        admin: Address,
        token_contract: Address,
    ) -> Result<(), IncentiveError> {
        if env.storage().instance().has(&DataKey::Initialized) {
            return Err(IncentiveError::AlreadyInitialized);
        }

        admin.require_auth();

        env.storage().instance().set(&DataKey::Initialized, &true);
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::TokenContract, &token_contract);
        env.storage().instance().set(&DataKey::NextEventId, &1u64);

        Ok(())
    }

    /// Register a grid operator
    pub fn register_grid_operator(env: Env, grid_operator: Address) -> Result<(), IncentiveError> {
        Self::check_initialized(&env)?;
        grid_operator.require_auth();

        let mut grid_operators: Map<Address, bool> = env
            .storage()
            .instance()
            .get(&DataKey::GridOperators)
            .unwrap_or_else(|| Map::new(&env));

        grid_operators.set(grid_operator, true);
        env.storage()
            .instance()
            .set(&DataKey::GridOperators, &grid_operators);

        Ok(())
    }

    /// Register a consumer for demand response programs
    pub fn register_consumer(env: Env, consumer: Address) -> Result<(), IncentiveError> {
        Self::check_initialized(&env)?;
        consumer.require_auth();

        let mut consumers: Map<Address, bool> = env
            .storage()
            .instance()
            .get(&DataKey::Consumers)
            .unwrap_or_else(|| Map::new(&env));

        consumers.set(consumer, true);
        env.storage().instance().set(&DataKey::Consumers, &consumers);

        Ok(())
    }

    /// Start a demand response event with target reductions
    pub fn start_event(
        env: Env,
        grid_operator: Address,
        target_reduction_kw: u64,
        reward_per_kw: u64,
        duration_seconds: u64,
    ) -> Result<u64, IncentiveError> {
        Self::check_initialized(&env)?;
        grid_operator.require_auth();

        Self::validate_grid_operator(&env, &grid_operator)?;
        Self::validate_event_params(target_reduction_kw, reward_per_kw, duration_seconds)?;

        demand::start_event(
            &env,
            grid_operator,
            target_reduction_kw,
            reward_per_kw,
            duration_seconds,
        )
    }

    /// Verify consumer load reduction via meter data
    pub fn verify_reduction(
        env: Env,
        event_id: u64,
        consumer: Address,
        baseline_usage_kw: u64,
        actual_usage_kw: u64,
        meter_reading_start: u64,
        meter_reading_end: u64,
    ) -> Result<u64, IncentiveError> {
        Self::check_initialized(&env)?;
        consumer.require_auth();

        Self::validate_consumer(&env, &consumer)?;
        Self::validate_meter_readings(meter_reading_start, meter_reading_end)?;

        demand::verify_reduction(
            &env,
            event_id,
            consumer,
            baseline_usage_kw,
            actual_usage_kw,
            meter_reading_start,
            meter_reading_end,
        )
    }

    /// Distribute rewards to participating consumers
    pub fn distribute_rewards(
        env: Env,
        event_id: u64,
        distributor: Address,
    ) -> Result<(), IncentiveError> {
        Self::check_initialized(&env)?;
        distributor.require_auth();

        incentives::distribute_rewards(&env, event_id, distributor)
    }

    /// Get demand response event details
    pub fn get_event(env: Env, event_id: u64) -> Result<DemandResponseEvent, IncentiveError> {
        Self::check_initialized(&env)?;
        demand::get_event(&env, event_id)
    }

    /// Get participation record details
    pub fn get_participation(
        env: Env,
        participation_id: u64,
    ) -> Result<ParticipationRecord, IncentiveError> {
        Self::check_initialized(&env)?;
        incentives::get_participation(&env, participation_id)
    }

    /// Get participation history for a consumer
    pub fn get_consumer_participations(
        env: Env,
        consumer: Address,
    ) -> Result<Vec<ParticipationRecord>, IncentiveError> {
        Self::check_initialized(&env)?;
        incentives::get_consumer_participations(&env, consumer)
    }

    fn check_initialized(env: &Env) -> Result<(), IncentiveError> {
        if !env.storage().instance().has(&DataKey::Initialized) {
            return Err(IncentiveError::NotInitialized);
        }
        Ok(())
    }

    fn validate_grid_operator(env: &Env, grid_operator: &Address) -> Result<(), IncentiveError> {
        let grid_operators: Map<Address, bool> = env
            .storage()
            .instance()
            .get(&DataKey::GridOperators)
            .unwrap_or_else(|| Map::new(env));

        if !grid_operators.contains_key(grid_operator.clone()) {
            return Err(IncentiveError::GridOperatorNotRegistered);
        }

        Ok(())
    }

    fn validate_consumer(env: &Env, consumer: &Address) -> Result<(), IncentiveError> {
        let consumers: Map<Address, bool> = env
            .storage()
            .instance()
            .get(&DataKey::Consumers)
            .unwrap_or_else(|| Map::new(env));

        if !consumers.contains_key(consumer.clone()) {
            return Err(IncentiveError::ConsumerNotRegistered);
        }

        Ok(())
    }

    fn validate_event_params(
        target_reduction_kw: u64,
        reward_per_kw: u64,
        duration_seconds: u64,
    ) -> Result<(), IncentiveError> {
        if target_reduction_kw == 0 || reward_per_kw == 0 || duration_seconds == 0 {
            return Err(IncentiveError::InvalidInput);
        }
        Ok(())
    }

    fn validate_meter_readings(
        meter_reading_start: u64,
        meter_reading_end: u64,
    ) -> Result<(), IncentiveError> {
        if meter_reading_start >= meter_reading_end {
            return Err(IncentiveError::InvalidMeterReading);
        }
        Ok(())
    }
}