#![cfg(test)]

use crate::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String, Vec};

// ============ TEST CONTEXT ============

pub struct TestContext {
    pub env: Env,
    pub contract_id: Address,
    pub client: EnergyProjectFundingEscrowClient<'static>,
}

// ============ SETUP FUNCTIONS ============

pub fn setup_test() -> TestContext {
    let env = Env::default();
    let contract_id = env.register(EnergyProjectFundingEscrow, ());
    let client = EnergyProjectFundingEscrowClient::new(&env, &contract_id);

    TestContext {
        env,
        contract_id,
        client,
    }
}

pub fn setup_initialized() -> TestContext {
    let ctx = setup_test();
    ctx.client.initialize();
    ctx
}

// ============ USER CREATION ============

pub fn create_investor(env: &Env) -> Address {
    Address::generate(env)
}

pub fn create_project_manager(env: &Env) -> Address {
    Address::generate(env)
}

// ============ PROJECT HELPERS ============

pub fn create_basic_project(ctx: &TestContext, investor: &Address, project_manager: &Address) -> u64 {
    ctx.env.mock_all_auths();

    ctx.client.initialize_project(
        investor,
        project_manager,
        &String::from_str(&ctx.env, "Test Project"),
        &String::from_str(&ctx.env, "Test Description"),
        &10_000_000_000i128,
        &3u32,
        &String::from_str(&ctx.env, "solar"),
        &10_000_000i128,
    )
}

pub fn create_project_with_params(
    ctx: &TestContext,
    investor: &Address,
    project_manager: &Address,
    name: &str,
    description: &str,
    total_funding: i128,
    milestone_count: u32,
    energy_type: &str,
    expected_capacity: i128,
) -> u64 {
    ctx.env.mock_all_auths();

    ctx.client.initialize_project(
        investor,
        project_manager,
        &String::from_str(&ctx.env, name),
        &String::from_str(&ctx.env, description),
        &total_funding,
        &milestone_count,
        &String::from_str(&ctx.env, energy_type),
        &expected_capacity,
    )
}

// ============ MILESTONE HELPERS ============

pub fn create_basic_milestone(
    ctx: &TestContext,
    project_id: u64,
    project_manager: &Address,
    funding_percentage: u32,
) -> u32 {
    ctx.env.mock_all_auths();

    let mut required_verifications = Vec::new(&ctx.env);
    required_verifications.push_back(String::from_str(&ctx.env, "technical_specification"));

    ctx.client.create_milestone(
        &project_id,
        project_manager,
        &String::from_str(&ctx.env, "Test Milestone"),
        &String::from_str(&ctx.env, "Test milestone description"),
        &funding_percentage,
        &required_verifications,
        &(ctx.env.ledger().timestamp() + 86400 * 60),
        &None,
        &None,
    )
}

pub fn create_milestone_with_verifications(
    ctx: &TestContext,
    project_id: u64,
    project_manager: &Address,
    milestone_name: &str,
    milestone_desc: &str,
    funding_percentage: u32,
    verifications: Vec<String>,
    due_date: u64,
) -> u32 {
    ctx.env.mock_all_auths();

    ctx.client.create_milestone(
        &project_id,
        project_manager,
        &String::from_str(&ctx.env, milestone_name),
        &String::from_str(&ctx.env, milestone_desc),
        &funding_percentage,
        &verifications,
        &due_date,
        &None,
        &None,
    )
}

// ============ MILESTONE ACTIONS ============

pub fn start_milestone(ctx: &TestContext, project_id: u64, milestone_id: u32, project_manager: &Address) {
    ctx.env.mock_all_auths();
    ctx.client.start_milestone(&project_id, &milestone_id, project_manager);
}

pub fn verify_milestone(
    ctx: &TestContext,
    project_id: u64,
    milestone_id: u32,
    project_manager: &Address,
    verification_type: &str,
    evidence: &str,
) {
    ctx.env.mock_all_auths();
    ctx.client.verify_milestone(
        &project_id,
        &milestone_id,
        project_manager,
        &String::from_str(&ctx.env, verification_type),
        &String::from_str(&ctx.env, evidence),
    );
}

// ============ FUND OPERATIONS ============

pub fn release_funds(ctx: &TestContext, project_id: u64, milestone_id: u32, investor: &Address) {
    ctx.env.mock_all_auths();
    ctx.client.release_funds(&project_id, &milestone_id, investor);
}

pub fn request_refund(ctx: &TestContext, project_id: u64, investor: &Address, amount: i128, reason: &str) {
    ctx.env.mock_all_auths();
    ctx.client.request_refund(
        &project_id,
        investor,
        &amount,
        &String::from_str(&ctx.env, reason),
    );
}

pub fn process_refund(ctx: &TestContext, project_id: u64, project_manager: &Address) {
    ctx.env.mock_all_auths();
    ctx.client.process_refund(&project_id, project_manager);
}

// ============ STORAGE ACCESS ============

pub fn get_project(ctx: &TestContext, project_id: u64) -> ProjectDetails {
    ctx.client.get_project(&project_id)
}

pub fn get_milestone(ctx: &TestContext, project_id: u64, milestone_id: u32) -> MilestoneDetails {
    ctx.client.get_milestone(&project_id, &milestone_id)
}


pub fn get_next_project_id(ctx: &TestContext) -> u64 {
    ctx.env.as_contract(&ctx.contract_id, || {
        ctx.env.storage().instance().get(&DataKey::NextProjectId).unwrap()
    })
}

// ============ ASSERTIONS ============

pub fn assert_project_status(ctx: &TestContext, project_id: u64, expected_status: ProjectStatus) {
    let project = get_project(ctx, project_id);
    assert_eq!(project.status, expected_status, "Project status should match");
}

pub fn assert_milestone_status(ctx: &TestContext, project_id: u64, milestone_id: u32, expected_status: MilestoneStatus) {
    let milestone = get_milestone(ctx, project_id, milestone_id);
    assert_eq!(milestone.status, expected_status, "Milestone status should match");
}

pub fn assert_escrow_status(ctx: &TestContext, project_id: u64, expected_status: EscrowStatus) {
    let project = get_project(ctx, project_id);
    assert_eq!(project.escrow_status, expected_status, "Escrow status should match");
}

// ============ WORKFLOW HELPERS ============

/// Complete milestone workflow: create → start → verify
pub fn complete_milestone_workflow(
    ctx: &TestContext,
    project_id: u64,
    project_manager: &Address,
    funding_percentage: u32,
) -> u32 {
    let milestone_id = create_basic_milestone(ctx, project_id, project_manager, funding_percentage);
    start_milestone(ctx, project_id, milestone_id, project_manager);
    verify_milestone(ctx, project_id, milestone_id, project_manager, "technical_specification", "Approved");
    milestone_id
}

/// Create project with completed milestone ready for fund release
pub fn setup_project_with_completed_milestone(
    ctx: &TestContext,
    investor: &Address,
    project_manager: &Address,
    total_funding: i128,
    funding_percentage: u32,
) -> (u64, u32) {
    let project_id = create_project_with_params(
        ctx,
        investor,
        project_manager,
        "Funded Project",
        "Project ready for funding",
        total_funding,
        2,
        "solar",
        10_000_000,
    );

    let milestone_id = complete_milestone_workflow(ctx, project_id, project_manager, funding_percentage);

    (project_id, milestone_id)
}
