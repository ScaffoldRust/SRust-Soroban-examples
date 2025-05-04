#![cfg(test)]
use crate::{
    channel::{open_channel, close_channel, PaymentChannel},
    withdraw::{withdraw_from_stream, calculate_available},
    error::PaymentStreamingError,
    stream::{create_stream, pause_stream, resume_stream, cancel_stream, Stream}
};
use soroban_sdk::{
    testutils::{Address as _, BytesN as _, Ledger},
    Address, BytesN, Env, IntoVal, Symbol, Vec, contractimpl, contract
};

// Helper function to create a test environment with advanced time capabilities
fn create_test_env() -> Env {
    let env = Env::default();
    env
}

// Helper function to advance ledger time
fn advance_ledger_time(env: &Env, seconds: u64) {
    let timestamp = env.ledger().timestamp();
    env.ledger().set(Ledger {
        timestamp: timestamp + seconds,
        ..env.ledger()
    });
}

#[test]
fn test_stream_creation_and_flow() {
    // Setup test environment
    let env = create_test_env();
    
    // Create test addresses
    let sender = Address::random(&env);
    let recipient = Address::random(&env);
    
    // Create a stream that releases tokens over 30 days (2,592,000 seconds)
    let total_amount = 30_000_000; // 30 million tokens
    let duration = 2_592_000u64; // 30 days in seconds
    let start_time = env.ledger().timestamp();
    
    // Create the stream
    let stream_id = create_stream(&env, sender.clone(), recipient.clone(), total_amount, duration);
    
    // Verify stream was created correctly
    let stream: Stream = env.storage().persistent().get(&stream_id).unwrap();
    assert_eq!(stream.sender, sender);
    assert_eq!(stream.recipient, recipient);
    assert_eq!(stream.total_amount, total_amount);
    assert_eq!(stream.duration, duration);
    assert_eq!(stream.start_time, start_time);
    assert_eq!(stream.withdrawn, 0);
    assert!(stream.is_active);
    
    // Advance time by 10 days
    advance_ledger_time(&env, 864_000); // 10 days in seconds
    
    // Calculate expected available amount (10 days = 1/3 of total)
    let expected_available = total_amount / 3;
    
    // Authorize as recipient for withdrawal
    env.mock_all_auths();
    
    // Withdraw half of the available amount
    let withdrawal_amount = expected_available / 2;
    withdraw_from_stream(&env, stream_id.clone(), withdrawal_amount);
    
    // Verify withdrawal was successful
    let stream_after_withdrawal: Stream = env.storage().persistent().get(&stream_id).unwrap();
    assert_eq!(stream_after_withdrawal.withdrawn, withdrawal_amount);
    
    // Advance time by another 10 days
    advance_ledger_time(&env, 864_000); // Another 10 days
    
    // Withdraw remaining available amount (should be approximately 2/3 of total minus previous withdrawal)
    let elapsed = env.ledger().timestamp() - start_time;
    let available = calculate_available(&stream_after_withdrawal, elapsed);
    withdraw_from_stream(&env, stream_id.clone(), available);
    
    // Verify final state
    let final_stream: Stream = env.storage().persistent().get(&stream_id).unwrap();
    assert_eq!(final_stream.withdrawn, withdrawal_amount + available);
}

