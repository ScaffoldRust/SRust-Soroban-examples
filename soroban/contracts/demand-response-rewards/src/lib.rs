#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, panic_with_error, symbol_short, vec,
    Address, Bytes, Env, IntoVal, Map, Symbol, TryIntoVal, Val, Vec,
};

pub mod events;
pub mod rewards;
pub mod utils;

#[cfg(test)]
mod tests;

use events::{Event, EventState};
use rewards::{safe_mul, RewardError};
use utils::safe_math;

#[contracttype]
#[derive(Copy, Clone)]
#[repr(u32)]
pub enum DataKey {
    Initialized = 0,
    Admin = 1,
    Operators = 2,
    Consumers = 3,
    Events = 4,
    Participations = 5,
    RewardRate = 6,
    EventCount = 7,
    MeterContract = 8,
}

#[contracttype]
#[derive(Clone)]
pub struct Participation {
    pub event_id: u64,
    pub reduction: i128,
    pub status: Symbol,
    pub timestamp: u64,
    pub claimed: bool,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[repr(u32)]
pub enum ContractError {
    AlreadyInitialized = 1,
    NotAuthorized = 2,
    InvalidInput = 3,
    EventNotFound = 4,
    NotRegistered = 5,
    ParticipationNotVerified = 6,
    InsufficientRewards = 7,
    MathOverflow = 8,
    NotInitialized = 9,
}

#[contract]
pub struct DemandResponseRewards;

#[contractimpl]
impl DemandResponseRewards {
    // Initialize the contract with admin, reward rate, and meter contract
    pub fn initialize(env: Env, admin: Address, reward_rate: i128, meter_contract: Address) {
        admin.require_auth();
        if env.storage().instance().has(&DataKey::Initialized) {
            panic!("already initialized");
        }
        env.storage().instance().set(&DataKey::Initialized, &true);
        env.storage().instance().set(&DataKey::Admin, &admin);
        let mut operators = Map::new(&env);
        operators.set(admin.clone(), true);
        env.storage()
            .instance()
            .set(&DataKey::Operators, &operators);
        env.storage()
            .instance()
            .set(&DataKey::RewardRate, &reward_rate);
        env.storage()
            .instance()
            .set(&DataKey::MeterContract, &meter_contract);
        env.events().publish(
            (symbol_short!("init"), admin),
            (reward_rate, meter_contract),
        );
    }

    // Register a consumer
    pub fn register_consumer(env: Env, consumer: Address) {
        if !env.storage().instance().has(&DataKey::Initialized) {
            panic_with_error!(env, ContractError::NotInitialized);
        }
        consumer.require_auth();
        let consumers: Map<Address, bool> = env
            .storage()
            .instance()
            .get(&DataKey::Consumers)
            .unwrap_or_else(|| Map::new(&env));
        if consumers.contains_key(consumer.clone()) {
            panic!("consumer already registered");
        }
        let mut new_consumers = consumers;
        new_consumers.set(consumer.clone(), true);
        env.storage()
            .instance()
            .set(&DataKey::Consumers, &new_consumers);
        env.events()
            .publish((symbol_short!("reg_cons"), consumer), ());
    }

    // Start a demand response event
    pub fn start_event(
        env: Env,
        caller: Address,
        target_reduction: i128,
        duration: u64,
        rewards_pool: i128,
    ) -> u64 {
        if !env.storage().instance().has(&DataKey::Initialized) {
            panic_with_error!(env, ContractError::NotInitialized);
        }
        caller.require_auth();
        let operators: Map<Address, bool> =
            env.storage().instance().get(&DataKey::Operators).unwrap();
        if !operators.get(caller.clone()).unwrap_or(false) {
            panic!("not authorized as operator");
        }
        if target_reduction <= 0 || rewards_pool <= 0 || duration == 0 {
            panic!("invalid parameters");
        }
        let event_id = utils::generate_event_id(&env);
        let event = Event {
            id: event_id,
            target_reduction,
            duration,
            start_time: env.ledger().timestamp(),
            status: EventState::Active,
            rewards_pool,
            reward_rate: 0i128,
        };
        let mut events: Map<u64, Event> = env
            .storage()
            .instance()
            .get(&DataKey::Events)
            .unwrap_or_else(|| Map::new(&env));
        events.set(event_id, event.clone());
        env.storage().instance().set(&DataKey::Events, &events);
        env.events().publish(
            (symbol_short!("evt_start"), caller),
            (event_id, target_reduction, duration),
        );
        event_id
    }

    // Verify participation using meter contract for reduction data
    pub fn verify_participation(
        env: Env,
        caller: Address,
        event_id: u64,
        oracle_data: Bytes, // Placeholder for signed oracle data
    ) {
        if !env.storage().instance().has(&DataKey::Initialized) {
            panic_with_error!(env, ContractError::NotInitialized);
        }
        caller.require_auth();
        let consumers: Map<Address, bool> = env
            .storage()
            .instance()
            .get(&DataKey::Consumers)
            .unwrap_or_else(|| Map::new(&env));
        if !consumers.get(caller.clone()).unwrap_or(false) {
            panic!("consumer not registered");
        }
        let events: Map<u64, Event> = env
            .storage()
            .instance()
            .get(&DataKey::Events)
            .unwrap_or_else(|| Map::new(&env));
        let event = events
            .get(event_id)
            .ok_or(ContractError::EventNotFound)
            .unwrap();
        if event.status != EventState::Active {
            panic!("event not active");
        }
        // Get reduction from meter contract
        let reduction = Self::invoke_meter_contract(env.clone(), caller.clone(), event_id);
        if reduction <= 0 {
            panic!("invalid reduction from meter");
        }
        // Simplified validation - in production, verify oracle_data signature
        utils::validate_oracle_data(&env, oracle_data, event_id, caller.clone(), reduction);
        let mut participations: Map<Address, Vec<Participation>> = env
            .storage()
            .instance()
            .get(&DataKey::Participations)
            .unwrap_or_else(|| Map::new(&env));
        let consumer_parts = participations
            .get(caller.clone())
            .unwrap_or_else(|| Vec::new(&env));
        let participation = Participation {
            event_id,
            reduction,
            status: symbol_short!("verif"),
            timestamp: env.ledger().timestamp(),
            claimed: false,
        };
        let mut new_parts = consumer_parts;
        new_parts.push_back(participation);
        participations.set(caller.clone(), new_parts);
        env.storage()
            .instance()
            .set(&DataKey::Participations, &participations);
        env.events()
            .publish((symbol_short!("part_ver"), caller), (event_id, reduction));
    }

