#![cfg(test)]

use soroban_sdk::{testutils::{Address as _,}, Address, Env, Vec, String};
use crate::{OrganDonationMatchingContract, OrganDonationMatchingContractClient};
use crate::HLAProfile;

pub fn setup_test_environment() -> (Env, OrganDonationMatchingContractClient<'static>, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(OrganDonationMatchingContract, ());
    let client = OrganDonationMatchingContractClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    
    (env, client, admin)
}

pub fn create_test_hla_profile(env: &Env) -> HLAProfile {
        let mut hla_a = Vec::new(env);
        hla_a.push_back(String::from_str(env, "A*02:01"));
        hla_a.push_back(String::from_str(env, "A*01:01"));

        let mut hla_b = Vec::new(env);
        hla_b.push_back(String::from_str(env, "B*07:02"));
        hla_b.push_back(String::from_str(env, "B*08:01"));

        let mut hla_dr = Vec::new(env);
        hla_dr.push_back(String::from_str(env, "DRB1*15:01"));
        hla_dr.push_back(String::from_str(env, "DRB1*03:01"));

        HLAProfile {
            hla_a,
            hla_b,
            hla_dr,
        }
    }


pub fn create_test_hla_profile_with_alleles(
    env: &Env,
    hla_a_alleles: &[&str],
    hla_b_alleles: &[&str],
    hla_dr_alleles: &[&str],
) -> HLAProfile {
    let mut hla_a = Vec::new(env);
    for allele in hla_a_alleles {
        hla_a.push_back(String::from_str(env, allele));
    }
    let mut hla_b = Vec::new(env);
    for allele in hla_b_alleles {
        hla_b.push_back(String::from_str(env, allele));
    }
    let mut hla_dr = Vec::new(env);
    for allele in hla_dr_alleles {
        hla_dr.push_back(String::from_str(env, allele));
    }
    HLAProfile { hla_a, hla_b, hla_dr }
}