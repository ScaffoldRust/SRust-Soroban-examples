#![cfg(test)]

use super::utils::*;
use crate::*;
use soroban_sdk::{String, Vec};

// ============ FUND RELEASE TESTS ============

#[test]
fn test_funds_release() {
    let ctx = setup_initialized();

    let investor = create_investor(&ctx.env);
    let project_manager = create_project_manager(&ctx.env);
    let total_funding = 20_000_000_000i128;

    ctx.env.mock_all_auths();

    let project_id = ctx.client.initialize_project(
        &investor,
        &project_manager,
        &String::from_str(&ctx.env, "Funding Test Project"),
        &String::from_str(&ctx.env, "Testing fund release"),
        &total_funding,
        &2u32,
        &String::from_str(&ctx.env, "hydro"),
        &5_000_000i128,
    );

    let mut required_verifications = Vec::new(&ctx.env);
    required_verifications.push_back(String::from_str(&ctx.env, "performance_metrics"));

    let milestone_id = ctx.client.create_milestone(
        &project_id,
        &project_manager,
        &String::from_str(&ctx.env, "Performance Milestone"),
        &String::from_str(&ctx.env, "Achieve performance targets"),
        &40u32,
        &required_verifications,
        &(ctx.env.ledger().timestamp() + 86400 * 90),
        &Some(2_000_000i128),
        &Some(500i128),
    );

    ctx.client.start_milestone(&project_id, &milestone_id, &project_manager);

    ctx.client.verify_milestone(
        &project_id,
        &milestone_id,
        &project_manager,
        &String::from_str(&ctx.env, "performance_metrics"),
        &String::from_str(&ctx.env, "Performance targets achieved"),
    );

    let initial_available = ctx.client.get_available_funds(&project_id);
    assert_eq!(initial_available, total_funding);

    // Release funds
    ctx.client.release_funds(&project_id, &milestone_id, &investor);

    let project = get_project(&ctx, project_id);
    let expected_released = (total_funding * 40) / 100; // 40% of total funding
    assert_eq!(project.released_funding, expected_released);

    let remaining_available = ctx.client.get_available_funds(&project_id);
    assert_eq!(remaining_available, total_funding - expected_released);
}

// ============ REFUND TESTS ============

#[test]
fn test_refund_request_and_processing() {
    let ctx = setup_initialized();

    let investor = create_investor(&ctx.env);
    let project_manager = create_project_manager(&ctx.env);
    let total_funding = 15_000_000_000i128;

    ctx.env.mock_all_auths();

    let project_id = ctx.client.initialize_project(
        &investor,
        &project_manager,
        &String::from_str(&ctx.env, "Refund Test Project"),
        &String::from_str(&ctx.env, "Testing refund process"),
        &total_funding,
        &3u32,
        &String::from_str(&ctx.env, "geothermal"),
        &8_000_000i128,
    );

    let refund_amount = 5_000_000_000i128;
    let reason = String::from_str(&ctx.env, "Project delays due to permit issues");

    ctx.client.request_refund(&project_id, &investor, &refund_amount, &reason);

    // Process the refund
    ctx.client.process_refund(&project_id, &project_manager);

    let project = get_project(&ctx, project_id);
    assert_eq!(project.status, ProjectStatus::Refunded);
    assert_eq!(project.escrow_status, EscrowStatus::Refunded);
}

// ============ EDGE CASE TESTS ============

#[test]
#[should_panic(expected = "Milestone not completed")]
fn test_release_without_completed_milestone() {
    let ctx = setup_initialized();

    let investor = create_investor(&ctx.env);
    let project_manager = create_project_manager(&ctx.env);

    let project_id = create_basic_project(&ctx, &investor, &project_manager);

    let mut required_verifications = Vec::new(&ctx.env);
    required_verifications.push_back(String::from_str(&ctx.env, "technical_specification"));

    ctx.env.mock_all_auths();

    let milestone_id = ctx.client.create_milestone(
        &project_id,
        &project_manager,
        &String::from_str(&ctx.env, "Incomplete Milestone"),
        &String::from_str(&ctx.env, "Not completed"),
        &50u32,
        &required_verifications,
        &(ctx.env.ledger().timestamp() + 86400 * 60),
        &None,
        &None,
    );

    // Try to release funds without completing milestone
    ctx.client.release_funds(&project_id, &milestone_id, &investor);
}

#[test]
fn test_refund_workflow() {
    let ctx = setup_initialized();

    let investor = create_investor(&ctx.env);
    let project_manager = create_project_manager(&ctx.env);

    let project_id = create_basic_project(&ctx, &investor, &project_manager);

    ctx.env.mock_all_auths();

    // Create a refund request
    ctx.client.request_refund(
        &project_id,
        &investor,
        &5_000_000_000i128,
        &String::from_str(&ctx.env, "Project delay"),
    );

    // Process refund
    ctx.client.process_refund(&project_id, &project_manager);

    let project = get_project(&ctx, project_id);
    assert_eq!(project.status, ProjectStatus::Refunded);
}

#[test]
fn test_refund_for_failed_milestone() {
    let ctx = setup_initialized();

    let investor = create_investor(&ctx.env);
    let project_manager = create_project_manager(&ctx.env);

    let project_id = create_basic_project(&ctx, &investor, &project_manager);

    let mut required_verifications = Vec::new(&ctx.env);
    required_verifications.push_back(String::from_str(&ctx.env, "technical_specification"));

    ctx.env.mock_all_auths();

    let milestone_id = ctx.client.create_milestone(
        &project_id,
        &project_manager,
        &String::from_str(&ctx.env, "Failed Milestone"),
        &String::from_str(&ctx.env, "Will fail"),
        &30u32,
        &required_verifications,
        &(ctx.env.ledger().timestamp() + 86400 * 30),
        &None,
        &None,
    );

    ctx.client.start_milestone(&project_id, &milestone_id, &project_manager);

    // Mark milestone as failed
    ctx.client.fail_milestone(
        &project_id,
        &milestone_id,
        &project_manager,
        &String::from_str(&ctx.env, "Technical issues"),
    );

    let milestone = get_milestone(&ctx, project_id, milestone_id);
    assert_eq!(milestone.status, MilestoneStatus::Failed);

    // Request refund after milestone failure
    ctx.client.request_refund(
        &project_id,
        &investor,
        &3_000_000_000i128,
        &String::from_str(&ctx.env, "Milestone failed"),
    );

    // Process refund
    ctx.client.process_refund(&project_id, &project_manager);

    let project = get_project(&ctx, project_id);
    assert_eq!(project.status, ProjectStatus::Refunded);
}
