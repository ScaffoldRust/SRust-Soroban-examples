use soroban_sdk::{contracttype, Env, Map, symbol_short, Address, panic_with_error};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EventState {
    Active,
    Completed,
    Cancelled,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Event {
    pub id: u64,
    pub target_reduction: i128,
    pub duration: u64,
    pub start_time: u64,
    pub status: EventState,
    pub rewards_pool: i128,
    pub reward_rate: i128,
}

pub fn start_event(
    env: &Env,
    caller: Address,
    target_reduction: i128,
    duration: u64,
    rewards_pool: i128,
) -> u64 {
    if target_reduction <= 0 || rewards_pool <= 0 || duration == 0 {
        panic_with_error!(env, crate::ContractError::InvalidInput);
    }
    let event_id = super::utils::generate_event_id(env);
    let event = Event {
        id: event_id,
        target_reduction,
        duration,
        start_time: env.ledger().timestamp(),
        status: EventState::Active,
        rewards_pool,
        reward_rate: 0i128,
    };
    let mut events: Map<u64, Event> = env.storage().instance().get(&crate::DataKey::Events).unwrap_or_else(|| Map::new(env));
    events.set(event_id, event.clone());
    env.storage().instance().set(&crate::DataKey::Events, &events);
    env.events().publish(
        (symbol_short!("evt_start"), caller),
        (event_id, target_reduction, duration)
    );
    event_id
}

pub fn complete_event(env: &Env, event_id: u64) {
    let mut events: Map<u64, Event> = env.storage().instance().get(&crate::DataKey::Events).unwrap_or_else(|| Map::new(env));
    let mut event = events.get(event_id).unwrap_or_else(|| panic_with_error!(env, crate::ContractError::EventNotFound));
    if event.status != EventState::Active {
        panic_with_error!(env, crate::ContractError::InvalidInput);
    }
    event.status = EventState::Completed;
    events.set(event_id, event.clone());
    env.storage().instance().set(&crate::DataKey::Events, &events);
    env.events().publish(
        (symbol_short!("evt_comp"),),
        event_id
    );
}

pub fn get_event(env: &Env, event_id: u64) -> Event {
    let events: Map<u64, Event> = env.storage().instance().get(&crate::DataKey::Events).unwrap_or_else(|| Map::new(env));
    events.get(event_id).unwrap_or_else(|| panic_with_error!(env, crate::ContractError::EventNotFound))
}


