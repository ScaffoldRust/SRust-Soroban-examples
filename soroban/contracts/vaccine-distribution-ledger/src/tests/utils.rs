#![cfg(test)]

use crate::{
    VaccineDistributionLedger, VaccineDistributionLedgerClient,
};
use soroban_sdk::{
    Env,
};

pub fn create_test_contract(env: &Env) -> VaccineDistributionLedgerClient<'_> {
    VaccineDistributionLedgerClient::new(env, &env.register(VaccineDistributionLedger {}, ()))
}
