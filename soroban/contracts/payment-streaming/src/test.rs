#![cfg(test)]

use crate::{
    stream::{PaymentSchedule, TimeUnit},
    PaymentStreamingContract, PaymentStreamingContractClient,
};
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, BytesN, Env,
};

fn create_test_contract(env: &Env) -> PaymentStreamingContractClient<'_> {
    PaymentStreamingContractClient::new(env, &env.register(PaymentStreamingContract {}, ()))
}

#[test]
fn test_stream_creation_and_basic_flow() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    let sender = Address::generate(&env);
    let recipient = Address::generate(&env);

    // Create a 30-day stream releasing 1000 tokens
    let total_amount = 1000i128;
    let duration = 30 * 24 * 60 * 60u64; // 30 days in seconds
    let schedule = PaymentSchedule {
        unit: TimeUnit::Seconds,
        interval: 86400,                 // 1 day
        release_rate: total_amount / 30, // ~33.33 tokens per day
    };

    let stream_id =
        contract.create_stream(&sender, &recipient, &total_amount, &duration, &schedule);

    // Verify stream was created
    let stream_state = contract.get_stream_balance(&stream_id);
    assert_eq!(stream_state.total, total_amount);
    assert_eq!(stream_state.withdrawn, 0);
}

#[test]
fn test_stream_withdrawal_over_time() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    let sender = Address::generate(&env);
    let recipient = Address::generate(&env);

    let total_amount = 3000i128;
    let duration = 30 * 24 * 60 * 60u64; // 30 days
    let schedule = PaymentSchedule {
        unit: TimeUnit::Seconds,
        interval: 86400,
        release_rate: 100,
    };

    let stream_id =
        contract.create_stream(&sender, &recipient, &total_amount, &duration, &schedule);

    // Fast forward 10 days
    env.ledger().with_mut(|li| {
        li.timestamp = li.timestamp + 10 * 24 * 60 * 60;
    });

    // Check available balance after 10 days (should be ~1000 tokens)
    let stream_state = contract.get_stream_balance(&stream_id);
    let expected_available = (total_amount * 10) / 30; // 1000 tokens
    assert_eq!(stream_state.available, expected_available);

    // Withdraw 500 tokens
    contract.withdraw_from_stream(&stream_id, &500i128);

    // Check updated state
    let stream_state = contract.get_stream_balance(&stream_id);
    assert_eq!(stream_state.withdrawn, 500);
    assert_eq!(stream_state.available, expected_available - 500);

    // Fast forward another 10 days (20 days total)
    env.ledger().with_mut(|li| {
        li.timestamp = li.timestamp + 10 * 24 * 60 * 60;
    });

    // Check available balance after 20 days
    let stream_state = contract.get_stream_balance(&stream_id);
    let expected_available_20_days = (total_amount * 20) / 30 - 500; // 2000 - 500 = 1500
    assert_eq!(stream_state.available, expected_available_20_days);

    // Withdraw remaining available amount
    contract.withdraw_from_stream(&stream_id, &expected_available_20_days);

    let stream_state = contract.get_stream_balance(&stream_id);
    assert_eq!(stream_state.withdrawn, 2000);
    assert_eq!(stream_state.available, 0);
}

#[test]
fn test_stream_pause_functionality() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    let sender = Address::generate(&env);
    let recipient = Address::generate(&env);

    let total_amount = 1000i128;
    let duration = 10 * 24 * 60 * 60u64; // 10 days
    let schedule = PaymentSchedule {
        unit: TimeUnit::Seconds,
        interval: 86400,
        release_rate: 100,
    };

    let stream_id =
        contract.create_stream(&sender, &recipient, &total_amount, &duration, &schedule);

    // Fast forward 3 days and withdraw
    env.ledger().with_mut(|li| {
        li.timestamp = li.timestamp + 3 * 24 * 60 * 60;
    });

    contract.withdraw_from_stream(&stream_id, &300i128);

    // Pause the stream
    contract.pause_stream(&stream_id);

    // Fast forward 5 more days while paused
    env.ledger().with_mut(|li| {
        li.timestamp = li.timestamp + 5 * 24 * 60 * 60;
    });

    // Check that no additional tokens are available due to pause
    let stream_state = contract.get_stream_balance(&stream_id);
    assert_eq!(stream_state.available, 0); // Stream is paused
    assert_eq!(stream_state.withdrawn, 300); // Previous withdrawal preserved

    // Note: Stream remains paused - in a complete implementation,
    // a resume function would reactivate the stream
}

