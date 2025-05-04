#![no_std]
use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, String, Symbol};

mod compliance;
mod fees;
mod fx;
mod settlement;
mod transfer;
mod types;

pub use compliance::*;
pub use fees::*;
pub use fx::*;
pub use settlement::*;
pub use transfer::*;
pub use types::*;

#[contract]
pub struct CrossBorderPayment;

#[contractimpl]
impl CrossBorderPayment {
    // Initialize the contract
    pub fn initialize(
        env: Env,
        admin: Address,
        base_fee: i128,
        percentage: u32,
        urgency_multiplier: u32,
    ) {
        admin.require_auth();
        let fee_structure = FeeStructure {
            base_fee,
            percentage,
            urgency_multiplier,
        };
        env.storage().instance().set(&DataKey::Fees, &fee_structure);
        env.storage()
            .instance()
            .set(&DataKey::NextTransferId, &1u64);

        env.events().publish(
            (Symbol::new(&env, "Initialized"),),
            (admin, base_fee, percentage, urgency_multiplier),
        );
    }

    // Transfer functions
    pub fn initiate_transfer(
        env: Env,
        sender: Address,
        recipient: Address,
        amount: i128,
        currency: String,
        destination_network: String,
    ) -> u64 {
        transfer::initiate_transfer(
            env,
            sender,
            recipient,
            amount,
            currency,
            destination_network,
        )
    }

    // Compliance functions
    pub fn verify_compliance(env: Env, user: Address, documents: BytesN<32>) {
        compliance::verify_compliance(env, user, documents)
    }

    // Settlement functions
    pub fn execute_settlement(env: Env, transfer_id: u64, org: Address) {
        settlement::execute_settlement(env, transfer_id, org)
    }

    pub fn refund_transfer(env: Env, transfer_id: u64, org: Address) {
        settlement::refund_transfer(env, transfer_id, org)
    }

    // Fee functions
    pub fn calculate_fees(env: Env, amount: i128, is_urgent: bool) -> i128 {
        fees::calculate_fees(env, amount, is_urgent)
    }

    // FX functions
    pub fn update_fx_rate(env: Env, source_currency: String, target_currency: String, rate: i128) {
        fx::update_fx_rate(env, source_currency, target_currency, rate)
    }

    pub fn get_fx_rate(env: Env, source_currency: String, target_currency: String) -> i128 {
        fx::get_fx_rate(env, source_currency, target_currency)
    }

    // Query functions
    pub fn get_transfer_status(env: Env, transfer_id: u64) -> SettlementStatus {
        settlement::get_transfer_status(env, transfer_id)
    }

    pub fn get_transfer_details(env: Env, transfer_id: u64) -> TransferRequest {
        transfer::get_transfer_details(env, transfer_id)
    }
}
