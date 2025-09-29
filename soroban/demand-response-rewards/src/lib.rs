#![no_std]

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, Address, Env, Map, Symbol, Vec, symbol_short, Bytes, panic_with_error, Val, IntoVal, TryIntoVal, vec};

pub mod rewards;
pub mod events;
pub mod utils;

use rewards::{RewardError, safe_mul};
use events::{Event, EventState};
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
        env.storage().instance().set(&DataKey::Operators, &operators);
        env.storage().instance().set(&DataKey::RewardRate, &reward_rate);
        env.storage().instance().set(&DataKey::MeterContract, &meter_contract);
        env.events().publish(
            (symbol_short!("init"), admin),
            (reward_rate, meter_contract)
        );
    }

    // Register a consumer
    pub fn register_consumer(env: Env, consumer: Address) {
        if !env.storage().instance().has(&DataKey::Initialized) {
            panic_with_error!(env, ContractError::NotInitialized);
        }
        consumer.require_auth();
        let consumers: Map<Address, bool> = env.storage().instance().get(&DataKey::Consumers).unwrap_or_else(|| Map::new(&env));
        if consumers.contains_key(consumer.clone()) {
            panic!("consumer already registered");
        }
        let mut new_consumers = consumers;
        new_consumers.set(consumer.clone(), true);
        env.storage().instance().set(&DataKey::Consumers, &new_consumers);
        env.events().publish(
            (symbol_short!("reg_cons"), consumer),
            ()
        );
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
        let operators: Map<Address, bool> = env.storage().instance().get(&DataKey::Operators).unwrap();
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
        let mut events: Map<u64, Event> = env.storage().instance().get(&DataKey::Events).unwrap_or_else(|| Map::new(&env));
        events.set(event_id, event.clone());
        env.storage().instance().set(&DataKey::Events, &events);
        env.events().publish(
            (symbol_short!("evt_start"), caller),
            (event_id, target_reduction, duration)
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
        let consumers: Map<Address, bool> = env.storage().instance().get(&DataKey::Consumers).unwrap_or_else(|| Map::new(&env));
        if !consumers.get(caller.clone()).unwrap_or(false) {
            panic!("consumer not registered");
        }
        let events: Map<u64, Event> = env.storage().instance().get(&DataKey::Events).unwrap_or_else(|| Map::new(&env));
        let event = events.get(event_id).ok_or(ContractError::EventNotFound).unwrap();
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
        let mut participations: Map<Address, Vec<Participation>> = env.storage().instance().get(&DataKey::Participations).unwrap_or_else(|| Map::new(&env));
        let consumer_parts = participations.get(caller.clone()).unwrap_or_else(|| Vec::new(&env));
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
        env.storage().instance().set(&DataKey::Participations, &participations);
        env.events().publish(
            (symbol_short!("part_ver"), caller),
            (event_id, reduction)
        );
    }

    // Finalize event and set reward rate for claiming
    pub fn distribute_rewards(env: Env, caller: Address, event_id: u64) -> Result<i128, RewardError> {
        if !env.storage().instance().has(&DataKey::Initialized) {
            panic_with_error!(env, ContractError::NotInitialized);
        }
        caller.require_auth();
        let operators: Map<Address, bool> = env.storage().instance().get(&DataKey::Operators).unwrap();
        if !operators.get(caller.clone()).unwrap_or(false) {
            return Err(RewardError::NotAuthorized);
        }
        let mut events: Map<u64, Event> = env.storage().instance().get(&DataKey::Events).unwrap_or_else(|| Map::new(&env));
        let mut event = events.get(event_id).ok_or(RewardError::EventNotFound)?;
        if event.status != EventState::Active || env.ledger().timestamp() < event.start_time + event.duration {
            return Err(RewardError::EventNotCompleted);
        }
        event.status = EventState::Completed;
        let total_reduction = rewards::calculate_total_reduction(&env, event_id);
        if total_reduction <= 0 {
            return Err(RewardError::NoParticipation);
        }
        let global_reward_rate: i128 = env.storage().instance().get(&DataKey::RewardRate).unwrap();
        let total_rewards = safe_math::mul_i128(event.rewards_pool, global_reward_rate, 100).map_err(|_| RewardError::MathError)?;
        event.reward_rate = safe_math::div_i128(total_rewards, total_reduction).map_err(|_| RewardError::MathError)?;
        events.set(event_id, event.clone());
        env.storage().instance().set(&DataKey::Events, &events);
        env.events().publish(
            (symbol_short!("evt_final"), caller),
            (event_id, event.reward_rate)
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
        let events: Map<u64, Event> = env.storage().instance().get(&DataKey::Events).unwrap_or_else(|| Map::new(&env));
        let event = events.get(event_id).ok_or(ContractError::EventNotFound)?;
        if event.status != EventState::Completed || event.reward_rate <= 0 {
            panic_with_error!(env, ContractError::InvalidInput);
        }
        let mut participations: Map<Address, Vec<Participation>> = env.storage().instance().get(&DataKey::Participations).unwrap_or_else(|| Map::new(&env));
        let mut consumer_parts = participations.get(caller.clone()).unwrap_or_else(|| Vec::new(&env));
        let mut found = false;
        let mut reward = 0i128;
        for (i, part) in consumer_parts.iter().enumerate() {
            if part.event_id == event_id && part.status == symbol_short!("verif") && !part.claimed {
                reward = safe_mul(part.reduction, event.reward_rate).map_err(|_| ContractError::MathOverflow)?;
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
        env.storage().instance().set(&DataKey::Participations, &participations);
        env.events().publish(
            (symbol_short!("rwd_claim"), caller),
            (event_id, reward)
        );
        Ok(reward)
    }

    // Invoke external meter contract to get reduction data
    pub fn invoke_meter_contract(env: Env, consumer: Address, event_id: u64) -> i128 {
        if !env.storage().instance().has(&DataKey::Initialized) {
            panic_with_error!(env, ContractError::NotInitialized);
        }
        let meter_contract: Address = env.storage().instance().get(&DataKey::MeterContract).unwrap();
        let func = symbol_short!("get_red");
        let args_vec = vec![&env, consumer.clone().into_val(&env), event_id.into_val(&env)];
        let result_val: Val = env.invoke_contract(&meter_contract, &func, args_vec);
        let reduction: i128 = result_val.try_into_val(&env).unwrap_or_else(|_| panic_with_error!(env, ContractError::InvalidInput));
        env.events().publish(
            (symbol_short!("mtr_inv"), consumer),
            (event_id, reduction)
        );
        reduction
    }

}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{Address, Env, symbol_short};
    use soroban_sdk::testutils::{Address as _, Ledger as _};

    #[contract]
    pub struct MockMeter;

    #[contractimpl]
    impl MockMeter {
        pub fn set_red(env: Env, consumer: Address, event_id: u64, red: i128) {
            env.storage().instance().set(&(consumer, event_id), &red);
        }

        pub fn get_red(env: Env, consumer: Address, event_id: u64) -> i128 {
            env.storage().instance().get(&(consumer, event_id)).unwrap_or(0)
        }
    }

    #[test]
    fn test_initialize() {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let reward_rate = 10i128;
        let meter_contract = Address::generate(&env);
        let client = DemandResponseRewardsClient::new(&env, &env.register_contract(None, DemandResponseRewards));
        client.initialize(&admin, &reward_rate, &meter_contract);

        let initialized: bool = env.as_contract(&client.address, || {
            env.storage().instance().get(&DataKey::Initialized).unwrap()
        });
        assert!(initialized);

        let stored_admin: Address = env.as_contract(&client.address, || {
            env.storage().instance().get(&DataKey::Admin).unwrap()
        });
        assert_eq!(stored_admin, admin);

        let stored_rate: i128 = env.as_contract(&client.address, || {
            env.storage().instance().get(&DataKey::RewardRate).unwrap()
        });
        assert_eq!(stored_rate, reward_rate);

        let stored_meter: Address = env.as_contract(&client.address, || {
            env.storage().instance().get(&DataKey::MeterContract).unwrap()
        });
        assert_eq!(stored_meter, meter_contract);

        let operators: Map<Address, bool> = env.as_contract(&client.address, || {
            env.storage().instance().get(&DataKey::Operators).unwrap()
        });
        assert!(operators.get(admin).unwrap_or(false));
    }

    #[test]
    #[should_panic(expected = "already initialized")]
    fn test_initialize_twice() {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let meter_contract = Address::generate(&env);
        let client = DemandResponseRewardsClient::new(&env, &env.register_contract(None, DemandResponseRewards));
        client.initialize(&admin, &10, &meter_contract);
        client.initialize(&admin, &10, &meter_contract);
    }

    #[test]
    fn test_register_consumer() {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let consumer = Address::generate(&env);
        let meter_contract = Address::generate(&env);
        let client = DemandResponseRewardsClient::new(&env, &env.register_contract(None, DemandResponseRewards));
        client.initialize(&admin, &10, &meter_contract);
        client.register_consumer(&consumer);

        let consumers: Map<Address, bool> = env.as_contract(&client.address, || {
            env.storage().instance().get(&DataKey::Consumers).unwrap()
        });
        assert!(consumers.get(consumer).unwrap());
    }

    #[test]
    #[should_panic(expected = "consumer already registered")]
    fn test_register_consumer_twice() {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let consumer = Address::generate(&env);
        let meter_contract = Address::generate(&env);
        let client = DemandResponseRewardsClient::new(&env, &env.register_contract(None, DemandResponseRewards));
        client.initialize(&admin, &10, &meter_contract);
        client.register_consumer(&consumer);
        client.register_consumer(&consumer);
    }

    #[test]
    fn test_start_event() {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let meter_contract = Address::generate(&env);
        let client = DemandResponseRewardsClient::new(&env, &env.register_contract(None, DemandResponseRewards));
        client.initialize(&admin, &10, &meter_contract);
        let event_id = client.start_event(&admin, &100, &3600u64, &1000);

        let events: Map<u64, Event> = env.as_contract(&client.address, || {
            env.storage().instance().get(&DataKey::Events).unwrap()
        });
        let event = events.get(event_id).unwrap();
        assert_eq!(event.target_reduction, 100i128);
        assert_eq!(event.status, EventState::Active);
        assert_eq!(event.duration, 3600u64);
        assert_eq!(event.rewards_pool, 1000i128);
    }

    #[test]
    #[should_panic(expected = "not authorized as operator")]
    fn test_start_event_unauthorized() {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let unauthorized = Address::generate(&env);
        let meter_contract = Address::generate(&env);
        let client = DemandResponseRewardsClient::new(&env, &env.register_contract(None, DemandResponseRewards));
        client.initialize(&admin, &10, &meter_contract);
        client.start_event(&unauthorized, &100, &3600u64, &1000);
    }

    #[test]
    #[should_panic(expected = "invalid parameters")]
    fn test_start_event_negative_target() {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let meter_contract = Address::generate(&env);
        let client = DemandResponseRewardsClient::new(&env, &env.register_contract(None, DemandResponseRewards));
        client.initialize(&admin, &10, &meter_contract);
        client.start_event(&admin, &-100, &3600u64, &1000);
    }

    #[test]
    #[should_panic(expected = "invalid parameters")]
    fn test_start_event_zero_duration() {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let meter_contract = Address::generate(&env);
        let client = DemandResponseRewardsClient::new(&env, &env.register_contract(None, DemandResponseRewards));
        client.initialize(&admin, &10, &meter_contract);
        client.start_event(&admin, &100, &0u64, &1000);
    }

    #[test]
    #[should_panic(expected = "invalid parameters")]
    fn test_start_event_negative_rewards() {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let meter_contract = Address::generate(&env);
        let client = DemandResponseRewardsClient::new(&env, &env.register_contract(None, DemandResponseRewards));
        client.initialize(&admin, &10, &meter_contract);
        client.start_event(&admin, &100, &3600u64, &-1000);
    }

    #[test]
    fn test_verify_participation() {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let consumer = Address::generate(&env);
        let meter_contract = env.register_contract(None, MockMeter);
        let client = DemandResponseRewardsClient::new(&env, &env.register_contract(None, DemandResponseRewards));
        client.initialize(&admin, &10, &meter_contract);
        client.register_consumer(&consumer);
        let event_id = client.start_event(&admin, &100, &3600u64, &1000);
        let oracle_data = Bytes::from_slice(&env, b"mock_oracle_data");
        let mock_meter_client = MockMeterClient::new(&env, &meter_contract);
        mock_meter_client.set_red(&consumer, &event_id, &50);
        client.verify_participation(&consumer, &event_id, &oracle_data);

        let participations: Map<Address, Vec<Participation>> = env.as_contract(&client.address, || {
            env.storage().instance().get(&DataKey::Participations).unwrap()
        });
        let parts = participations.get(consumer).unwrap();
        assert_eq!(parts.len(), 1);
        assert_eq!(parts.get(0).unwrap().reduction, 50i128);
        assert_eq!(parts.get(0).unwrap().status, symbol_short!("verif"));
        assert_eq!(parts.get(0).unwrap().event_id, event_id);
    }

    #[test]
    #[should_panic(expected = "consumer not registered")]
    fn test_verify_participation_not_registered() {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let consumer = Address::generate(&env);
        let meter_contract = env.register_contract(None, MockMeter);
        let client = DemandResponseRewardsClient::new(&env, &env.register_contract(None, DemandResponseRewards));
        client.initialize(&admin, &10, &meter_contract);
        let event_id = client.start_event(&admin, &100, &3600u64, &1000);
        let oracle_data = Bytes::from_slice(&env, b"mock");
        client.verify_participation(&consumer, &event_id, &oracle_data);
    }

    #[test]
    #[should_panic(expected = "invalid reduction from meter")]
    fn test_verify_participation_zero_reduction() {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let consumer = Address::generate(&env);
        let meter_contract = env.register_contract(None, MockMeter);
        let client = DemandResponseRewardsClient::new(&env, &env.register_contract(None, DemandResponseRewards));
        client.initialize(&admin, &10, &meter_contract);
        client.register_consumer(&consumer);
        let event_id = client.start_event(&admin, &100, &3600u64, &1000);
        let oracle_data = Bytes::from_slice(&env, b"mock_oracle_data");
        let mock_meter_client = MockMeterClient::new(&env, &meter_contract);
        mock_meter_client.set_red(&consumer, &event_id, &0);
        client.verify_participation(&consumer, &event_id, &oracle_data);
    }

    #[test]
    fn test_multiple_participations() {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let consumer = Address::generate(&env);
        let meter_contract = env.register_contract(None, MockMeter);
        let client = DemandResponseRewardsClient::new(&env, &env.register_contract(None, DemandResponseRewards));
        client.initialize(&admin, &10, &meter_contract);
        client.register_consumer(&consumer);
        let event_id = client.start_event(&admin, &100, &3600u64, &1000);
        let mock_meter_client = MockMeterClient::new(&env, &meter_contract);
        let oracle_data1 = Bytes::from_slice(&env, b"mock_oracle_data_1");
        mock_meter_client.set_red(&consumer, &event_id, &30);
        client.verify_participation(&consumer, &event_id, &oracle_data1);
        let oracle_data2 = Bytes::from_slice(&env, b"mock_oracle_data_2");
        mock_meter_client.set_red(&consumer, &event_id, &20);
        client.verify_participation(&consumer, &event_id, &oracle_data2);

        let participations: Map<Address, Vec<Participation>> = env.as_contract(&client.address, || {
            env.storage().instance().get(&DataKey::Participations).unwrap()
        });
        let parts = participations.get(consumer).unwrap();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts.get(0).unwrap().reduction, 30i128);
        assert_eq!(parts.get(1).unwrap().reduction, 20i128);
    }

    #[test]
    fn test_distribute_rewards() {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let consumer = Address::generate(&env);
        let meter_contract = env.register_contract(None, MockMeter);
        let client = DemandResponseRewardsClient::new(&env, &env.register_contract(None, DemandResponseRewards));
        client.initialize(&admin, &10, &meter_contract);
        client.register_consumer(&consumer);
        let event_id = client.start_event(&admin, &100, &3600u64, &1000);
        let events: Map<u64, Event> = env.as_contract(&client.address, || {
            env.storage().instance().get(&DataKey::Events).unwrap()
        });
        let event = events.get(event_id).unwrap();
        let completion_time = event.start_time + event.duration + 1;
        env.ledger().set_timestamp(completion_time);
        let oracle_data = Bytes::from_slice(&env, b"mock");
        let mock_meter_client = MockMeterClient::new(&env, &meter_contract);
        mock_meter_client.set_red(&consumer, &event_id, &50);
        client.verify_participation(&consumer, &event_id, &oracle_data);
        let reward_rate = client.distribute_rewards(&admin, &event_id);
        assert_eq!(reward_rate, 2i128); // (1000 * 10 / 100) / 50 = 100 / 50 = 2

        let events: Map<u64, Event> = env.as_contract(&client.address, || {
            env.storage().instance().get(&DataKey::Events).unwrap()
        });
        let event = events.get(event_id).unwrap();
        assert_eq!(event.status, EventState::Completed);
        assert_eq!(event.reward_rate, 2i128);
    }

    #[test]
    fn test_claim_reward() {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let consumer = Address::generate(&env);
        let meter_contract = env.register_contract(None, MockMeter);
        let client = DemandResponseRewardsClient::new(&env, &env.register_contract(None, DemandResponseRewards));
        client.initialize(&admin, &10, &meter_contract);
        client.register_consumer(&consumer);
        let event_id = client.start_event(&admin, &100, &3600u64, &1000);
        let events: Map<u64, Event> = env.as_contract(&client.address, || {
            env.storage().instance().get(&DataKey::Events).unwrap()
        });
        let event = events.get(event_id).unwrap();
        let completion_time = event.start_time + event.duration + 1;
        env.ledger().set_timestamp(completion_time);
        let oracle_data = Bytes::from_slice(&env, b"mock");
        let mock_meter_client = MockMeterClient::new(&env, &meter_contract);
        mock_meter_client.set_red(&consumer, &event_id, &50);
        client.verify_participation(&consumer, &event_id, &oracle_data);
        let _reward_rate = client.distribute_rewards(&admin, &event_id);
        let claimed_reward = client.claim_reward(&consumer, &event_id);
        assert_eq!(claimed_reward, 100i128); // 50 * 2 = 100

        let participations: Map<Address, Vec<Participation>> = env.as_contract(&client.address, || {
            env.storage().instance().get(&DataKey::Participations).unwrap()
        });
        let parts = participations.get(consumer).unwrap();
        assert_eq!(parts.len(), 1);
        assert!(parts.get(0).unwrap().claimed);
    }

    #[test]
    fn test_distribute_rewards_unauthorized() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().set_timestamp(7200);
        let admin = Address::generate(&env);
        let unauthorized = Address::generate(&env);
        let consumer = Address::generate(&env);
        let meter_contract = env.register_contract(None, MockMeter);
        let client = DemandResponseRewardsClient::new(&env, &env.register_contract(None, DemandResponseRewards));
        client.initialize(&admin, &10, &meter_contract);
        client.register_consumer(&consumer);
        let event_id = client.start_event(&admin, &100, &3600u64, &1000);
        let oracle_data = Bytes::from_slice(&env, b"mock");
        let mock_meter_client = MockMeterClient::new(&env, &meter_contract);
        mock_meter_client.set_red(&consumer, &event_id, &50);
        client.verify_participation(&consumer, &event_id, &oracle_data);
        let result = client.try_distribute_rewards(&unauthorized, &event_id);
        assert!(result.is_err());
    }

    #[test]
    fn test_distribute_rewards_event_not_completed() {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let consumer = Address::generate(&env);
        let meter_contract = env.register_contract(None, MockMeter);
        let client = DemandResponseRewardsClient::new(&env, &env.register_contract(None, DemandResponseRewards));
        client.initialize(&admin, &10, &meter_contract);
        client.register_consumer(&consumer);
        let event_id = client.start_event(&admin, &100, &3600u64, &1000);
        let oracle_data = Bytes::from_slice(&env, b"mock");
        let mock_meter_client = MockMeterClient::new(&env, &meter_contract);
        mock_meter_client.set_red(&consumer, &event_id, &50);
        client.verify_participation(&consumer, &event_id, &oracle_data);
        let result = client.try_distribute_rewards(&admin, &event_id);
        assert!(result.is_err());
    }

    #[test]
    fn test_distribute_rewards_no_participation() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().set_timestamp(7200);
        let admin = Address::generate(&env);
        let meter_contract = env.register_contract(None, MockMeter);
        let client = DemandResponseRewardsClient::new(&env, &env.register_contract(None, DemandResponseRewards));
        client.initialize(&admin, &10, &meter_contract);
        let event_id = client.start_event(&admin, &100, &3600u64, &1000);
        let result = client.try_distribute_rewards(&admin, &event_id);
        assert!(result.is_err());
    }

    #[test]
    fn test_overflow_protection() {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let consumer = Address::generate(&env);
        let meter_contract = env.register_contract(None, MockMeter);
        let client = DemandResponseRewardsClient::new(&env, &env.register_contract(None, DemandResponseRewards));
        client.initialize(&admin, &(i128::MAX / 100), &meter_contract);
        client.register_consumer(&consumer);
        let event_id = client.start_event(&admin, &i128::MAX, &3600u64, &i128::MAX);
        let events: Map<u64, Event> = env.as_contract(&client.address, || {
            env.storage().instance().get(&DataKey::Events).unwrap()
        });
        let event = events.get(event_id).unwrap();
        let completion_time = event.start_time + event.duration + 1;
        env.ledger().set_timestamp(completion_time);
        let oracle_data = Bytes::from_slice(&env, b"mock");
        let mock_meter_client = MockMeterClient::new(&env, &meter_contract);
        mock_meter_client.set_red(&consumer, &event_id, &i128::MAX);
        client.verify_participation(&consumer, &event_id, &oracle_data);
        let result = client.try_distribute_rewards(&admin, &event_id);
        assert!(result.is_err());
    }

    #[test]
    fn test_multiple_consumers_single_event() {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let consumer1 = Address::generate(&env);
        let consumer2 = Address::generate(&env);
        let meter_contract = env.register_contract(None, MockMeter);
        let client = DemandResponseRewardsClient::new(&env, &env.register_contract(None, DemandResponseRewards));
        client.initialize(&admin, &10, &meter_contract);
        client.register_consumer(&consumer1);
        client.register_consumer(&consumer2);
        let event_id = client.start_event(&admin, &100, &3600u64, &1000);
        let events: Map<u64, Event> = env.as_contract(&client.address, || {
            env.storage().instance().get(&DataKey::Events).unwrap()
        });
        let event = events.get(event_id).unwrap();
        let completion_time = event.start_time + event.duration + 1;
        env.ledger().set_timestamp(completion_time);
        let mock_meter_client = MockMeterClient::new(&env, &meter_contract);
        let oracle_data1 = Bytes::from_slice(&env, b"mock_consumer1");
        mock_meter_client.set_red(&consumer1, &event_id, &30);
        client.verify_participation(&consumer1, &event_id, &oracle_data1);
        let oracle_data2 = Bytes::from_slice(&env, b"mock_consumer2");
        mock_meter_client.set_red(&consumer2, &event_id, &70);
        client.verify_participation(&consumer2, &event_id, &oracle_data2);
        let reward_rate = client.distribute_rewards(&admin, &event_id);
        assert_eq!(reward_rate, 1i128); // (1000 * 10 / 100) / 100 = 100 / 100 = 1

        let participations: Map<Address, Vec<Participation>> = env.as_contract(&client.address, || {
            env.storage().instance().get(&DataKey::Participations).unwrap()
        });
        let parts1 = participations.get(consumer1).unwrap();
        let parts2 = participations.get(consumer2).unwrap();
        assert_eq!(parts1.len(), 1);
        assert_eq!(parts2.len(), 1);
        assert_eq!(parts1.get(0).unwrap().reduction, 30i128);
        assert_eq!(parts2.get(0).unwrap().reduction, 70i128);
    }
}