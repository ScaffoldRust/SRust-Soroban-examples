use crate::{
    error::PaymentStreamingError,
    stream::{Stream, StreamState},
};
use soroban_sdk::{panic_with_error, BytesN, Env};

pub fn get_stream_balance(env: &Env, stream_id: BytesN<32>) -> StreamState {
    let stream: Stream = env
        .storage()
        .persistent()
        .get(&stream_id)
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
        // Calculate proportional amount to avoid integer division precision loss
        let proportional_amount = (stream.total_amount * elapsed as i128) / stream.duration as i128;
        proportional_amount - stream.withdrawn
    }
}
