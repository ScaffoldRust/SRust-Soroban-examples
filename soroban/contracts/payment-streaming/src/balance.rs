use soroban_sdk::{BytesN, Env, panic_with_error};
use crate::{error::PaymentStreamingError, stream::{Stream, StreamState}};

pub fn get_stream_balance(env: &Env, stream_id: BytesN<32>) -> StreamState {
    let stream: Stream = env.storage().persistent().get(&stream_id)
        .unwrap_or_else(|| panic_with_error!(env, PaymentStreamingError::StreamNotFound));

    let current_time = env.ledger().timestamp();
    let elapsed = current_time - stream.start_time;
    let available = if stream.is_active {
        calculate_available(&stream, elapsed)
    } else {
        0
    };

    StreamState {
        stream_id,
        withdrawn: stream.withdrawn,
        available,
        total: stream.total_amount,
    }
}

fn calculate_available(stream: &Stream, elapsed: u64) -> i128 {
    if elapsed >= stream.duration {
        stream.total_amount - stream.withdrawn
    } else {
        let rate = stream.total_amount / stream.duration as i128;
        (rate * elapsed as i128) - stream.withdrawn
    }
}