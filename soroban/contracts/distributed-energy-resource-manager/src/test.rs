#[cfg(test)]
mod test {
    use soroban_sdk::{
        testutils::{Address as _},
        Address, Env, String,
    };

    use crate::{
        DistributedEnergyResourceManager, DistributedEnergyResourceManagerClient, ResourceType, 
        DERStatus
    };

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
fn test_update_status() {
    let env = Env::default();
    let contract_id = env.register(DistributedEnergyResourceManager, ());
    let client = DistributedEnergyResourceManagerClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let der_owner = Address::generate(&env);
    
    client.initialize(&admin);
    
    let der_id = String::from_str(&env, "SOLAR_001");
    let location = String::from_str(&env, "Test Location");
    
    client.register_der(&der_owner, &der_id, &ResourceType::Solar, &1000, &location);
    
    // Update status to maintenance
    let result = client.update_status(&der_owner, &der_id, &DERStatus::Maintenance);
    assert!(result);
    
    // Verify status was updated
    let der_info = client.get_der_info(&der_id);
    assert_eq!(der_info.status, DERStatus::Maintenance);
}

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

#[test]
fn test_get_stats() {
    let env = Env::default();
    let contract_id = env.register(DistributedEnergyResourceManager, ());
    let client = DistributedEnergyResourceManagerClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let der_owner = Address::generate(&env);
    
    client.initialize(&admin);
    
    // Register multiple DERs with different statuses
    let solar_der = String::from_str(&env, "SOLAR_001");
    let wind_der = String::from_str(&env, "WIND_001");
    let battery_der = String::from_str(&env, "BATTERY_001");
    let location = String::from_str(&env, "Test Location");
    
    // Register solar DER
    client.register_der(&der_owner, &solar_der, &ResourceType::Solar, &1000, &location);
    
    // Register wind DER
    client.register_der(&der_owner, &wind_der, &ResourceType::Wind, &500, &location);
    
    // Register battery DER
    client.register_der(&der_owner, &battery_der, &ResourceType::Battery, &200, &location);
    
    // Put one DER in maintenance
    client.update_status(&der_owner, &battery_der, &DERStatus::Maintenance);
    
    let stats = client.get_stats();
    assert_eq!(stats.total_ders, 3);
    assert_eq!(stats.online_ders, 2); // Only solar and wind are online
    assert_eq!(stats.total_capacity, 1700);
    assert_eq!(stats.utilization_rate, 66); // 2 out of 3 DERs online
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
}