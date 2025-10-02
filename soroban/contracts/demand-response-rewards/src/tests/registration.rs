#![cfg(test)]

use super::utils::*;

// ============ CONSUMER REGISTRATION TESTS ============

#[test]
fn test_consumer_registration() {
    let ctx = setup_initialized();
    let consumer = create_consumer(&ctx.env);

    register_consumer(&ctx, &consumer);

    assert_consumer_registered(&ctx, &consumer);
}

#[test]
#[should_panic(expected = "consumer already registered")]
fn test_duplicate_registration() {
    let ctx = setup_initialized();
    let consumer = create_consumer(&ctx.env);

    register_consumer(&ctx, &consumer);
    register_consumer(&ctx, &consumer); // Key Scenario: Duplicate registration
}

#[test]
#[should_panic(expected = "Error(Contract, #9)")] // NotInitialized
fn test_registration_with_invalid_data() {
    let ctx = setup_test(); // Key Scenario: Invalid state (not initialized)
    let consumer = create_consumer(&ctx.env);

    register_consumer(&ctx, &consumer);
}

#[test]
fn test_high_volume_registration() {
    let ctx = setup_initialized();
    let num_consumers = 50u32; // Note: Scalability requirement
    let consumers = create_multiple_consumers(&ctx.env, num_consumers);

    for i in 0..consumers.len() {
        let consumer = consumers.get(i).unwrap();
        register_consumer(&ctx, &consumer);
    }

    let stored_consumers = get_consumers(&ctx);
    assert_eq!(stored_consumers.len(), num_consumers);
}
