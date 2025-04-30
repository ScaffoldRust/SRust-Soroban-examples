use crate::types::*;
use soroban_sdk::{Env, String};

pub fn update_fx_rate(env: Env, source_currency: String, target_currency: String, rate: i128) {
    // In practice, restrict this to an authorized oracle or admin
    let fx_rate = ExchangeRate {
        source_currency: source_currency.clone(),
        target_currency: target_currency.clone(),
        rate,
        timestamp: env.ledger().timestamp(),
    };
    env.storage().instance().set(
        &DataKey::ExchangeRate(source_currency, target_currency),
        &fx_rate,
    );
}

pub fn get_fx_rate(env: Env, source_currency: String, target_currency: String) -> i128 {
    let fx_rate: ExchangeRate = env
        .storage()
        .instance()
        .get(&DataKey::ExchangeRate(
            source_currency.clone(),
            target_currency.clone(),
        ))
        .unwrap_or(ExchangeRate {
            source_currency,
            target_currency,
            rate: RATE_SCALE, // Default rate (1:1)
            timestamp: 0,
        });
    fx_rate.rate
}