#[test]
fn test_stream_pause_resume_cancel() {
    // Setup test environment
    let env = create_test_env();
    
    // Create test addresses
    let sender = Address::random(&env);
    let recipient = Address::random(&env);
    
    // Create a stream
    let total_amount = 10_000_000;
    let duration = 2_592_000u64; // 30 days
    let stream_id = create_stream(&env, sender.clone(), recipient.clone(), total_amount, duration);
    
    // Advance time by 5 days
    advance_ledger_time(&env, 432_000); // 5 days
    
    // Calculate available amount after 5 days
    let stream: Stream = env.storage().persistent().get(&stream_id).unwrap();
    let elapsed = env.ledger().timestamp() - stream.start_time;
    let available_before_pause = calculate_available(&stream, elapsed);
    
    // Pause the stream
    env.mock_all_auths();
    pause_stream(&env, stream_id.clone());
    
    // Verify stream is paused
    let paused_stream: Stream = env.storage().persistent().get(&stream_id).unwrap();
    assert!(!paused_stream.is_active);
    
    // Advance time by another 5 days while paused
    advance_ledger_time(&env, 432_000); // 5 more days
    
    // Resume the stream
    resume_stream(&env, stream_id.clone());
    
    // Verify stream is active again
    let resumed_stream: Stream = env.storage().persistent().get(&stream_id).unwrap();
    assert!(resumed_stream.is_active);
    
    // Verify available amount hasn't changed during pause
    let elapsed_after_resume = env.ledger().timestamp() - resumed_stream.start_time;
    let available_after_resume = calculate_available(&resumed_stream, elapsed_after_resume);
    assert_eq!(available_before_pause, available_after_resume);
    
    // Withdraw some tokens
    let withdrawal_amount = available_after_resume / 2;
    withdraw_from_stream(&env, stream_id.clone(), withdrawal_amount);
    
    // Cancel the stream
    cancel_stream(&env, stream_id.clone());
    
    // Verify stream is inactive after cancellation
    let cancelled_stream: Stream = env.storage().persistent().get(&stream_id).unwrap();
    assert!(!cancelled_stream.is_active);
    
    // Attempt to withdraw after cancellation should fail
    let result = std::panic::catch_unwind(|| {
        withdraw_from_stream(&env, stream_id.clone(), 1);
    });
    assert!(result.is_err());
}

#[test]
fn test_micropayment_channel() {
    // Setup test environment
    let env = create_test_env();
    
    // Create test addresses
    let party_a = Address::random(&env);
    let party_b = Address::random(&env);
    
    // Set current contract address to party_a for authentication
    env.set_current_contract_address(party_a.clone());
    
    // Open a payment channel with a deposit
    let deposit = 1_000_000;
    env.mock_all_auths();
    let channel_id = open_channel(&env, party_b.clone(), deposit);
    
    // Verify channel was created correctly
    let channel: PaymentChannel = env.storage().persistent().get(&channel_id).unwrap();
    assert_eq!(channel.party_a, party_a);
    assert_eq!(channel.party_b, party_b);
    assert_eq!(channel.deposit, deposit);
    assert_eq!(channel.balance_a, deposit);
    assert_eq!(channel.balance_b, 0);
    assert!(!channel.is_closed);
    
    // Simulate off-chain voucher exchange
    // In a real scenario, party_a would sign vouchers off-chain
    // Here we'll just simulate the final state
    
    // Final state: party_a keeps 400,000, party_b gets 600,000
    let final_state = (400_000, 600_000);
    
    // Close the channel with the final state
    close_channel(&env, channel_id.clone(), final_state);
    
    // Verify channel is closed with correct balances
    let closed_channel: PaymentChannel = env.storage().persistent().get(&channel_id).unwrap();
    assert!(closed_channel.is_closed);
    assert_eq!(closed_channel.balance_a, final_state.0);
    assert_eq!(closed_channel.balance_b, final_state.1);
}

// Test invalid scenarios
#[test]
fn test_invalid_operations() {
    let env = create_test_env();
    let sender = Address::random(&env);
    let recipient = Address::random(&env);
    
    // Create a stream
    let total_amount = 10_000_000;
    let duration = 2_592_000u64; // 30 days
    let stream_id = create_stream(&env, sender.clone(), recipient.clone(), total_amount, duration);
    
    // Advance time by 5 days
    advance_ledger_time(&env, 432_000); // 5 days
    
    // Try to withdraw more than available
    env.mock_all_auths();
    let stream: Stream = env.storage().persistent().get(&stream_id).unwrap();
    let elapsed = env.ledger().timestamp() - stream.start_time;
    let available = calculate_available(&stream, elapsed);
    
    // This should fail
    let result = std::panic::catch_unwind(|| {
        withdraw_from_stream(&env, stream_id.clone(), available + 1);
    });
    assert!(result.is_err());
    
    // Test invalid channel operations
    env.set_current_contract_address(sender.clone());
    
    // Try to open channel with invalid deposit
    let result = std::panic::catch_unwind(|| {
        open_channel(&env, recipient.clone(), 0);
    });
    assert!(result.is_err());
    
    // Open a valid channel
    let deposit = 1_000_000;
    let channel_id = open_channel(&env, recipient.clone(), deposit);
    
    // Try to close with invalid final state
    let invalid_state = (600_000, 600_000); // Sum exceeds deposit
    let result = std::panic::catch_unwind(|| {
        close_channel(&env, channel_id.clone(), invalid_state);
    });
    assert!(result.is_err());
}