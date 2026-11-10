use super::utils::setup;
use crate::{GridLoadBalancingIncentives, GridLoadBalancingIncentivesClient, IncentiveError};
use soroban_sdk::{testutils::Address as _, Address, Env};

#[test]
fn registers_consumer_and_preserves_registration() {
    let context = setup();
    context.client.register_consumer(&context.consumer);
    // Duplicate registration is idempotent
    context.client.register_consumer(&context.consumer);
    assert_eq!(
        context
            .client
            .get_consumer_participations(&context.consumer)
            .len(),
        0
    );
}

#[test]
fn registers_multiple_grid_operators() {
    let context = setup();
    let op2 = Address::generate(&context.env);
    context.client.register_grid_operator(&context.operator);
    context.client.register_grid_operator(&op2);

    let event_id1 = context.client.start_event(&context.operator, &100, &5, &60);
    let event_id2 = context.client.start_event(&op2, &200, &10, &120);

    assert_eq!(
        context.client.get_event(&event_id1).grid_operator,
        context.operator
    );
    assert_eq!(context.client.get_event(&event_id2).grid_operator, op2);
}

#[test]
fn duplicate_initialization_rejected() {
    let context = setup();
    let result = context
        .client
        .try_initialize(&context.admin, &context.token);
    assert_eq!(
        result.unwrap_err().unwrap(),
        IncentiveError::AlreadyInitialized
    );
}

#[test]
fn uninitialized_calls_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let contract = env.register(GridLoadBalancingIncentives, ());
    let client = GridLoadBalancingIncentivesClient::new(&env, &contract);
    let dummy = Address::generate(&env);

    assert_eq!(
        client.try_register_consumer(&dummy).unwrap_err().unwrap(),
        IncentiveError::NotInitialized
    );
    assert_eq!(
        client
            .try_register_grid_operator(&dummy)
            .unwrap_err()
            .unwrap(),
        IncentiveError::NotInitialized
    );
}

#[test]
fn unregistered_consumer_cannot_verify_reduction() {
    let context = setup();
    let event_id = {
        context.client.register_grid_operator(&context.operator);
        context.client.start_event(&context.operator, &100, &5, &60)
    };
    let result =
        context
            .client
            .try_verify_reduction(&event_id, &context.consumer, &50, &100, &10, &20);
    assert_eq!(
        result.unwrap_err().unwrap(),
        IncentiveError::ConsumerNotRegistered
    );
}

#[test]
fn unregistered_operator_cannot_start_event() {
    let context = setup();
    let operator = Address::generate(&context.env);
    let result = context.client.try_start_event(&operator, &100, &5, &60);
    assert_eq!(
        result.unwrap_err().unwrap(),
        IncentiveError::GridOperatorNotRegistered
    );
}
