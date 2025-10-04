#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _},
    Address, Env, String,
};

use crate::{
    DistributedEnergyResourceManager, DistributedEnergyResourceManagerClient, ResourceType,
    DERStatus
};

// ============ STATUS UPDATE TESTS ============

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

// ============ EDGE CASE TESTS ============

#[test]
#[should_panic(expected = "DER not found")]
fn test_update_status_non_existent_der() {
    let env = Env::default();
    let contract_id = env.register(DistributedEnergyResourceManager, ());
    let client = DistributedEnergyResourceManagerClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let der_owner = Address::generate(&env);

    client.initialize(&admin);

    let der_id = String::from_str(&env, "NON_EXISTENT_DER");

    // Attempt to update status of non-existent DER - should panic
    client.update_status(&der_owner, &der_id, &DERStatus::Maintenance);
}

#[test]
#[should_panic(expected = "DER not found")]
fn test_get_info_non_existent_der() {
    let env = Env::default();
    let contract_id = env.register(DistributedEnergyResourceManager, ());
    let client = DistributedEnergyResourceManagerClient::new(&env, &contract_id);

    let admin = Address::generate(&env);

    client.initialize(&admin);

    let der_id = String::from_str(&env, "NON_EXISTENT_DER");

    // Attempt to get info of non-existent DER - should panic
    client.get_der_info(&der_id);
}

#[test]
fn test_status_transitions_all_states() {
    let env = Env::default();
    let contract_id = env.register(DistributedEnergyResourceManager, ());
    let client = DistributedEnergyResourceManagerClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let der_owner = Address::generate(&env);

    client.initialize(&admin);

    let der_id = String::from_str(&env, "SOLAR_001");
    let location = String::from_str(&env, "Test Location");

    client.register_der(&der_owner, &der_id, &ResourceType::Solar, &1000, &location);

    // Test all status transitions
    let statuses = [
        DERStatus::Online,
        DERStatus::Maintenance,
        DERStatus::Offline,
        DERStatus::Online,
        DERStatus::Optimized,
    ];

    for status in statuses {
        client.update_status(&der_owner, &der_id, &status);
        let der_info = client.get_der_info(&der_id);
        assert_eq!(der_info.status, status);
    }
}

#[test]
fn test_multiple_ders_different_statuses() {
    let env = Env::default();
    let contract_id = env.register(DistributedEnergyResourceManager, ());
    let client = DistributedEnergyResourceManagerClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let der_owner = Address::generate(&env);

    client.initialize(&admin);

    let location = String::from_str(&env, "Test Location");

    // Register 10 DERs and set different statuses
    let der_configs = [
        ("DER_0", DERStatus::Online),
        ("DER_1", DERStatus::Maintenance),
        ("DER_2", DERStatus::Offline),
        ("DER_3", DERStatus::Optimized),
        ("DER_4", DERStatus::Online),
        ("DER_5", DERStatus::Maintenance),
        ("DER_6", DERStatus::Offline),
        ("DER_7", DERStatus::Optimized),
        ("DER_8", DERStatus::Online),
        ("DER_9", DERStatus::Maintenance),
    ];

    for (der_id_str, status) in der_configs {
        let der_id = String::from_str(&env, der_id_str);
        client.register_der(&der_owner, &der_id, &ResourceType::Solar, &100, &location);

        if status != DERStatus::Online {
            client.update_status(&der_owner, &der_id, &status);
        }
    }

    // Verify stats reflect mixed statuses
    let stats = client.get_stats();
    assert_eq!(stats.total_ders, 10);
    // Online (3) + Optimized (2) = 5
    assert_eq!(stats.online_ders, 5);
    assert_eq!(stats.total_capacity, 1000);
}

