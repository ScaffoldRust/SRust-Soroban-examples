#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _},
    Address, Env, String,
};

use crate::{
    DistributedEnergyResourceManager, DistributedEnergyResourceManagerClient, ResourceType,
    DERStatus
};

// ============ INITIALIZATION TESTS ============

#[test]
fn test_initialize() {
    let env = Env::default();
    let contract_id = env.register(DistributedEnergyResourceManager, ());
    let client = DistributedEnergyResourceManagerClient::new(&env, &contract_id);

    let admin = Address::generate(&env);

    client.initialize(&admin);

    // Verify contract is initialized
    let stats = client.get_stats();
    assert_eq!(stats.total_ders, 0);
    assert_eq!(stats.online_ders, 0);
    assert_eq!(stats.total_capacity, 0);
}

// ============ DER REGISTRATION TESTS ============

#[test]
fn test_register_der() {
    let env = Env::default();
    let contract_id = env.register(DistributedEnergyResourceManager, ());
    let client = DistributedEnergyResourceManagerClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let der_owner = Address::generate(&env);

    // Initialize contract
    client.initialize(&admin);

    // Register a solar DER
    let der_id = String::from_str(&env, "SOLAR_001");
    let location = String::from_str(&env, "San Francisco, CA");

    let result = client.register_der(
        &der_owner,
        &der_id,
        &ResourceType::Solar,
        &1000, // 1 MW
        &location
    );

    assert!(result);

    // Verify DER was registered
    let der_info = client.get_der_info(&der_id);
    assert_eq!(der_info.owner, der_owner);
    assert_eq!(der_info.resource_type, ResourceType::Solar);
    assert_eq!(der_info.capacity, 1000);
    assert_eq!(der_info.status, DERStatus::Online);
    assert_eq!(der_info.location, location);
}

#[test]
fn test_register_multiple_ders() {
    let env = Env::default();
    let contract_id = env.register(DistributedEnergyResourceManager, ());
    let client = DistributedEnergyResourceManagerClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let der_owner = Address::generate(&env);

    client.initialize(&admin);

    // Register multiple DERs
    let solar_der = String::from_str(&env, "SOLAR_001");
    let wind_der = String::from_str(&env, "WIND_001");
    let battery_der = String::from_str(&env, "BATTERY_001");

    let location = String::from_str(&env, "Test Location");

    client.register_der(&der_owner, &solar_der, &ResourceType::Solar, &1000, &location);
    client.register_der(&der_owner, &wind_der, &ResourceType::Wind, &500, &location);
    client.register_der(&der_owner, &battery_der, &ResourceType::Battery, &200, &location);

    // Verify all DERs are registered
    let stats = client.get_stats();
    assert_eq!(stats.total_ders, 3);
    assert_eq!(stats.online_ders, 3);
    assert_eq!(stats.total_capacity, 1700);

    // Verify owner can see all their DERs
    let owner_ders = client.get_owner_ders(&der_owner);
    assert_eq!(owner_ders.len(), 3);
}

#[test]
fn test_different_resource_types() {
    let env = Env::default();
    let contract_id = env.register(DistributedEnergyResourceManager, ());
    let client = DistributedEnergyResourceManagerClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let der_owner = Address::generate(&env);

    client.initialize(&admin);

    let location = String::from_str(&env, "Test Location");

    // Test all resource types
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

        let result = client.register_der(
            &der_owner,
            &der_id_str,
            &resource_type,
            &capacity,
            &location
        );
        assert!(result);

        // Verify DER was registered correctly
        let der_info = client.get_der_info(&der_id_str);
        assert_eq!(der_info.resource_type, resource_type);
        assert_eq!(der_info.capacity, capacity);
    }

    let stats = client.get_stats();
    assert_eq!(stats.total_ders, 6);
    assert_eq!(stats.total_capacity, 3200);
}

// ============ GRID OPERATOR TESTS ============

#[test]
fn test_add_grid_operator() {
    let env = Env::default();
    let contract_id = env.register(DistributedEnergyResourceManager, ());
    let client = DistributedEnergyResourceManagerClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let operator = Address::generate(&env);

    client.initialize(&admin);

    let operator_name = String::from_str(&env, "Grid Operator 1");

    let result = client.add_grid_operator(
        &admin,
        &operator,
        &operator_name,
        &5 // Authority level 5
    );

    assert!(result);
}

// ============ EDGE CASE TESTS ============

#[test]
#[should_panic(expected = "DER already exists")]
fn test_duplicate_der_registration() {
    let env = Env::default();
    let contract_id = env.register(DistributedEnergyResourceManager, ());
    let client = DistributedEnergyResourceManagerClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let der_owner = Address::generate(&env);

    client.initialize(&admin);

    let der_id = String::from_str(&env, "SOLAR_001");
    let location = String::from_str(&env, "Test Location");

    // Register DER first time
    client.register_der(&der_owner, &der_id, &ResourceType::Solar, &1000, &location);

    // Attempt to register same DER again - should panic
    client.register_der(&der_owner, &der_id, &ResourceType::Solar, &1000, &location);
}

#[test]
#[should_panic(expected = "Contract already initialized")]
fn test_duplicate_initialization() {
    let env = Env::default();
    let contract_id = env.register(DistributedEnergyResourceManager, ());
    let client = DistributedEnergyResourceManagerClient::new(&env, &contract_id);

    let admin = Address::generate(&env);

    // Initialize first time
    client.initialize(&admin);

    // Attempt to initialize again - should panic
    client.initialize(&admin);
}

#[test]
fn test_high_volume_der_registrations() {
    let env = Env::default();
    let contract_id = env.register(DistributedEnergyResourceManager, ());
    let client = DistributedEnergyResourceManagerClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let der_owner = Address::generate(&env);

    client.initialize(&admin);

    let location = String::from_str(&env, "Test Location");

    // Register 50 DERs to test scalability
    let der_ids = [
        "DER_000", "DER_001", "DER_002", "DER_003", "DER_004", "DER_005", "DER_006", "DER_007", "DER_008", "DER_009",
        "DER_010", "DER_011", "DER_012", "DER_013", "DER_014", "DER_015", "DER_016", "DER_017", "DER_018", "DER_019",
        "DER_020", "DER_021", "DER_022", "DER_023", "DER_024", "DER_025", "DER_026", "DER_027", "DER_028", "DER_029",
        "DER_030", "DER_031", "DER_032", "DER_033", "DER_034", "DER_035", "DER_036", "DER_037", "DER_038", "DER_039",
        "DER_040", "DER_041", "DER_042", "DER_043", "DER_044", "DER_045", "DER_046", "DER_047", "DER_048", "DER_049",
    ];

    for der_id_str in der_ids {
        let der_id = String::from_str(&env, der_id_str);
        client.register_der(&der_owner, &der_id, &ResourceType::Solar, &100, &location);
    }

    // Verify all DERs registered
    let stats = client.get_stats();
    assert_eq!(stats.total_ders, 50);
    assert_eq!(stats.online_ders, 50);
    assert_eq!(stats.total_capacity, 5000);

    // Verify owner can see all their DERs
    let owner_ders = client.get_owner_ders(&der_owner);
    assert_eq!(owner_ders.len(), 50);
}

