use soroban_sdk::{contracttype, panic_with_error, Address, BytesN, Env};

use crate::error::PaymentStreamingError;

#[contracttype]
#[derive(Clone)]
pub struct PaymentChannel {
    pub channel_id: BytesN<32>,
    pub party_a: Address,
    pub party_b: Address,
    pub deposit: i128,
    pub balance_a: i128,
    pub balance_b: i128,
    pub is_closed: bool,
}

const CHANNEL_KEY: &str = "CHANNEL";

pub fn open_channel(
    env: &Env,
    sender: Address,
    counterparty: Address,
    deposit: i128,
) -> BytesN<32> {
    if deposit <= 0 {
        panic_with_error!(env, PaymentStreamingError::InvalidDeposit);
    }

    sender.require_auth();

    let current_channel_id = env
        .storage()
        .instance()
        .get(&CHANNEL_KEY)
        .unwrap_or(BytesN::from_array(&env, &[0; 32]));
    let channel_id = increment_bytesn(&env, current_channel_id);
    let channel = PaymentChannel {
        channel_id: channel_id.clone(),
        party_a: sender,
        party_b: counterparty,
        deposit,
        balance_a: deposit,
        balance_b: 0,
        is_closed: false,
    };

    env.storage().persistent().set(&channel_id, &channel);
    env.storage().instance().set(&CHANNEL_KEY, &channel_id);

    channel_id
}

pub fn close_channel(env: &Env, channel_id: BytesN<32>, final_state: (i128, i128)) {
    let mut channel: PaymentChannel = env
        .storage()
        .persistent()
        .get(&channel_id)
        .unwrap_or_else(|| panic_with_error!(env, PaymentStreamingError::ChannelNotFound));

    channel.party_a.require_auth();
    channel.party_b.require_auth();

    if channel.is_closed {
        panic_with_error!(env, PaymentStreamingError::ChannelAlreadyClosed);
    }

    let (final_a, final_b) = final_state;
    if final_a + final_b != channel.deposit {
        panic_with_error!(env, PaymentStreamingError::InvalidFinalState);
    }

    channel.balance_a = final_a;
    channel.balance_b = final_b;
    channel.is_closed = true;

    env.storage().persistent().set(&channel_id, &channel);
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
