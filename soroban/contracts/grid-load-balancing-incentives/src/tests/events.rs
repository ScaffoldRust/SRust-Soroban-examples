use super::utils::{setup, setup_event};
use crate::{demand, EventStatus, IncentiveError};
use soroban_sdk::testutils::{Address as _, Ledger as _};

#[test]
fn starts_event_with_expected_data() {
    let context = setup();
    let event_id = setup_event(&context);
    let event = context.client.get_event(&event_id);
    assert_eq!(event.event_id, event_id);
    assert_eq!(event.target_reduction_kw, 100);
    assert_eq!(event.reward_per_kw, 5);
    assert_eq!(event.total_participants, 0);
    assert_eq!(event.status, EventStatus::Active);
}

#[test]
fn rejects_invalid_event_parameters() {
    let context = setup();
    context.client.register_grid_operator(&context.operator);
    for params in [(0, 5, 60), (100, 0, 60), (100, 5, 0)] {
        assert_eq!(
            context
                .client
                .try_start_event(&context.operator, &params.0, &params.1, &params.2)
                .unwrap_err()
                .unwrap(),
            IncentiveError::InvalidInput
        );
    }
}

#[test]
fn rejects_missing_event() {
    let context = setup();
    assert_eq!(
        context.client.try_get_event(&999).unwrap_err().unwrap(),
        IncentiveError::EventNotFound
    );
}

#[test]
fn rejects_invalid_meter_readings() {
    let context = setup();
    let event_id = setup_event(&context);
    context.client.register_consumer(&context.consumer);

    // meter_reading_start >= meter_reading_end
    let res1 =
        context
            .client
            .try_verify_reduction(&event_id, &context.consumer, &100, &50, &500, &500);
    assert_eq!(
        res1.unwrap_err().unwrap(),
        IncentiveError::InvalidMeterReading
    );

    let res2 =
        context
            .client
            .try_verify_reduction(&event_id, &context.consumer, &100, &50, &600, &500);
    assert_eq!(
        res2.unwrap_err().unwrap(),
        IncentiveError::InvalidMeterReading
    );
}

#[test]
fn rejects_expired_event_reduction() {
    let context = setup();
    let event_id = setup_event(&context);
    context.client.register_consumer(&context.consumer);

    // Advance ledger timestamp beyond event duration (60s)
    context.env.ledger().set_timestamp(65);

    let result =
        context
            .client
            .try_verify_reduction(&event_id, &context.consumer, &100, &50, &10, &20);
    assert_eq!(result.unwrap_err().unwrap(), IncentiveError::EventExpired);
}

#[test]
fn completes_event_successfully() {
    let context = setup();
    let event_id = setup_event(&context);

    // Non-operator cannot complete event
    let other = soroban_sdk::Address::generate(&context.env);
    let unauthorized_res = context.env.as_contract(&context.contract_id, || {
        demand::complete_event(&context.env, event_id, other)
    });
    assert_eq!(unauthorized_res, Err(IncentiveError::NotAuthorized));

    // Operator completes event
    let res = context.env.as_contract(&context.contract_id, || {
        demand::complete_event(&context.env, event_id, context.operator.clone())
    });
    assert!(res.is_ok());

    let event = context.client.get_event(&event_id);
    assert_eq!(event.status, EventStatus::Completed);

    // Cannot complete already completed event
    let repeated = context.env.as_contract(&context.contract_id, || {
        demand::complete_event(&context.env, event_id, context.operator)
    });
    assert_eq!(repeated, Err(IncentiveError::EventNotActive));
}
