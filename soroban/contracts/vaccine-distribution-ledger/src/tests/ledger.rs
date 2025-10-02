#![cfg(test)]

use crate::{
    distribution_storage::BatchStatus,
};
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env, String,
};

use super::utils::create_test_contract;

#[test]
fn test_contract_initialization() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    let admin = Address::generate(&env);

    // Initialize contract
    contract.initialize(&admin);

    // Verify admin can perform admin operations by trying to initialize again
    let result = contract.try_initialize(&admin);
    assert!(result.is_err()); // Should fail as already initialized
}

#[test]
fn test_batch_creation_and_basic_flow() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    let admin = Address::generate(&env);
    let manufacturer = Address::generate(&env);

    contract.initialize(&admin);

    // Create a vaccine batch
    let batch_id = String::from_str(&env, "PFIZER_001");
    let vaccine_type = String::from_str(&env, "COVID-19 mRNA");
    let production_date = 1640995200u64; // Jan 1, 2022
    let quantity = 1000u32;
    let expiry_date = 1672531200u64; // Jan 1, 2023

    contract.initialize_batch(
        &batch_id,
        &manufacturer,
        &vaccine_type,
        &production_date,
        &quantity,
        &expiry_date,
    );

    // Verify batch was created
    let batch = contract.get_batch(&batch_id);
    assert_eq!(batch.batch_id, batch_id);
    assert_eq!(batch.manufacturer, manufacturer);
    assert_eq!(batch.vaccine_type, vaccine_type);
    assert_eq!(batch.initial_quantity, quantity);
    assert_eq!(batch.current_quantity, quantity);
    assert_eq!(batch.status, BatchStatus::Produced);
}

#[test]
fn test_distribution_logging() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    let admin = Address::generate(&env);
    let manufacturer = Address::generate(&env);
    let distributor = Address::generate(&env);

    contract.initialize(&admin);

    // Create batch
    let batch_id = String::from_str(&env, "MODERNA_001");
    let vaccine_type = String::from_str(&env, "COVID-19 mRNA");
    
    contract.initialize_batch(
        &batch_id,
        &manufacturer,
        &vaccine_type,
        &1640995200u64,
        &1000u32,
        &1672531200u64,
    );

    // Log distribution
    let destination = String::from_str(&env, "Regional Health Center");
    let quantity = 100u32;
    let temp_log = Some(String::from_str(&env, "2-8°C maintained"));

    contract.log_distribution(
        &batch_id,
        &distributor,
        &destination,
        &quantity,
        &temp_log,
    );

    // Check batch inventory updated
    let remaining = contract.inventory_check(&batch_id);
    assert_eq!(remaining, 900);

    // Check batch status changed to InTransit
    let batch = contract.get_batch(&batch_id);
    assert_eq!(batch.status, BatchStatus::InTransit);
}

#[test]
fn test_batch_status_updates() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    let admin = Address::generate(&env);
    let manufacturer = Address::generate(&env);

    contract.initialize(&admin);

    // Create batch
    let batch_id = String::from_str(&env, "ASTRAZENECA_001");
    let vaccine_type = String::from_str(&env, "COVID-19 Viral Vector");

    contract.initialize_batch(
        &batch_id,
        &manufacturer,
        &vaccine_type,
        &1640995200u64,
        &800u32,
        &1672531200u64,
    );

    // Update status to Distributed
    let notes = Some(String::from_str(&env, "Distributed to regional centers"));
    
    contract.update_batch_status(
        &batch_id,
        &manufacturer,
        &BatchStatus::Distributed,
        &notes,
    );

    // Verify status changed
    let batch = contract.get_batch(&batch_id);
    assert_eq!(batch.status, BatchStatus::Distributed);
    assert_eq!(batch.notes, notes);
}

#[test]
fn test_insufficient_inventory_withdrawal() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    let admin = Address::generate(&env);
    let manufacturer = Address::generate(&env);
    let distributor = Address::generate(&env);

    contract.initialize(&admin);

    // Create small batch
    let batch_id = String::from_str(&env, "SMALL_BATCH");
    
    contract.initialize_batch(
        &batch_id,
        &manufacturer,
        &String::from_str(&env, "COVID-19 mRNA"),
        &1640995200u64,
        &50u32,
        &1672531200u64,
    );

    // Try to distribute more than available
    let result = contract.try_log_distribution(
        &batch_id,
        &distributor,
        &String::from_str(&env, "Hospital"),
        &100u32, // More than the 50 available
        &None,
    );
    assert!(result.is_err());
}

#[test]
fn test_expired_batch_handling() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    let admin = Address::generate(&env);
    let manufacturer = Address::generate(&env);
    let distributor = Address::generate(&env);

    contract.initialize(&admin);

    // Create batch with past expiry date
    let batch_id = String::from_str(&env, "EXPIRED_BATCH");
    let production_date = 1640995200u64; // Jan 1, 2022
    let expiry_date = 1641081600u64;     // Jan 2, 2022 (very short expiry)
    
    contract.initialize_batch(
        &batch_id,
        &manufacturer,
        &String::from_str(&env, "COVID-19 mRNA"),
        &production_date,
        &500u32,
        &expiry_date,
    );

    // Fast forward time past expiry
    env.ledger().with_mut(|li| {
        li.timestamp = 1641168000; // Jan 3, 2022
    });

    // Try to distribute expired batch
    let result = contract.try_log_distribution(
        &batch_id,
        &distributor,
        &String::from_str(&env, "Hospital"),
        &10u32,
        &None,
    );
    assert!(result.is_err());
}

#[test]
fn test_error_conditions() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = create_test_contract(&env);
    let admin = Address::generate(&env);
    let manufacturer = Address::generate(&env);

    contract.initialize(&admin);

    // Test duplicate batch creation
    let batch_id = String::from_str(&env, "DUPLICATE_TEST");
    
    contract.initialize_batch(
        &batch_id,
        &manufacturer,
        &String::from_str(&env, "COVID-19 mRNA"),
        &1640995200u64,
        &500u32,
        &1672531200u64,
    );

    // Try to create same batch again
    let result = contract.try_initialize_batch(
        &batch_id,
        &manufacturer,
        &String::from_str(&env, "COVID-19 mRNA"),
        &1640995200u64,
        &500u32,
        &1672531200u64,
    );
    assert!(result.is_err());

    // Test invalid quantity
    let result = contract.try_initialize_batch(
        &String::from_str(&env, "INVALID_QTY"),
        &manufacturer,
        &String::from_str(&env, "COVID-19 mRNA"),
        &1640995200u64,
        &0u32, // Invalid quantity
        &1672531200u64,
    );
    assert!(result.is_err());

    // Test non-existent batch query
    let result = contract.try_get_batch(&String::from_str(&env, "NON_EXISTENT"));
    assert!(result.is_err());
}
