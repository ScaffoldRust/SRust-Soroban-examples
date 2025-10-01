#![cfg(test)]

use super::utils::*;
use crate::*;
use soroban_sdk::String;

// ============ CONTRACT INITIALIZATION TESTS ============

#[test]
fn test_contract_initialization() {
    let ctx = setup_test();

    ctx.client.initialize();

    // Test that next project ID is set to 1
    let next_id = get_next_project_id(&ctx);
    assert_eq!(next_id, 1);
}

// ============ PROJECT INITIALIZATION TESTS ============

#[test]
fn test_project_initialization() {
    let ctx = setup_initialized();

    let investor = create_investor(&ctx.env);
    let project_manager = create_project_manager(&ctx.env);
    let name = String::from_str(&ctx.env, "Solar Farm Alpha");
    let description = String::from_str(&ctx.env, "100MW solar installation");
    let total_funding = 500_000_000_000i128; // 500K XLM (within limits)
    let milestone_count = 5u32;
    let energy_type = String::from_str(&ctx.env, "solar");
    let expected_capacity = 100_000_000i128; // 100MW

    ctx.env.mock_all_auths();

    let project_id = ctx.client.initialize_project(
        &investor,
        &project_manager,
        &name,
        &description,
        &total_funding,
        &milestone_count,
        &energy_type,
        &expected_capacity,
    );

    assert_eq!(project_id, 1);

    let project = get_project(&ctx, project_id);
    assert_eq!(project.investor, investor);
    assert_eq!(project.project_manager, project_manager);
    assert_eq!(project.total_funding, total_funding);
    assert_eq!(project.milestone_count, milestone_count);
    assert_eq!(project.status, ProjectStatus::Active);
}

// ============ VALIDATION TESTS ============

#[test]
fn test_validation_functions() {
    let ctx = setup_test();

    let investor = create_investor(&ctx.env);
    let project_manager = create_project_manager(&ctx.env);

    // Test valid project validation
    let is_valid = ctx.client.validate_project(
        &investor,
        &project_manager,
        &10_000_000_000i128,
        &5u32,
        &String::from_str(&ctx.env, "solar"),
        &50_000_000i128,
    );
    assert!(is_valid);

    // Test milestone funding calculation
    let total_funding = 100_000_000_000i128;
    let percentage = 25u32;
    let expected_amount = ctx.client.calculate_milestone_funding(&total_funding, &percentage);
    assert_eq!(expected_amount, 25_000_000_000i128);
}

// ============ DEPOSIT TESTS ============

#[test]
fn test_deposit_funds() {
    let ctx = setup_initialized();

    let investor = create_investor(&ctx.env);
    let project_manager = create_project_manager(&ctx.env);

    let project_id = create_basic_project(&ctx, &investor, &project_manager);

    ctx.env.mock_all_auths();

    let deposit_amount = 5_000_000_000i128;
    ctx.client.deposit(&project_id, &investor, &deposit_amount);

    let project = get_project(&ctx, project_id);
    let expected_total = 10_000_000_000i128 + deposit_amount; // original + deposit
    assert_eq!(project.total_funding, expected_total);
}

#[test]
#[should_panic(expected = "Project not found")]
fn test_deposit_non_existent_project() {
    let ctx = setup_initialized();
    let investor = create_investor(&ctx.env);

    ctx.env.mock_all_auths();

    // Try to deposit to non-existent project
    ctx.client.deposit(&999u64, &investor, &1_000_000_000i128);
}

#[test]
fn test_multiple_deposits() {
    let ctx = setup_initialized();

    let investor = create_investor(&ctx.env);
    let project_manager = create_project_manager(&ctx.env);

    let project_id = create_basic_project(&ctx, &investor, &project_manager);

    ctx.env.mock_all_auths();

    // First deposit
    ctx.client.deposit(&project_id, &investor, &2_000_000_000i128);

    // Second deposit (allowed)
    ctx.client.deposit(&project_id, &investor, &3_000_000_000i128);

    let project = get_project(&ctx, project_id);
    let expected_total = 10_000_000_000i128 + 2_000_000_000i128 + 3_000_000_000i128;
    assert_eq!(project.total_funding, expected_total);
}

// ============ NEGATIVE TESTS ============

#[test]
#[should_panic(expected = "Invalid project setup parameters")]
fn test_invalid_project_setup() {
    let ctx = setup_initialized();

    let same_address = create_investor(&ctx.env);

    ctx.env.mock_all_auths();

    // Should panic because investor and project manager are the same
    ctx.client.initialize_project(
        &same_address,
        &same_address,
        &String::from_str(&ctx.env, "Invalid Project"),
        &String::from_str(&ctx.env, "This should fail"),
        &10_000_000_000i128,
        &3u32,
        &String::from_str(&ctx.env, "solar"),
        &10_000_000i128,
    );
}

#[test]
#[should_panic(expected = "Invalid project setup parameters")]
fn test_invalid_funding_amount() {
    let ctx = setup_initialized();

    let investor = create_investor(&ctx.env);
    let project_manager = create_project_manager(&ctx.env);

    ctx.env.mock_all_auths();

    // Try to create project with zero funding (caught by validation)
    ctx.client.initialize_project(
        &investor,
        &project_manager,
        &String::from_str(&ctx.env, "Zero Funding Project"),
        &String::from_str(&ctx.env, "Should fail"),
        &0i128,
        &3u32,
        &String::from_str(&ctx.env, "solar"),
        &10_000_000i128,
    );
}

// ============ SCALABILITY TESTS ============

#[test]
fn test_high_volume_deposits() {
    let ctx = setup_initialized();

    let investor = create_investor(&ctx.env);
    let project_manager = create_project_manager(&ctx.env);

    let project_id = create_basic_project(&ctx, &investor, &project_manager);

    ctx.env.mock_all_auths();

    let num_deposits = 20;
    let deposit_amount = 1_000_000_000i128;

    // Simulate multiple deposits
    for _ in 0..num_deposits {
        ctx.client.deposit(&project_id, &investor, &deposit_amount);
    }

    let project = get_project(&ctx, project_id);
    let expected_total = 10_000_000_000i128 + (deposit_amount * num_deposits);
    assert_eq!(project.total_funding, expected_total);
}