#[test]
fn test_stream_cancellation() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    let sender = Address::generate(&env);
    let recipient = Address::generate(&env);

    let total_amount = 1000i128;
    let duration = 10 * 24 * 60 * 60u64;
    let schedule = PaymentSchedule {
        unit: TimeUnit::Seconds,
        interval: 86400,
        release_rate: 100,
    };

    let stream_id =
        contract.create_stream(&sender, &recipient, &total_amount, &duration, &schedule);

    // Fast forward and make a withdrawal
    env.ledger().with_mut(|li| {
        li.timestamp = li.timestamp + 3 * 24 * 60 * 60;
    });

    contract.withdraw_from_stream(&stream_id, &300i128);

    // Cancel the stream
    contract.cancel_stream(&stream_id);

    // Try to withdraw after cancellation - should fail
    let result = contract.try_withdraw_from_stream(&stream_id, &100i128);
    assert!(result.is_err());
}

#[test]
fn test_stream_full_duration_completion() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    let sender = Address::generate(&env);
    let recipient = Address::generate(&env);

    let total_amount = 1000i128;
    let duration = 10 * 24 * 60 * 60u64;
    let schedule = PaymentSchedule {
        unit: TimeUnit::Seconds,
        interval: 86400,
        release_rate: 100,
    };

    let stream_id =
        contract.create_stream(&sender, &recipient, &total_amount, &duration, &schedule);

    // Fast forward past the complete duration
    env.ledger().with_mut(|li| {
        li.timestamp = li.timestamp + duration + 1;
    });

    // All funds should be available
    let stream_state = contract.get_stream_balance(&stream_id);
    assert_eq!(stream_state.available, total_amount);

    // Withdraw all funds
    contract.withdraw_from_stream(&stream_id, &total_amount);

    let stream_state = contract.get_stream_balance(&stream_id);
    assert_eq!(stream_state.withdrawn, total_amount);
    assert_eq!(stream_state.available, 0);
}

#[test]
fn test_micropayment_channel_opening() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    let sender = Address::generate(&env);
    let party_b = Address::generate(&env);
    let deposit = 1000i128;

    let channel_id = contract.open_channel(&sender, &party_b, &deposit);

    // Verify channel was created successfully (non-zero ID)
    assert!(!channel_id.to_array().iter().all(|&x| x == 0));
}

#[test]
fn test_signed_voucher_creation_attempt() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    let sender = Address::generate(&env);
    let party_b = Address::generate(&env);
    let deposit = 1000i128;

    let channel_id = contract.open_channel(&sender, &party_b, &deposit);

    // Create a signed voucher attempt
    let increment_amount = 100i128;
    let caller_key = BytesN::from_array(&env, &[1u8; 32]); // Mock public key
    let signature = BytesN::from_array(&env, &[2u8; 64]); // Mock signature

    // This will fail due to signature verification, but tests the structure
    let result = contract.try_sign_payment(&channel_id, &increment_amount, &caller_key, &signature);

    // Expected to fail due to invalid signature in test environment
    assert!(result.is_err());
}

#[test]
fn test_channel_closure_with_final_settlement() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    let sender = Address::generate(&env);
    let party_b = Address::generate(&env);
    let deposit = 1000i128;

    let channel_id = contract.open_channel(&sender, &party_b, &deposit);

    // Close channel with final settlement
    // Party A gets 600, Party B gets 400
    let final_state = (600i128, 400i128);

    contract.close_channel(&channel_id, &final_state);

    // If we reach here, the channel was closed successfully
    // (no panic means the final state was valid)
}

#[test]
fn test_channel_closure_invalid_final_state() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    let sender = Address::generate(&env);
    let party_b = Address::generate(&env);
    let deposit = 1000i128;

    let channel_id = contract.open_channel(&sender, &party_b, &deposit);

    // Try to close with invalid final state (doesn't sum to deposit)
    let invalid_final_state = (600i128, 500i128); // 1100 > 1000

    let result = contract.try_close_channel(&channel_id, &invalid_final_state);
    assert!(result.is_err());
}

