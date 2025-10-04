#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _},
    Address, Env, String,
};

use crate::{
    DistributedEnergyResourceManager, DistributedEnergyResourceManagerClient, ResourceType,
    DERStatus
};

// ============ RESOURCE OPTIMIZATION TESTS ============

#[test]
fn test_optimize_resources() {
    let env = Env::default();
    let contract_id = env.register(DistributedEnergyResourceManager, ());
    let client = DistributedEnergyResourceManagerClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let der_owner = Address::generate(&env);
    let operator = Address::generate(&env);

    client.initialize(&admin);

    // Add grid operator
    let operator_name = String::from_str(&env, "Grid Operator 1");
    client.add_grid_operator(&admin, &operator, &operator_name, &5);

    // Register some DERs
    let solar_der = String::from_str(&env, "SOLAR_001");
    let battery_der = String::from_str(&env, "BATTERY_001");
    let location = String::from_str(&env, "Test Location");

    client.register_der(&der_owner, &solar_der, &ResourceType::Solar, &1000, &location);
    client.register_der(&der_owner, &battery_der, &ResourceType::Battery, &500, &location);

    // Optimize resources
    let schedules = client.optimize_resources(&operator);

    // Verify optimization schedules were created
    assert_eq!(schedules.len(), 2);

    // Check that we can retrieve the schedules
    let retrieved_schedules = client.get_optimization_schedules();
    assert_eq!(retrieved_schedules.len(), 2);
}

#[test]
fn test_emergency_allocation() {
    let env = Env::default();
    let contract_id = env.register(DistributedEnergyResourceManager, ());
    let client = DistributedEnergyResourceManagerClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let der_owner = Address::generate(&env);
    let operator = Address::generate(&env);

    client.initialize(&admin);

    // Add grid operator
    let operator_name = String::from_str(&env, "Grid Operator 1");
    client.add_grid_operator(&admin, &operator, &operator_name, &5);

    // Register a DER
    let der_id = String::from_str(&env, "BATTERY_001");
    let location = String::from_str(&env, "Test Location");

    client.register_der(&der_owner, &der_id, &ResourceType::Battery, &500, &location);

    // Emergency allocation
    let result = client.emergency_allocation(
        &operator,
        &der_id,
        &300, // 300 kW required
        &3600 // 1 hour duration
    );

    assert!(result);

    // Verify DER status changed to emergency
    let der_info = client.get_der_info(&der_id);
    assert_eq!(der_info.status, DERStatus::Emergency);
}

// ============ EDGE CASE TESTS ============

#[test]
#[should_panic(expected = "DER not found")]
fn test_emergency_allocation_non_existent_der() {
    let env = Env::default();
    let contract_id = env.register(DistributedEnergyResourceManager, ());
    let client = DistributedEnergyResourceManagerClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let operator = Address::generate(&env);

    client.initialize(&admin);

    // Add grid operator
    let operator_name = String::from_str(&env, "Grid Operator 1");
    client.add_grid_operator(&admin, &operator, &operator_name, &5);

    let der_id = String::from_str(&env, "NON_EXISTENT_DER");

    // Attempt emergency allocation on non-existent DER - should panic
    client.emergency_allocation(&operator, &der_id, &300, &3600);
}

#[test]
#[should_panic(expected = "DER is not available for emergency allocation")]
fn test_emergency_allocation_offline_der() {
    let env = Env::default();
    let contract_id = env.register(DistributedEnergyResourceManager, ());
    let client = DistributedEnergyResourceManagerClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let der_owner = Address::generate(&env);
    let operator = Address::generate(&env);

    client.initialize(&admin);

    // Add grid operator
    let operator_name = String::from_str(&env, "Grid Operator 1");
    client.add_grid_operator(&admin, &operator, &operator_name, &5);

    // Register a DER
    let der_id = String::from_str(&env, "BATTERY_001");
    let location = String::from_str(&env, "Test Location");

    client.register_der(&der_owner, &der_id, &ResourceType::Battery, &500, &location);

    // Set DER to offline
    client.update_status(&der_owner, &der_id, &DERStatus::Offline);

    // Attempt emergency allocation on offline DER - should panic
    client.emergency_allocation(&operator, &der_id, &300, &3600);
}

