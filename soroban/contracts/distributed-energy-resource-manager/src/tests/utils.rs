#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _},
    Address, Env, String,
};

use crate::{
    DistributedEnergyResourceManager, DistributedEnergyResourceManagerClient, ResourceType,
    DERStatus
};

// ============ TEST CONTEXT ============

pub struct TestContext {
    pub env: Env,
    pub client: DistributedEnergyResourceManagerClient<'static>,
    pub admin: Address,
}

// ============ SETUP FUNCTIONS ============

/// Creates a basic test environment with initialized contract
#[allow(dead_code)]
pub fn setup_test_env() -> TestContext {
    let env = Env::default();
    let contract_id = env.register(DistributedEnergyResourceManager, ());
    let client = DistributedEnergyResourceManagerClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    client.initialize(&admin);

    TestContext {
        env,
        client,
        admin,
    }
}

/// Creates test environment with admin and grid operator
#[allow(dead_code)]
pub fn setup_with_operator() -> (TestContext, Address) {
    let ctx = setup_test_env();
    let operator = Address::generate(&ctx.env);
    let operator_name = String::from_str(&ctx.env, "Grid Operator");

    ctx.client.add_grid_operator(&ctx.admin, &operator, &operator_name, &5);

    (ctx, operator)
}

// ============ HELPER FUNCTIONS ============

/// Registers a basic DER for testing
#[allow(dead_code)]
pub fn register_test_der(
    ctx: &TestContext,
    owner: &Address,
    der_id: &str,
    resource_type: ResourceType,
    capacity: u32,
) -> String {
    let der_id_str = String::from_str(&ctx.env, der_id);
    let location = String::from_str(&ctx.env, "Test Location");

    ctx.client.register_der(owner, &der_id_str, &resource_type, &capacity, &location);
    der_id_str
}

/// Registers multiple DERs for testing (up to 10)
#[allow(dead_code)]
pub fn register_multiple_ders(
    ctx: &TestContext,
    owner: &Address,
    count: u32,
) {
    let location = String::from_str(&ctx.env, "Test Location");
    let der_ids = ["DER_00", "DER_01", "DER_02", "DER_03", "DER_04",
                   "DER_05", "DER_06", "DER_07", "DER_08", "DER_09"];

    for i in 0..(count.min(10) as usize) {
        let der_id = String::from_str(&ctx.env, der_ids[i]);
        ctx.client.register_der(owner, &der_id, &ResourceType::Solar, &100, &location);
    }
}

// ============ ASSERTION HELPERS ============

/// Asserts DER has expected status
#[allow(dead_code)]
pub fn assert_der_status(ctx: &TestContext, der_id: &String, expected: DERStatus) {
    let der_info = ctx.client.get_der_info(der_id);
    assert_eq!(der_info.status, expected, "DER status should match expected");
}

/// Asserts stats match expected values
#[allow(dead_code)]
pub fn assert_stats(
    ctx: &TestContext,
    total: u32,
    online: u32,
    capacity: u32,
) {
    let stats = ctx.client.get_stats();
    assert_eq!(stats.total_ders, total, "Total DERs mismatch");
    assert_eq!(stats.online_ders, online, "Online DERs mismatch");
    assert_eq!(stats.total_capacity, capacity, "Total capacity mismatch");
}
