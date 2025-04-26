// stablecoin-contract/src/oracle.rs

use soroban_sdk::{contracttype, symbol_short, Address, Env, Symbol};

#[derive(Clone, Debug)]
#[contracttype]
pub struct OraclePrice {
    pub source: Symbol,     // Oracle source ID, e.g. "chainlink"
    pub asset_pair: Symbol, // E.g., "USDC/XLM"
    pub rate: i128,         // Price rate as an integer (scaled by 1e8, for example)
    pub timestamp: u64,     // Unix timestamp of the price
}

#[contracttype]
pub enum DataKey {
    Price(Symbol), // Keyed by asset_pair
}

// Set price feed from an authorized oracle address
pub fn set_price(env: &Env, oracle: Address, asset_pair: Symbol, rate: i128, timestamp: u64) {
    oracle.require_auth();

    let current = env
        .storage()
        .instance()
        .get::<_, OraclePrice>(&DataKey::Price(asset_pair.clone()));

    // If a price exists, check timestamp to prevent replay
    if let Some(price) = current {
        assert!(timestamp > price.timestamp, "Price update too old");
    }

    let price = OraclePrice {
        source: symbol_short!("oracle"),
        asset_pair: asset_pair.clone(),
        rate,
        timestamp,
    };

    env.storage()
        .instance()
        .set(&DataKey::Price(asset_pair), &price);
}

// Get the latest price for an asset pair
pub fn get_price(env: &Env, asset_pair: Symbol) -> Option<OraclePrice> {
    env.storage().instance().get(&DataKey::Price(asset_pair))
}
