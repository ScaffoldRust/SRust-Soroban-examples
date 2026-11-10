use super::utils::{mint, setup, setup_event};
use crate::{incentives, IncentiveError, ParticipationStatus};
use soroban_sdk::{
    testutils::{Address as _, Ledger as _},
    token, Address,
};

#[test]
fn verifies_reduction_and_distributes_reward() {
    let context = setup();
    let event_id = setup_event(&context);
    context.client.register_consumer(&context.consumer);
    mint(&context.env, &context.token, &context.admin, 500);

    let participation_id =
        context
            .client
            .verify_reduction(&event_id, &context.consumer, &100, &60, &1, &2);

    // Fast-forward ledger timestamp past event duration
    context.env.ledger().set_timestamp(61);
    context.env.mock_all_auths_allowing_non_root_auth();

    context
        .client
        .distribute_rewards(&event_id, &context.operator);

    let participation = context.client.get_participation(&participation_id);
    assert_eq!(participation.reward_amount, 200);
    assert_eq!(participation.status, ParticipationStatus::Rewarded);

    // Verify token balance received by consumer
    let token_client = token::Client::new(&context.env, &context.token);
    assert_eq!(token_client.balance(&context.consumer), 200);
}

#[test]
fn multiple_consumers_rewarded_in_single_distribution() {
    let context = setup();
    let event_id = setup_event(&context);
    let consumer2 = Address::generate(&context.env);

    context.client.register_consumer(&context.consumer);
    context.client.register_consumer(&consumer2);
    mint(&context.env, &context.token, &context.admin, 1000);

    // Consumer 1: baseline 100, actual 70 -> 30 kW reduction * 5 reward = 150
    let p1 = context
        .client
        .verify_reduction(&event_id, &context.consumer, &100, &70, &10, &20);
    // Consumer 2: baseline 100, actual 50 -> 50 kW reduction * 5 reward = 250
    let p2 = context
        .client
        .verify_reduction(&event_id, &consumer2, &100, &50, &10, &20);

    context.env.ledger().set_timestamp(65);
    context.env.mock_all_auths_allowing_non_root_auth();

    // Admin distributes rewards
    context.client.distribute_rewards(&event_id, &context.admin);

    assert_eq!(
        context.client.get_participation(&p1).status,
        ParticipationStatus::Rewarded
    );
    assert_eq!(
        context.client.get_participation(&p2).status,
        ParticipationStatus::Rewarded
    );

    let token_client = token::Client::new(&context.env, &context.token);
    assert_eq!(token_client.balance(&context.consumer), 150);
    assert_eq!(token_client.balance(&consumer2), 250);
}

#[test]
fn rejects_distribution_while_event_active() {
    let context = setup();
    let event_id = setup_event(&context);
    context.client.register_consumer(&context.consumer);
    context
        .client
        .verify_reduction(&event_id, &context.consumer, &100, &60, &1, &2);

    // Event is still active (timestamp 0 <= 60)
    let result = context
        .client
        .try_distribute_rewards(&event_id, &context.operator);
    assert_eq!(result.unwrap_err().unwrap(), IncentiveError::EventNotActive);
}

#[test]
fn rejects_missing_reduction_and_duplicate_participation() {
    let context = setup();
    let event_id = setup_event(&context);
    context.client.register_consumer(&context.consumer);

    // Reduction is zero (baseline <= actual)
    assert_eq!(
        context
            .client
            .try_verify_reduction(&event_id, &context.consumer, &50, &50, &1, &2)
            .unwrap_err()
            .unwrap(),
        IncentiveError::InsufficientReduction
    );

    context
        .client
        .verify_reduction(&event_id, &context.consumer, &50, &40, &1, &2);

    // Duplicate verification in same event
    assert_eq!(
        context
            .client
            .try_verify_reduction(&event_id, &context.consumer, &50, &40, &2, &3)
            .unwrap_err()
            .unwrap(),
        IncentiveError::AlreadyParticipating
    );
}

#[test]
fn unauthorized_distributor_is_rejected() {
    let context = setup();
    let event_id = setup_event(&context);
    let unauthorized = Address::generate(&context.env);
    let result = context
        .client
        .try_distribute_rewards(&event_id, &unauthorized);
    assert_eq!(result.unwrap_err().unwrap(), IncentiveError::NotAuthorized);
}

#[test]
fn audits_and_rejects_participation_records() {
    let context = setup();
    let event_id = setup_event(&context);
    context.client.register_consumer(&context.consumer);

    let p_id = context
        .client
        .verify_reduction(&event_id, &context.consumer, &100, &80, &1, &2);

    // Audit participations for event
    let records = context.env.as_contract(&context.contract_id, || {
        incentives::audit_event_rewards(&context.env, event_id).unwrap()
    });
    assert_eq!(records.len(), 1);
    assert_eq!(records.get(0).unwrap().participation_id, p_id);

    // Unauthorized rejection
    let unauthorized = Address::generate(&context.env);
    let rej_err = context.env.as_contract(&context.contract_id, || {
        incentives::reject_participation(&context.env, p_id, unauthorized)
    });
    assert_eq!(rej_err, Err(IncentiveError::NotAuthorized));

    // Admin rejects participation
    let rej_ok = context.env.as_contract(&context.contract_id, || {
        incentives::reject_participation(&context.env, p_id, context.admin.clone())
    });
    assert!(rej_ok.is_ok());

    let participation = context.client.get_participation(&p_id);
    assert_eq!(participation.status, ParticipationStatus::Rejected);
}
