#![cfg(test)]

use crate::{utils::*, PeerToPeerEnergySharing, PeerToPeerEnergySharingClient};
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env,
};

pub struct TestSetup {
    pub env: Env,
    pub client: PeerToPeerEnergySharingClient<'static>,
    pub admin: Address,
    pub token_contract: Address,
    pub provider: Address,
    pub consumer: Address,
    pub prosumer3: Address,
}

impl TestSetup {
    pub fn new() -> Self {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let token_contract = create_test_token(&env, &admin);
        let provider = Address::generate(&env);
        let consumer = Address::generate(&env);
        let prosumer3 = Address::generate(&env);

        let contract_id = env.register(PeerToPeerEnergySharing, ());
        let client = PeerToPeerEnergySharingClient::new(&env, &contract_id);

        // Initialize contract
        client.initialize(&admin, &token_contract);

        // Register prosumers
        client.register_prosumer(&provider);
        client.register_prosumer(&consumer);
        client.register_prosumer(&prosumer3);

        // Mint tokens for testing payments
        mint_tokens(&env, &token_contract, &admin, &consumer, 10_000_000);
        mint_tokens(&env, &token_contract, &admin, &prosumer3, 5_000_000);

        Self {
            env,
            client,
            admin,
            token_contract,
            provider,
            consumer,
            prosumer3,
        }
    }

    pub fn advance_ledger_time(&self, seconds: u64) {
        self.env
            .ledger()
            .set_timestamp(self.env.ledger().timestamp() + seconds);
    }

    pub fn get_future_deadline(&self) -> u64 {
        self.env.ledger().timestamp() + 86400 // 1 day from now
    }

    pub fn create_test_agreement(&self) -> u64 {
        self.client.create_agreement(
            &self.provider,
            &self.consumer,
            &100u64, // 100 kWh
            &50u64,  // 50 units per kWh
            &self.get_future_deadline(),
        )
    }

    pub fn create_custom_agreement(
        &self,
        provider: &Address,
        consumer: &Address,
        energy_amount: u64,
        price_per_kwh: u64,
        deadline: u64,
    ) -> u64 {
        self.client.create_agreement(
            provider,
            consumer,
            &energy_amount,
            &price_per_kwh,
            &deadline,
        )
    }
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

pub fn get_token_balance(env: &Env, token_address: &Address, account: &Address) -> i128 {
    use soroban_sdk::token;
    let token = token::Client::new(env, token_address);
    token.balance(account)
}
