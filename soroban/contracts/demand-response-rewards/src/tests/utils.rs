#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Bytes, Env, Map, Vec,
};

use crate::{
    events::{Event, EventState},
    DemandResponseRewards, DemandResponseRewardsClient, Participation,
};

// ============ MOCK METER CONTRACT ============

use soroban_sdk::{contract, contractimpl};

#[contract]
pub struct MockMeter;

#[contractimpl]
impl MockMeter {
    pub fn set_red(env: Env, consumer: Address, event_id: u64, red: i128) {
        env.storage().instance().set(&(consumer, event_id), &red);
    }

    pub fn get_red(env: Env, consumer: Address, event_id: u64) -> i128 {
        env.storage()
            .instance()
            .get(&(consumer, event_id))
            .unwrap_or(0)
    }
}

// ============ TEST CONTEXT ============

pub struct TestContext {
    pub env: Env,
    pub client: DemandResponseRewardsClient<'static>,
    pub admin: Address,
    pub meter_contract: Address,
    pub mock_meter_client: crate::tests::utils::MockMeterClient<'static>,
}

// ============ SETUP FUNCTIONS ============

pub fn setup_test() -> TestContext {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let meter_contract = env.register_contract(None, MockMeter);
    let client =
        DemandResponseRewardsClient::new(&env, &env.register_contract(None, DemandResponseRewards));
    let mock_meter_client = MockMeterClient::new(&env, &meter_contract);

    TestContext {
        env,
        client,
        admin,
        meter_contract,
        mock_meter_client,
    }
}

pub fn setup_initialized() -> TestContext {
    let ctx = setup_test();
    ctx.client
        .initialize(&ctx.admin, &10i128, &ctx.meter_contract);
    ctx
}

// ============ USER CREATION ============

pub fn create_consumer(env: &Env) -> Address {
    Address::generate(env)
}

pub fn create_multiple_consumers(env: &Env, count: u32) -> Vec<Address> {
    let mut consumers = Vec::new(env);
    for _ in 0..count {
        consumers.push_back(Address::generate(env));
    }
    consumers
}

// ============ REGISTRATION ============

pub fn register_consumer(ctx: &TestContext, consumer: &Address) {
    ctx.client.register_consumer(consumer);
}

// ============ EVENT CREATION ============

pub fn create_basic_event(ctx: &TestContext) -> u64 {
    ctx.client
        .start_event(&ctx.admin, &100i128, &3600u64, &1000i128)
}

pub fn create_event_with_params(
    ctx: &TestContext,
    caller: &Address,
    target_reduction: i128,
    duration: u64,
    rewards_pool: i128,
) -> u64 {
    ctx.client
        .start_event(caller, &target_reduction, &duration, &rewards_pool)
}

// ============ PARTICIPATION ============

pub fn setup_meter_reduction(
    ctx: &TestContext,
    consumer: &Address,
    event_id: u64,
    reduction: i128,
) {
    ctx.mock_meter_client
        .set_red(consumer, &event_id, &reduction);
}

pub fn verify_participation(
    ctx: &TestContext,
    consumer: &Address,
    event_id: u64,
    oracle_data: &Bytes,
) {
    ctx.client
        .verify_participation(consumer, &event_id, oracle_data);
}

pub fn setup_and_verify_participation(
    ctx: &TestContext,
    consumer: &Address,
    event_id: u64,
    reduction: i128,
) {
    setup_meter_reduction(ctx, consumer, event_id, reduction);
    let oracle_data = create_mock_oracle_data(&ctx.env, event_id, consumer, reduction);
    verify_participation(ctx, consumer, event_id, &oracle_data);
}

pub fn create_mock_oracle_data(
    env: &Env,
    event_id: u64,
    _consumer: &Address,
    _reduction: i128,
) -> Bytes {
    let mut bytes = Bytes::new(env);
    bytes.append(&Bytes::from_slice(env, b"oracle_evt_"));
    bytes.append(&Bytes::from_slice(env, &event_id.to_le_bytes()));
    bytes
}

// ============ TIME MANIPULATION ============

pub fn advance_time_past_event(ctx: &TestContext, event_id: u64) {
    let events: Map<u64, Event> = ctx.env.as_contract(&ctx.client.address, || {
        ctx.env
            .storage()
            .instance()
            .get(&crate::DataKey::Events)
            .unwrap()
    });
    let event = events.get(event_id).unwrap();
    let completion_time = event.start_time + event.duration + 1;
    ctx.env.ledger().set_timestamp(completion_time);
}

