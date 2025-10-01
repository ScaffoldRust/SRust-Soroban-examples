#![no_std]

use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, Symbol, String};

mod payment;
mod gateway;
mod utils;

#[cfg(test)]
mod test;
#[cfg(test)]
mod tests; // additional modular payment lifecycle tests (payment, confirmation, refund, stress)

#[contract]
pub struct TelemedicinePaymentGateway;

#[contractimpl]
impl TelemedicinePaymentGateway {
    /// Initialize the telemedicine payment gateway
    pub fn initialize(
        env: Env,
        admin: Address,
        platform_fee_percentage: u32,
        min_session_duration: u64,
        max_session_duration: u64,
    ) {
        gateway::initialize(&env, admin, platform_fee_percentage, min_session_duration, max_session_duration);
    }

    /// Register a healthcare provider
    pub fn register_provider(
        env: Env,
        provider_address: Address,
        provider_name: String,
        specialty: String,
        hourly_rate: i128,
        currency: Address,
    ) {
        gateway::register_provider(&env, provider_address, provider_name, specialty, hourly_rate, currency);
    }

    /// Initiate a payment for a telemedicine session
    pub fn initiate_payment(
        env: Env,
        patient: Address,
        provider: Address,
        session_id: BytesN<32>,
        estimated_duration: u64,
        consent_hash: BytesN<32>,
    ) -> BytesN<32> {
        payment::initiate_payment(&env, patient, provider, session_id, estimated_duration, consent_hash)
    }

    /// Confirm session completion and release payment
    pub fn confirm_session(
        env: Env,
        provider: Address,
        payment_id: BytesN<32>,
        actual_duration: u64,
        session_notes: String,
    ) {
        payment::confirm_session(&env, provider, payment_id, actual_duration, session_notes);
    }

    /// Process refund for canceled or incomplete sessions
    pub fn refund_payment(
        env: Env,
        caller: Address,
        payment_id: BytesN<32>,
        reason: String,
    ) {
        payment::refund_payment(&env, caller, payment_id, reason);
    }

    /// Get payment status
    pub fn get_payment_status(env: Env, payment_id: BytesN<32>) -> u32 {
        payment::get_payment_status(&env, payment_id)
    }

    /// Get payment amount
    pub fn get_payment_amount(env: Env, payment_id: BytesN<32>) -> i128 {
        payment::get_payment_amount(&env, payment_id)
    }

    /// Get provider hourly rate
    pub fn get_provider_hourly_rate(env: Env, provider: Address) -> i128 {
        gateway::get_provider_hourly_rate(&env, provider)
    }

    /// Get platform fee percentage
    pub fn get_platform_fee_percentage(env: Env) -> u32 {
        gateway::get_platform_fee_percentage(&env)
    }

    /// Get balance for an address
    pub fn get_balance(env: Env, address: Address) -> (i128, i128) {
        payment::get_balance(&env, address)
    }

    /// Pause contract operations
    pub fn pause_contract(env: Env) {
        gateway::pause_contract(&env);
    }

    /// Resume contract operations
    pub fn resume_contract(env: Env) {
        gateway::resume_contract(&env);
    }

    /// Get contract status
    pub fn get_contract_status(env: Env) -> bool {
        gateway::get_contract_status(&env)
    }
}