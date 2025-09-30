#![cfg(test)]

use super::utils::*;

// ============ EVENT INITIATION TESTS ============

#[test]
fn test_event_initiation() {
    let ctx = setup_initialized();

    let event_id = create_basic_event(&ctx);

    assert_event_exists(&ctx, event_id);
    let event = get_event(&ctx, event_id);
    assert_eq!(event.target_reduction, 100i128);
    assert_eq!(event.rewards_pool, 1000i128);
}

#[test]
#[should_panic(expected = "not authorized as operator")]
fn test_unauthorized_event_initiation() {
    let ctx = setup_initialized();
    let unauthorized = create_consumer(&ctx.env); // Key Scenario: Unauthorized attempt

    create_event_with_params(&ctx, &unauthorized, 100i128, 3600u64, 1000i128);
}

#[test]
#[should_panic(expected = "invalid parameters")]
fn test_event_with_invalid_reduction_target() {
    let ctx = setup_initialized();
    // Key Scenario: Invalid reduction target
    create_event_with_params(&ctx, &ctx.admin, -100i128, 3600u64, 1000i128);
}

#[test]
#[should_panic(expected = "invalid parameters")]
fn test_event_with_incomplete_data() {
    let ctx = setup_initialized();
    // Key Scenario: Incomplete/invalid event data (zero duration)
    create_event_with_params(&ctx, &ctx.admin, 100i128, 0u64, 1000i128);
}

#[test]
fn test_consumer_participation_verification() {
    let ctx = setup_initialized();
    let consumer = create_consumer(&ctx.env);

    register_consumer(&ctx, &consumer);
    let event_id = create_basic_event(&ctx);

    // Note: Integration with smart-meter-data-ledger (mock)
    setup_and_verify_participation(&ctx, &consumer, event_id, 50);

    assert_participation_verified(&ctx, &consumer, event_id, 50);
}

#[test]
fn test_high_volume_participation() {
    let ctx = setup_initialized();
    let num_consumers = 30u32; // Note: Scalability requirement
    let consumers = create_multiple_consumers(&ctx.env, num_consumers);

    for i in 0..consumers.len() {
        let consumer = consumers.get(i).unwrap();
        register_consumer(&ctx, &consumer);
    }

    let event_id = create_basic_event(&ctx);

    for i in 0..consumers.len() {
        let consumer = consumers.get(i).unwrap();
        setup_and_verify_participation(&ctx, &consumer, event_id, 10);
    }
}
