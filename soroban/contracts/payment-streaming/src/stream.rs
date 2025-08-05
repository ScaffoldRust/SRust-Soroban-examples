use soroban_sdk::{contracttype, panic_with_error, Address, BytesN, Env};

use crate::error::PaymentStreamingError;

#[contracttype]
#[derive(Clone)]
pub enum TimeUnit {
    Seconds,
    Blocks,
}

#[contracttype]
#[derive(Clone)]
pub struct PaymentSchedule {
    pub unit: TimeUnit,
    pub interval: u64,
    pub release_rate: i128,
}

#[contracttype]
#[derive(Clone)]
pub struct Stream {
    pub sender: Address,
    pub recipient: Address,
    pub total_amount: i128,
    pub start_time: u64,
    pub duration: u64,
    pub withdrawn: i128,
    pub is_active: bool,
    pub schedule: PaymentSchedule,
}

#[contracttype]
#[derive(Clone)]
pub struct StreamState {
    pub stream_id: BytesN<32>,
    pub withdrawn: i128,
    pub available: i128,
    pub total: i128,
}

const STREAM_KEY: &str = "STREAM";

pub fn create_stream(
    env: &Env,
    sender: Address,
    recipient: Address,
    total_amount: i128,
    duration: u64,
    schedule: PaymentSchedule,
) -> BytesN<32> {
    if total_amount <= 0 || duration == 0 {
        panic_with_error!(env, PaymentStreamingError::InvalidParameters);
    }

    sender.require_auth();

    let current_stream_id = env
        .storage()
        .instance()
        .get(&STREAM_KEY)
        .unwrap_or(BytesN::from_array(&env, &[0; 32]));

    let stream_id = increment_bytesn(&env, current_stream_id);
    let stream = Stream {
        sender,
        recipient,
        total_amount,
        start_time: env.ledger().timestamp(),
        duration,
        withdrawn: 0,
        is_active: true,
        schedule,
    };

    env.storage().persistent().set(&stream_id, &stream);
    env.storage().instance().set(&STREAM_KEY, &stream_id);

    stream_id
}

pub fn cancel_stream(env: &Env, stream_id: BytesN<32>) {
    let mut stream: Stream = env
        .storage()
        .persistent()
        .get(&stream_id)
        .unwrap_or_else(|| panic_with_error!(env, PaymentStreamingError::StreamNotFound));

    stream.sender.require_auth();

    if !stream.is_active {
        panic_with_error!(env, PaymentStreamingError::StreamNotActive);
    }

    stream.is_active = false;
    env.storage().persistent().set(&stream_id, &stream);
}

pub fn pause_stream(env: &Env, stream_id: BytesN<32>) {
    let mut stream: Stream = env
        .storage()
        .persistent()
        .get(&stream_id)
        .unwrap_or_else(|| panic_with_error!(env, PaymentStreamingError::StreamNotFound));

    stream.sender.require_auth();

    if !stream.is_active {
        panic_with_error!(env, PaymentStreamingError::StreamAllreadyPaused);
    }

    stream.is_active = false;
    env.storage().persistent().set(&stream_id, &stream);
}
fn increment_bytesn(env: &Env, bytes: BytesN<32>) -> BytesN<32> {
    let mut byte_array = bytes.to_array();
    // Increment the byte array (little-endian)
    for i in (0..byte_array.len()).rev() {
        if byte_array[i] == 255 {
            byte_array[i] = 0;
        } else {
            byte_array[i] += 1;
            break;
        }
    }
    BytesN::from_array(env, &byte_array)
}