    // Finalize event and set reward rate for claiming
    pub fn distribute_rewards(
        env: Env,
        caller: Address,
        event_id: u64,
    ) -> Result<i128, RewardError> {
        if !env.storage().instance().has(&DataKey::Initialized) {
            panic_with_error!(env, ContractError::NotInitialized);
        }
        caller.require_auth();
        let operators: Map<Address, bool> =
            env.storage().instance().get(&DataKey::Operators).unwrap();
        if !operators.get(caller.clone()).unwrap_or(false) {
            return Err(RewardError::NotAuthorized);
        }
        let mut events: Map<u64, Event> = env
            .storage()
            .instance()
            .get(&DataKey::Events)
            .unwrap_or_else(|| Map::new(&env));
        let mut event = events.get(event_id).ok_or(RewardError::EventNotFound)?;
        if event.status != EventState::Active
            || env.ledger().timestamp() < event.start_time + event.duration
        {
            return Err(RewardError::EventNotCompleted);
        }
        event.status = EventState::Completed;
        let total_reduction = rewards::calculate_total_reduction(&env, event_id);
        if total_reduction <= 0 {
            return Err(RewardError::NoParticipation);
        }
        let global_reward_rate: i128 = env.storage().instance().get(&DataKey::RewardRate).unwrap();
        let total_rewards = safe_math::mul_i128(event.rewards_pool, global_reward_rate, 100)
            .map_err(|_| RewardError::MathError)?;
        event.reward_rate = safe_math::div_i128(total_rewards, total_reduction)
            .map_err(|_| RewardError::MathError)?;
        events.set(event_id, event.clone());
        env.storage().instance().set(&DataKey::Events, &events);
        env.events().publish(
            (symbol_short!("evt_final"), caller),
            (event_id, event.reward_rate),
        );
        Ok(event.reward_rate)
    }

    // Claim reward for a specific event
    pub fn claim_reward(env: Env, caller: Address, event_id: u64) -> Result<i128, ContractError> {
        if !env.storage().instance().has(&DataKey::Initialized) {
            panic_with_error!(env, ContractError::NotInitialized);
        }
        caller.require_auth();
        if !utils::is_consumer(&env, caller.clone()) {
            panic_with_error!(env, ContractError::NotRegistered);
        }
        let events: Map<u64, Event> = env
            .storage()
            .instance()
            .get(&DataKey::Events)
            .unwrap_or_else(|| Map::new(&env));
        let event = events.get(event_id).ok_or(ContractError::EventNotFound)?;
        if event.status != EventState::Completed || event.reward_rate <= 0 {
            panic_with_error!(env, ContractError::InvalidInput);
        }
        let mut participations: Map<Address, Vec<Participation>> = env
            .storage()
            .instance()
            .get(&DataKey::Participations)
            .unwrap_or_else(|| Map::new(&env));
        let mut consumer_parts = participations
            .get(caller.clone())
            .unwrap_or_else(|| Vec::new(&env));
        let mut found = false;
        let mut reward = 0i128;
        for (i, part) in consumer_parts.iter().enumerate() {
            if part.event_id == event_id && part.status == symbol_short!("verif") && !part.claimed {
                reward = safe_mul(part.reduction, event.reward_rate)
                    .map_err(|_| ContractError::MathOverflow)?;
                let mut updated_part = part.clone();
                updated_part.claimed = true;
                consumer_parts.set(i as u32, updated_part);
                found = true;
                break;
            }
        }
        if !found {
            panic_with_error!(env, ContractError::ParticipationNotVerified);
        }
        participations.set(caller.clone(), consumer_parts);
        env.storage()
            .instance()
            .set(&DataKey::Participations, &participations);
        env.events()
            .publish((symbol_short!("rwd_claim"), caller), (event_id, reward));
        Ok(reward)
    }

    // Invoke external meter contract to get reduction data
    pub fn invoke_meter_contract(env: Env, consumer: Address, event_id: u64) -> i128 {
        if !env.storage().instance().has(&DataKey::Initialized) {
            panic_with_error!(env, ContractError::NotInitialized);
        }
        let meter_contract: Address = env
            .storage()
            .instance()
            .get(&DataKey::MeterContract)
            .unwrap();
        let func = symbol_short!("get_red");
        let args_vec = vec![
            &env,
            consumer.clone().into_val(&env),
            event_id.into_val(&env),
        ];
        let result_val: Val = env.invoke_contract(&meter_contract, &func, args_vec);
        let reduction: i128 = result_val
            .try_into_val(&env)
            .unwrap_or_else(|_| panic_with_error!(env, ContractError::InvalidInput));
        env.events()
            .publish((symbol_short!("mtr_inv"), consumer), (event_id, reduction));
        reduction
    }
}