#[test]
#[should_panic(expected = "Required power exceeds DER capacity")]
fn test_emergency_allocation_exceeds_capacity() {
    let env = Env::default();
    let contract_id = env.register(DistributedEnergyResourceManager, ());
    let client = DistributedEnergyResourceManagerClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let der_owner = Address::generate(&env);
    let operator = Address::generate(&env);

    client.initialize(&admin);

    // Add grid operator
    let operator_name = String::from_str(&env, "Grid Operator 1");
    client.add_grid_operator(&admin, &operator, &operator_name, &5);

    // Register a DER with 500 capacity
    let der_id = String::from_str(&env, "BATTERY_001");
    let location = String::from_str(&env, "Test Location");

    client.register_der(&der_owner, &der_id, &ResourceType::Battery, &500, &location);

    // Attempt to allocate more than capacity - should panic
    client.emergency_allocation(&operator, &der_id, &600, &3600);
}

#[test]
fn test_optimize_with_mixed_status_ders() {
    let env = Env::default();
    let contract_id = env.register(DistributedEnergyResourceManager, ());
    let client = DistributedEnergyResourceManagerClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let der_owner = Address::generate(&env);
    let operator = Address::generate(&env);

    client.initialize(&admin);

    // Add grid operator
    let operator_name = String::from_str(&env, "Grid Operator 1");
    client.add_grid_operator(&admin, &operator, &operator_name, &5);

    let location = String::from_str(&env, "Test Location");

    // Register DERs with different statuses
    let solar_der = String::from_str(&env, "SOLAR_001");
    let wind_der = String::from_str(&env, "WIND_001");
    let battery_der = String::from_str(&env, "BATTERY_001");

    client.register_der(&der_owner, &solar_der, &ResourceType::Solar, &1000, &location);
    client.register_der(&der_owner, &wind_der, &ResourceType::Wind, &500, &location);
    client.register_der(&der_owner, &battery_der, &ResourceType::Battery, &200, &location);

    // Set one DER to offline
    client.update_status(&der_owner, &battery_der, &DERStatus::Offline);

    // Optimize resources - should only include online/optimized DERs
    let schedules = client.optimize_resources(&operator);

    // Only solar and wind should be in optimization (battery is offline)
    assert_eq!(schedules.len(), 2);
}

#[test]
fn test_optimize_empty_der_list() {
    let env = Env::default();
    let contract_id = env.register(DistributedEnergyResourceManager, ());
    let client = DistributedEnergyResourceManagerClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let operator = Address::generate(&env);

    client.initialize(&admin);

    // Add grid operator
    let operator_name = String::from_str(&env, "Grid Operator 1");
    client.add_grid_operator(&admin, &operator, &operator_name, &5);

    // Optimize with no DERs registered
    let schedules = client.optimize_resources(&operator);

    // Should return empty schedule
    assert_eq!(schedules.len(), 0);
}

#[test]
fn test_optimization_all_resource_types() {
    let env = Env::default();
    let contract_id = env.register(DistributedEnergyResourceManager, ());
    let client = DistributedEnergyResourceManagerClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let der_owner = Address::generate(&env);
    let operator = Address::generate(&env);

    client.initialize(&admin);

    // Add grid operator
    let operator_name = String::from_str(&env, "Grid Operator 1");
    client.add_grid_operator(&admin, &operator, &operator_name, &5);

    let location = String::from_str(&env, "Test Location");

    // Register all resource types
    let resource_types = [
        (ResourceType::Solar, "SOLAR_001", 1000),
        (ResourceType::Wind, "WIND_001", 500),
        (ResourceType::Battery, "BATTERY_001", 200),
        (ResourceType::Hydro, "HYDRO_001", 800),
        (ResourceType::Geothermal, "GEO_001", 600),
        (ResourceType::FuelCell, "FUEL_001", 100),
    ];

    for (resource_type, der_id, capacity) in resource_types {
        let der_id_str = String::from_str(&env, der_id);
        client.register_der(&der_owner, &der_id_str, &resource_type, &capacity, &location);
    }

    // Optimize all resources
    let schedules = client.optimize_resources(&operator);

    // Should create schedules for all 6 resource types
    assert_eq!(schedules.len(), 6);

    // Verify schedule priorities are set (different for each type)
    // Battery should have highest priority (9), Geothermal lowest (4)
    let mut has_battery_priority = false;
    let mut has_geothermal_priority = false;

    for schedule in schedules.iter() {
        if schedule.priority == 9 {
            has_battery_priority = true;
        }
        if schedule.priority == 4 {
            has_geothermal_priority = true;
        }
    }

    assert!(has_battery_priority, "Battery priority (9) should be present");
    assert!(has_geothermal_priority, "Geothermal priority (4) should be present");
}