// ============ REWARDS ============

pub fn distribute_rewards(ctx: &TestContext, event_id: u64) -> i128 {
    ctx.client.distribute_rewards(&ctx.admin, &event_id)
}

pub fn claim_reward(ctx: &TestContext, consumer: &Address, event_id: u64) -> i128 {
    ctx.client.claim_reward(consumer, &event_id)
}

// ============ STORAGE ACCESS ============

pub fn get_consumers(ctx: &TestContext) -> Map<Address, bool> {
    ctx.env.as_contract(&ctx.client.address, || {
        ctx.env
            .storage()
            .instance()
            .get(&crate::DataKey::Consumers)
            .unwrap_or_else(|| Map::new(&ctx.env))
    })
}

pub fn get_events(ctx: &TestContext) -> Map<u64, Event> {
    ctx.env.as_contract(&ctx.client.address, || {
        ctx.env
            .storage()
            .instance()
            .get(&crate::DataKey::Events)
            .unwrap_or_else(|| Map::new(&ctx.env))
    })
}

pub fn get_event(ctx: &TestContext, event_id: u64) -> Event {
    let events = get_events(ctx);
    events.get(event_id).unwrap()
}

pub fn get_consumer_participations(ctx: &TestContext, consumer: &Address) -> Vec<Participation> {
    let participations: Map<Address, Vec<Participation>> =
        ctx.env.as_contract(&ctx.client.address, || {
            ctx.env
                .storage()
                .instance()
                .get(&crate::DataKey::Participations)
                .unwrap_or_else(|| Map::new(&ctx.env))
        });
    participations
        .get(consumer.clone())
        .unwrap_or_else(|| Vec::new(&ctx.env))
}

// ============ ASSERTIONS ============

pub fn assert_consumer_registered(ctx: &TestContext, consumer: &Address) {
    let consumers = get_consumers(ctx);
    assert!(
        consumers.get(consumer.clone()).unwrap_or(false),
        "Consumer should be registered"
    );
}

pub fn assert_event_exists(ctx: &TestContext, event_id: u64) {
    let events = get_events(ctx);
    assert!(events.contains_key(event_id), "Event should exist");
}

pub fn assert_event_status(ctx: &TestContext, event_id: u64, expected_status: EventState) {
    let event = get_event(ctx, event_id);
    assert_eq!(event.status, expected_status, "Event status should match");
}

pub fn assert_participation_verified(
    ctx: &TestContext,
    consumer: &Address,
    event_id: u64,
    expected_reduction: i128,
) {
    let participations = get_consumer_participations(ctx, consumer);
    let participation = participations
        .iter()
        .find(|p| p.event_id == event_id)
        .expect("Participation should exist");

    assert_eq!(
        participation.reduction, expected_reduction,
        "Reduction should match"
    );
    assert_eq!(
        participation.status,
        soroban_sdk::symbol_short!("verif"),
        "Should be verified"
    );
}

pub fn assert_participation_claimed(ctx: &TestContext, consumer: &Address, event_id: u64) {
    let participations = get_consumer_participations(ctx, consumer);
    let participation = participations
        .iter()
        .find(|p| p.event_id == event_id)
        .expect("Participation should exist");

    assert!(participation.claimed, "Participation should be claimed");
}

// ============ WORKFLOW HELPERS ============

pub fn setup_complete_event_flow(ctx: &TestContext, consumer: &Address, reduction: i128) -> u64 {
    register_consumer(ctx, consumer);
    let event_id = create_basic_event(ctx);
    setup_and_verify_participation(ctx, consumer, event_id, reduction);
    event_id
}

pub fn setup_and_complete_event(
    ctx: &TestContext,
    consumer: &Address,
    reduction: i128,
) -> (u64, i128) {
    let event_id = setup_complete_event_flow(ctx, consumer, reduction);
    advance_time_past_event(ctx, event_id);
    let reward_rate = distribute_rewards(ctx, event_id);
    (event_id, reward_rate)
}

pub fn setup_multi_consumer_event(
    ctx: &TestContext,
    consumers: &[Address],
    reductions: &[i128],
) -> u64 {
    assert_eq!(
        consumers.len(),
        reductions.len(),
        "Consumers and reductions must match"
    );

    for consumer in consumers {
        register_consumer(ctx, consumer);
    }

    let event_id = create_basic_event(ctx);

    for (consumer, &reduction) in consumers.iter().zip(reductions.iter()) {
        setup_and_verify_participation(ctx, consumer, event_id, reduction);
    }

    event_id
}
