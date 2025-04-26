// stablecoin-contract/src/parameters.rs

use soroban_sdk::{contracttype, Env};

#[derive(Clone, Debug)]
#[contracttype]
pub struct StablecoinParameters {
    pub min_collateral_ratio: u32, // in basis points, e.g., 15000 = 150%
    pub max_mint_amount: i128,     // max stablecoin that can be minted at once
    pub rebalance_interval: u64,   // in seconds or blocks
}

#[contracttype]
pub enum DataKey {
    Parameters,
}

// Initialize default parameters
pub fn init_parameters(env: &Env, params: StablecoinParameters) {
    env.storage().instance().set(&DataKey::Parameters, &params);
}

// Get current parameters
pub fn get_parameters(env: &Env) -> StablecoinParameters {
    env.storage()
        .instance()
        .get(&DataKey::Parameters)
        .unwrap_or(StablecoinParameters {
            min_collateral_ratio: 15000, // 150%
            max_mint_amount: 1_000_000_000,
            rebalance_interval: 86400, // e.g., 1 day in seconds
        })
}

// Adjust parameters (requires authorization)
pub fn adjust_parameters(env: &Env, params: StablecoinParameters) {
    // Add authorization logic here (e.g., only governance)
    env.storage().instance().set(&DataKey::Parameters, &params);
}
