#![no_std]

use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, Vec};
use crate::voucher::SignedVoucher; 
use stream::{Stream, StreamState, PaymentSchedule, TimeUnit};
use channel::{PaymentChannel};

mod stream;
mod withdraw;
mod channel;
mod voucher;
mod balance;
mod error;
mod test;

#[contract]
pub struct PaymentStreamingContract;

#[contractimpl]
impl PaymentStreamingContract {
    // Stream functions
    pub fn create_stream(
        env: Env,
        sender: Address,
        recipient: Address,
        total_amount: i128,
        duration: u64,
        schedule: PaymentSchedule,
    ) -> BytesN<32> {
        stream::create_stream(&env, sender, recipient, total_amount, duration, schedule)
    }

    pub fn withdraw_from_stream(env: Env, stream_id: BytesN<32>, amount: i128) {
        withdraw::withdraw_from_stream(&env, stream_id, amount)
    }

    pub fn cancel_stream(env: Env, stream_id: BytesN<32>) {
        stream::cancel_stream(&env, stream_id)
    }

    pub fn pause_stream(env: Env, stream_id: BytesN<32>) {
        stream::pause_stream(&env, stream_id)
    }

    pub fn get_stream_balance(env: Env, stream_id: BytesN<32>) -> StreamState {
        balance::get_stream_balance(&env, stream_id)
    }

    // Channel functions
    pub fn open_channel(env: Env, counterparty: Address, deposit: i128) -> BytesN<32> {
        channel::open_channel(&env, counterparty, deposit)
    }

    pub fn sign_payment(env: Env, channel_id: BytesN<32>, increment_amount: i128,caller: BytesN<32>, // Ed25519 public key
        signature: BytesN<64>,) -> SignedVoucher {
        voucher::sign_payment(&env, channel_id, increment_amount, caller, signature)    }

    pub fn close_channel(env: Env, channel_id: BytesN<32>, final_state: (i128, i128)) {
        channel::close_channel(&env, channel_id, final_state)
    }
}
