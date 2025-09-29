#![cfg(test)]

use crate::{EnergyTradingMarketplace, EnergyTradingMarketplaceClient};
use soroban_sdk::{testutils::Address as _, Address, Env};

pub fn setup_test_environment() -> (
    Env,
    EnergyTradingMarketplaceClient<'static>,
    Address,
    Address,
    Address,
    Address,
) {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token_contract = create_test_token(&env, &admin);
    let producer = Address::generate(&env);
    let consumer = Address::generate(&env);

    let contract_id = env.register(EnergyTradingMarketplace, ());
    let client = EnergyTradingMarketplaceClient::new(&env, &contract_id);

    // Initialize contract
    client.initialize(&admin, &token_contract, &10u64, &1000u64);

    // Register traders
    client.register_producer(&producer);
    client.register_consumer(&consumer);

    (env, client, admin, token_contract, producer, consumer)
}

pub fn create_test_token(env: &Env, admin: &Address) -> Address {
    env.register_stellar_asset_contract_v2(admin.clone())
        .address()
}

pub fn mint_tokens(
    env: &Env,
    token_address: &Address,
    _admin: &Address,
    to: &Address,
    amount: i128,
) {
    use soroban_sdk::token;
    let token = token::StellarAssetClient::new(env, token_address);
    token.mint(to, &amount);
}
