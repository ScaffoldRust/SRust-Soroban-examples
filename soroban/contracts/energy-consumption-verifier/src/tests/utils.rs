#![cfg(test)]

use soroban_sdk::{testutils::{Address as _,}, Address, Env};
use crate::{EnergyConsumptionVerifier, EnergyConsumptionVerifierClient};

pub fn setup_test_environment() -> (Env, EnergyConsumptionVerifierClient<'static>, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(EnergyConsumptionVerifier, ());
    let client = EnergyConsumptionVerifierClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    
    (env, client, admin)
}