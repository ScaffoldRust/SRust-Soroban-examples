// stablecoin-contract/src/lib.rs

#![no_std]

mod burn;
mod collateral;
mod governance;
mod mint;
mod oracle;
mod parameters;

// use soroban_sdk::{contractimpl, contracttype, contractclient, Vec, Env, Address, String, Symbol};
use soroban_sdk::{contract, contractimpl, Address, Env, String, Symbol, Vec};
// use soroban_sdk::contractclient;

#[contract]
pub struct StablecoinContract;

#[contractimpl]
impl StablecoinContract {
    // Collateral functions
    pub fn deposit_collateral(env: Env, user: Address, asset: Symbol, amount: i128) {
        collateral::deposit_collateral(&env, user, asset, amount);
    }

    pub fn withdraw_collateral(env: Env, user: Address, asset: Symbol, amount: i128) {
        collateral::withdraw_collateral(&env, user, asset, amount);
    }

    pub fn get_user_collateral(env: Env, user: Address, asset: Symbol) -> i128 {
        collateral::get_user_collateral(&env, user, asset)
    }

    pub fn get_total_reserve(env: Env, asset: Symbol) -> i128 {
        collateral::get_total_reserve(&env, asset)
    }

    // Minting
    pub fn mint(env: Env, user: Address, amount: i128, asset: Symbol) {
        mint::mint(&env, user, amount, asset);
    }

    // Burning
    pub fn burn(env: Env, user: Address, amount: i128, asset: Symbol) {
        burn::burn(&env, user, amount, asset);
    }

    // Parameters
    pub fn init_parameters(env: Env, params: parameters::StablecoinParameters) {
        parameters::init_parameters(&env, params);
    }

    pub fn get_parameters(env: Env) -> parameters::StablecoinParameters {
        parameters::get_parameters(&env)
    }

    pub fn adjust_parameters(env: Env, params: parameters::StablecoinParameters) {
        parameters::adjust_parameters(&env, params);
    }

    // Oracle
    pub fn set_price(env: Env, oracle: Address, asset_pair: Symbol, rate: i128, timestamp: u64) {
        oracle::set_price(&env, oracle, asset_pair, rate, timestamp);
    }

    pub fn get_price(env: Env, asset_pair: Symbol) -> Option<oracle::OraclePrice> {
        oracle::get_price(&env, asset_pair)
    }

    // Governance
    pub fn propose_governance_action(
        env: Env,
        proposer: Address,
        action_type: String,
        parameters: Vec<String>,
    ) -> u64 {
        governance::propose_governance_action(&env, proposer, action_type, parameters)
    }

    pub fn vote(env: Env, voter: Address, proposal_id: u64, vote: governance::Vote) {
        governance::vote(&env, voter, proposal_id, vote);
    }

    pub fn execute_proposal(env: Env, executor: Address, proposal_id: u64) {
        governance::execute_proposal(&env, executor, proposal_id);
    }
}
