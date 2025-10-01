#![cfg(test)]

use super::utils::*;
use crate::events::EventState;

// ============ REWARD DISTRIBUTION TESTS ============

#[test]
fn test_reward_distribution() {
    let ctx = setup_initialized();
    let consumer = create_consumer(&ctx.env);

    let (event_id, reward_rate) = setup_and_complete_event(&ctx, &consumer, 50);

    // Verify reward accuracy: (1000 * 10 / 100) / 50 = 2
    assert_eq!(reward_rate, 2i128);
    assert_event_status(&ctx, event_id, EventState::Completed);
}

#[test]
fn test_reward_accuracy_multiple_consumers() {
    let ctx = setup_initialized();
    let consumer1 = create_consumer(&ctx.env);
    let consumer2 = create_consumer(&ctx.env);

    // Note: Verify reward accuracy and fairness
    let event_id =
        setup_multi_consumer_event(&ctx, &[consumer1.clone(), consumer2.clone()], &[30, 70]);

    advance_time_past_event(&ctx, event_id);
    distribute_rewards(&ctx, event_id);

    let reward1 = claim_reward(&ctx, &consumer1, event_id);
    let reward2 = claim_reward(&ctx, &consumer2, event_id);

    // Proportional rewards: 30 and 70 (rate = 1)
    assert_eq!(reward1, 30i128);
    assert_eq!(reward2, 70i128);
}

#[test]
fn test_reward_distribution_missing_reduction_data() {
    let ctx = setup_initialized();

    let event_id = create_basic_event(&ctx);
    advance_time_past_event(&ctx, event_id);

    // Key Scenario: Missing reduction data (no participation)
    let result = ctx.client.try_distribute_rewards(&ctx.admin, &event_id);
    assert!(result.is_err());
}

#[test]
#[should_panic(expected = "Error(Contract, #5)")] // NotRegistered
fn test_unauthorized_reward_claim() {
    let ctx = setup_initialized();
    let registered = create_consumer(&ctx.env);
    let unregistered = create_consumer(&ctx.env); // Key Scenario: Unauthorized claim

    let (event_id, _) = setup_and_complete_event(&ctx, &registered, 50);

    claim_reward(&ctx, &unregistered, event_id);
}

#[test]
#[should_panic(expected = "Error(Contract, #6)")] // ParticipationNotVerified
fn test_double_claim_prevention() {
    let ctx = setup_initialized();
    let consumer = create_consumer(&ctx.env);

    let (event_id, _) = setup_and_complete_event(&ctx, &consumer, 50);

    claim_reward(&ctx, &consumer, event_id);
    claim_reward(&ctx, &consumer, event_id); // Should panic
}

#[test]
fn test_high_volume_reward_claiming() {
    let ctx = setup_initialized();
    let num_consumers = 25u32; // Note: Scalability requirement
    let consumers = create_multiple_consumers(&ctx.env, num_consumers);

    for i in 0..consumers.len() {
        let consumer = consumers.get(i).unwrap();
        register_consumer(&ctx, &consumer);
    }

    let event_id = create_event_with_params(&ctx, &ctx.admin, 10000, 3600, 50000);

    for i in 0..consumers.len() {
        let consumer = consumers.get(i).unwrap();
        setup_and_verify_participation(&ctx, &consumer, event_id, 100);
    }

    advance_time_past_event(&ctx, event_id);
    let reward_rate = distribute_rewards(&ctx, event_id);

    // All consumers claim - verify auditable outcomes
    for i in 0..consumers.len() {
        let consumer = consumers.get(i).unwrap();
        let reward = claim_reward(&ctx, &consumer, event_id);
        assert_eq!(reward, 100 * reward_rate);
    }
}

#[test]
fn test_full_lifecycle_integration() {
    let ctx = setup_initialized();
    let consumer = create_consumer(&ctx.env);

    // Note: End-to-end integration test for auditable outcomes

    // 1. Register
    register_consumer(&ctx, &consumer);
    assert_consumer_registered(&ctx, &consumer);

    // 2. Create event
    let event_id = create_basic_event(&ctx);

    // 3. Participate (meter integration)
    setup_and_verify_participation(&ctx, &consumer, event_id, 50);

    // 4. Distribute rewards
    advance_time_past_event(&ctx, event_id);
    let reward_rate = distribute_rewards(&ctx, event_id);
    assert_eq!(reward_rate, 2);

    // 5. Claim reward
    let claimed = claim_reward(&ctx, &consumer, event_id);
    assert_eq!(claimed, 100);
    assert_participation_claimed(&ctx, &consumer, event_id);
}
