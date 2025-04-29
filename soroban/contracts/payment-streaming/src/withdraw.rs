use soroban_sdk::{contracttype, Address, BytesN, Env, panic_with_error};
use crate::{error::PaymentStreamingError, stream::{Stream, StreamState}};

pub fn withdraw_from_stream(env: &Env, stream_id: BytesN<32>, amount: i128) {
    let mut stream: Stream = env.storage().persistent().get(&stream_id)
        .unwrap_or_else(|| panic_with_error!(env, PaymentStreamingError::StreamNotFound));
    
    stream.recipient.require_auth();
    
    if !stream.is_active {
        panic_with_error!(env, PaymentStreamingError::StreamNotActive);
    }

    let current_time = env.ledger().timestamp();
    let elapsed = current_time - stream.start_time;
    let available = calculate_available(&stream, elapsed);
    
    if amount > available {
        panic_with_error!(env,PaymentStreamingError::InsufficientFunds);
    }

    stream.withdrawn += amount;
    env.storage().persistent().set(&stream_id, &stream);
}

fn calculate_available(stream: &Stream, elapsed: u64) -> i128 {
    if elapsed >= stream.duration {
        stream.total_amount - stream.withdrawn
    } else {
        let rate = stream.total_amount / stream.duration as i128;
        (rate * elapsed as i128) - stream.withdrawn
    }
}