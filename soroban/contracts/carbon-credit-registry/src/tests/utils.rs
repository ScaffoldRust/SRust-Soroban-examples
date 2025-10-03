#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, Ledger as _},
    Address, BytesN, Env, Map, String, Vec,
};

use crate::{
    CarbonCreditRegistry, CarbonCreditRegistryClient, CreditStatus,
    RetirementParams, TradingParams,
};

// ============ TEST CONTEXT ============

pub struct TestContext {
    pub env: Env,
    pub client: CarbonCreditRegistryClient<'static>,
    pub admin: Address,
}

// ============ SETUP FUNCTIONS ============

/// Creates a basic test environment with initialized contract
#[allow(dead_code)]
pub fn setup_test_env() -> TestContext {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, CarbonCreditRegistry);
    let client = CarbonCreditRegistryClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    client.initialize(&admin, &25, &10); // 0.25% trading fee, 0.10% retirement fee

    TestContext { env, client, admin }
}

/// Creates test environment with registered issuer
#[allow(dead_code)]
pub fn setup_with_issuer() -> (TestContext, Address) {
    let ctx = setup_test_env();
    let issuer = Address::generate(&ctx.env);

    let issuer_name = String::from_str(&ctx.env, "Test Issuer");
    let mut standards = Vec::new(&ctx.env);
    standards.push_back(String::from_str(&ctx.env, "Verra"));
    standards.push_back(String::from_str(&ctx.env, "Gold Standard"));

    ctx.client.register_issuer(&issuer, &issuer_name, &standards);

    (ctx, issuer)
}

// ============ HELPER FUNCTIONS ============

/// Issues a basic carbon credit for testing
/// Note: Uses incrementing timestamp to ensure unique credit IDs
#[allow(dead_code)]
pub fn issue_test_credit(
    ctx: &TestContext,
    issuer: &Address,
    quantity: i128,
) -> BytesN<32> {
    let project_type = String::from_str(&ctx.env, "Reforestation");
    let project_location = String::from_str(&ctx.env, "Brazil");
    let verification_standard = String::from_str(&ctx.env, "Verra");

    // Create unique verification hash using current timestamp
    let timestamp = ctx.env.ledger().timestamp();
    let mut hash_bytes = [1u8; 32];
    let ts_bytes = timestamp.to_be_bytes();
    hash_bytes[0..8].copy_from_slice(&ts_bytes);
    let verification_hash = BytesN::from_array(&ctx.env, &hash_bytes);

    let metadata = Map::new(&ctx.env);

    let credit_id = ctx.client.issue_credit(
        issuer,
        &project_type,
        &project_location,
        &verification_standard,
        &2024,
        &quantity,
        &verification_hash,
        &metadata,
    );

    // Increment timestamp to ensure next credit gets different ID
    ctx.env.ledger().with_mut(|li| li.timestamp += 1);

    credit_id
}

/// Issues a credit with custom parameters
/// Note: Uses incrementing timestamp to ensure unique credit IDs
#[allow(dead_code)]
pub fn issue_credit_with_params(
    ctx: &TestContext,
    issuer: &Address,
    project_type: &str,
    location: &str,
    standard: &str,
    vintage: u32,
    quantity: i128,
) -> BytesN<32> {
    let project_type_str = String::from_str(&ctx.env, project_type);
    let location_str = String::from_str(&ctx.env, location);
    let standard_str = String::from_str(&ctx.env, standard);

    // Create unique verification hash using current timestamp
    let timestamp = ctx.env.ledger().timestamp();
    let mut hash_bytes = [2u8; 32];
    let ts_bytes = timestamp.to_be_bytes();
    hash_bytes[0..8].copy_from_slice(&ts_bytes);
    let verification_hash = BytesN::from_array(&ctx.env, &hash_bytes);

    let metadata = Map::new(&ctx.env);

    let credit_id = ctx.client.issue_credit(
        issuer,
        &project_type_str,
        &location_str,
        &standard_str,
        &vintage,
        &quantity,
        &verification_hash,
        &metadata,
    );

    // Increment timestamp to ensure next credit gets different ID
    ctx.env.ledger().with_mut(|li| li.timestamp += 1);

    credit_id
}

/// Trades a credit from one owner to another
#[allow(dead_code)]
pub fn trade_test_credit(
    ctx: &TestContext,
    credit_id: BytesN<32>,
    from: &Address,
    to: &Address,
    quantity: i128,
) {
    let payment_token = Address::generate(&ctx.env);

    let params = TradingParams {
        credit_id,
        from: from.clone(),
        to: to.clone(),
        quantity,
        price: 50,
        payment_token,
    };

    ctx.client.trade_credit(&params);
}

/// Retires a credit
#[allow(dead_code)]
pub fn retire_test_credit(
    ctx: &TestContext,
    credit_id: BytesN<32>,
    owner: &Address,
    quantity: i128,
) {
    let retirement_reason = String::from_str(&ctx.env, "Carbon offset claim");
    let retirement_certificate = BytesN::from_array(&ctx.env, &[3u8; 32]);

    let params = RetirementParams {
        credit_id,
        owner: owner.clone(),
        quantity,
        retirement_reason,
        retirement_certificate,
    };

    ctx.client.retire_credit(&params);
}

// ============ ASSERTION HELPERS ============

/// Asserts credit has expected status
#[allow(dead_code)]
pub fn assert_credit_status(ctx: &TestContext, credit_id: &BytesN<32>, expected: CreditStatus) {
    let credit = ctx.client.get_credit_status(credit_id);
    assert!(credit.is_some(), "Credit should exist");
    assert_eq!(
        credit.unwrap().status,
        expected,
        "Credit status should match expected"
    );
}

/// Asserts credit has expected owner
#[allow(dead_code)]
pub fn assert_credit_owner(ctx: &TestContext, credit_id: &BytesN<32>, expected_owner: &Address) {
    let credit = ctx.client.get_credit_status(credit_id);
    assert!(credit.is_some(), "Credit should exist");
    assert_eq!(
        credit.unwrap().current_owner,
        expected_owner.clone(),
        "Credit owner should match expected"
    );
}

/// Asserts credit has expected quantity
#[allow(dead_code)]
pub fn assert_credit_quantity(ctx: &TestContext, credit_id: &BytesN<32>, expected: i128) {
    let credit = ctx.client.get_credit_status(credit_id);
    assert!(credit.is_some(), "Credit should exist");
    assert_eq!(
        credit.unwrap().quantity,
        expected,
        "Credit quantity should match expected"
    );
}

/// Asserts contract stats match expected values
#[allow(dead_code)]
pub fn assert_contract_stats(
    ctx: &TestContext,
    expected_issuer_count: u32,
    expected_credit_count: u32,
    expected_total_issued: i128,
    expected_total_retired: i128,
) {
    let (issuer_count, credit_count, total_issued, total_retired) = ctx.client.get_contract_stats();

    assert_eq!(issuer_count, expected_issuer_count, "Issuer count mismatch");
    assert_eq!(credit_count, expected_credit_count, "Credit count mismatch");
    assert_eq!(total_issued, expected_total_issued, "Total issued mismatch");
    assert_eq!(total_retired, expected_total_retired, "Total retired mismatch");
}
