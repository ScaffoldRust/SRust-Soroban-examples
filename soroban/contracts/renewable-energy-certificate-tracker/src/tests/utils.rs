#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, Map, String};

use crate::{RenewableEnergyCertificateTracker, RenewableEnergyCertificateTrackerClient};

pub struct TestSetup {
    pub env: Env,
    pub client: RenewableEnergyCertificateTrackerClient<'static>,
    pub issuer: Address,
}

pub fn setup() -> TestSetup {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, RenewableEnergyCertificateTracker);
    let client = RenewableEnergyCertificateTrackerClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);

    client.initialize(&admin);
    client.register_issuer(&issuer, &String::from_str(&env, "Test issuer"));

    TestSetup {
        env,
        client,
        issuer,
    }
}

pub fn issue(setup: &TestSetup, capacity: i128) -> BytesN<32> {
    let mut hash = [7u8; 32];
    hash[0..8].copy_from_slice(&setup.env.ledger().timestamp().to_be_bytes());
    setup.client.issue_rec(
        &setup.issuer,
        &crate::EnergySource::Solar,
        &2024,
        &String::from_str(&setup.env, "Mexico"),
        &capacity,
        &String::from_str(&setup.env, "I-REC"),
        &BytesN::from_array(&setup.env, &hash),
        &Map::new(&setup.env),
    )
}
