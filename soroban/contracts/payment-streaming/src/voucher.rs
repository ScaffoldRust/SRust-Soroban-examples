use soroban_sdk::{contracttype, panic_with_error, Bytes, BytesN, Env};

#[contracttype]
#[derive(Clone)]
pub struct SignedVoucher {
    pub channel_id: BytesN<32>,
    pub increment_amount: i128,
    pub signature: BytesN<64>,
}

use crate::error::PaymentStreamingError;
use crate::channel::PaymentChannel;

pub fn sign_payment(
    env: &Env,
    channel_id: BytesN<32>,
    increment_amount: i128,
    caller: BytesN<32>,
    signature: BytesN<64>,
) -> SignedVoucher {
    let channel: PaymentChannel = env.storage().persistent().get(&channel_id)
        .unwrap_or_else(|| panic_with_error!(env, PaymentStreamingError::ChannelNotFound));

    if channel.is_closed {
        panic_with_error!(env, PaymentStreamingError::ChannelIsClosed); 
    }

    if increment_amount <= 0 {
        panic_with_error!(env, PaymentStreamingError::InvalidAmount); 
    }


    // Create message to verify
    let mut message: Bytes = Bytes::new(env);
    message.extend_from_slice(&increment_amount.to_le_bytes());

    // Verify the signature
    env.crypto().ed25519_verify(&caller, &message, &signature);

    SignedVoucher {
        channel_id,
        increment_amount,
        signature,
    }
}