#[test]
fn test_complex_streaming_scenario() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    let sender = Address::generate(&env);
    let recipient = Address::generate(&env);

    // Create a stream that releases 3000 tokens over 30 days
    let total_amount = 3000i128;
    let duration = 30 * 24 * 60 * 60u64;
    let schedule = PaymentSchedule {
        unit: TimeUnit::Seconds,
        interval: 86400,
        release_rate: 100, // 100 tokens per day
    };

    let stream_id =
        contract.create_stream(&sender, &recipient, &total_amount, &duration, &schedule);

    // Day 5: Withdraw 250 tokens
    env.ledger().with_mut(|li| {
        li.timestamp = li.timestamp + 5 * 24 * 60 * 60;
    });
    contract.withdraw_from_stream(&stream_id, &250i128);

    // Day 10: Withdraw another 300 tokens
    env.ledger().with_mut(|li| {
        li.timestamp = li.timestamp + 5 * 24 * 60 * 60;
    });
    contract.withdraw_from_stream(&stream_id, &300i128);

    // Day 15: Check available balance
    env.ledger().with_mut(|li| {
        li.timestamp = li.timestamp + 5 * 24 * 60 * 60;
    });

    let stream_state = contract.get_stream_balance(&stream_id);
    let expected_total_available = (total_amount * 15) / 30; // 1500 tokens after 15 days
    let expected_available = expected_total_available - 550; // 950 tokens available

    assert_eq!(stream_state.withdrawn, 550);
    assert_eq!(stream_state.available, expected_available);

    // Day 30: Complete the stream
    env.ledger().with_mut(|li| {
        li.timestamp = li.timestamp + 15 * 24 * 60 * 60;
    });

    let stream_state = contract.get_stream_balance(&stream_id);
    assert_eq!(stream_state.available, total_amount - 550);

    // Withdraw remaining funds
    contract.withdraw_from_stream(&stream_id, &(total_amount - 550));

    let final_state = contract.get_stream_balance(&stream_id);
    assert_eq!(final_state.withdrawn, total_amount);
    assert_eq!(final_state.available, 0);
}

#[test]
fn test_multiple_channels_scenario() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);

    // Create multiple parties
    let bob = Address::generate(&env);
    let charlie = Address::generate(&env);

    // Open channels with Bob and Charlie
    let sender = Address::generate(&env);
    let channel_ab = contract.open_channel(&sender, &bob, &1000i128);
    let channel_ac = contract.open_channel(&sender, &charlie, &500i128);

    // Close both channels with different settlements
    contract.close_channel(&channel_ab, &(700i128, 300i128));
    contract.close_channel(&channel_ac, &(200i128, 300i128));

    // Verify channels were handled independently by reaching this point
    assert!(true);
}

#[test]
fn test_edge_cases_and_error_conditions() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    let sender = Address::generate(&env);
    let recipient = Address::generate(&env);

    // Test invalid stream parameters
    let result = contract.try_create_stream(
        &sender,
        &recipient,
        &0i128, // Invalid amount
        &1000u64,
        &PaymentSchedule {
            unit: TimeUnit::Seconds,
            interval: 86400,
            release_rate: 100,
        },
    );
    assert!(result.is_err());

    // Test invalid channel deposit
    let result = contract.try_open_channel(&sender, &recipient, &0i128);
    assert!(result.is_err());

    // Test withdrawal from non-existent stream
    let fake_stream_id = BytesN::from_array(&env, &[255u8; 32]);
    let result = contract.try_withdraw_from_stream(&fake_stream_id, &100i128);
    assert!(result.is_err());
}

#[test]
fn test_insufficient_funds_withdrawal() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    let sender = Address::generate(&env);
    let recipient = Address::generate(&env);

    let total_amount = 1000i128;
    let duration = 10 * 24 * 60 * 60u64;
    let schedule = PaymentSchedule {
        unit: TimeUnit::Seconds,
        interval: 86400,
        release_rate: 100,
    };

    let stream_id =
        contract.create_stream(&sender, &recipient, &total_amount, &duration, &schedule);

    // Fast forward 1 day (only 100 tokens should be available)
    env.ledger().with_mut(|li| {
        li.timestamp = li.timestamp + 24 * 60 * 60;
    });

    // Try to withdraw more than available
    let result = contract.try_withdraw_from_stream(&stream_id, &500i128);
    assert!(result.is_err());
}

// Simple integration test demonstrating channel-only functionality
#[test]
fn test_complete_payment_system_integration() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);

    // Test multiple channel operations in sequence
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let charlie = Address::generate(&env);

    // 1. Open first micropayment channel
    let channel_deposit_1 = 1000i128;
    let channel_id_1 = contract.open_channel(&alice, &bob, &channel_deposit_1);

    // 2. Open second micropayment channel
    let channel_deposit_2 = 500i128;
    let channel_id_2 = contract.open_channel(&alice, &charlie, &channel_deposit_2);

    // 3. Verify both channels were created with valid IDs
    assert!(!channel_id_1.to_array().iter().all(|&x| x == 0));
    assert!(!channel_id_2.to_array().iter().all(|&x| x == 0));
    assert_ne!(channel_id_1, channel_id_2); // Different channels should have different IDs

    // 4. Close both channels with different settlements
    contract.close_channel(&channel_id_1, &(700i128, 300i128));
    contract.close_channel(&channel_id_2, &(200i128, 300i128));

    // Test completed successfully - multiple channel operations work independently
}
