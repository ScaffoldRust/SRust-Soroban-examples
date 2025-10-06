#![cfg(test)]

use super::utils::*;
use crate::*;
use soroban_sdk::{String, Vec};

// ============ MILESTONE CREATION TESTS ============

#[test]
fn test_milestone_creation() {
    let ctx = setup_initialized();

    let investor = create_investor(&ctx.env);
    let project_manager = create_project_manager(&ctx.env);

    let project_id = create_basic_project(&ctx, &investor, &project_manager);

    let milestone_name = String::from_str(&ctx.env, "Site Preparation");
    let milestone_desc = String::from_str(&ctx.env, "Prepare construction site");
    let funding_percentage = 30u32;
    let due_date = ctx.env.ledger().timestamp() + 86400 * 30; // 30 days from now

    let mut required_verifications = Vec::new(&ctx.env);
    required_verifications.push_back(String::from_str(&ctx.env, "environmental_impact"));
    required_verifications.push_back(String::from_str(&ctx.env, "safety_compliance"));

    ctx.env.mock_all_auths();

    let milestone_id = ctx.client.create_milestone(
        &project_id,
        &project_manager,
        &milestone_name,
        &milestone_desc,
        &funding_percentage,
        &required_verifications,
        &due_date,
        &Some(3_000_000i128),
        &Some(1_000i128),
    );

    assert_eq!(milestone_id, 1);

    let milestone = get_milestone(&ctx, project_id, milestone_id);
    assert_eq!(milestone.funding_percentage, funding_percentage);
    assert_eq!(milestone.status, MilestoneStatus::Pending);
}

// ============ MILESTONE LIFECYCLE TESTS ============

#[test]
fn test_milestone_verification_and_completion() {
    let ctx = setup_initialized();

    let investor = create_investor(&ctx.env);
    let project_manager = create_project_manager(&ctx.env);
    ctx.env.mock_all_auths();

    let project_id = ctx.client.initialize_project(
        &investor,
        &project_manager,
        &String::from_str(&ctx.env, "Test Project"),
        &String::from_str(&ctx.env, "Test Description"),
        &10_000_000_000i128,
        &2u32,
        &String::from_str(&ctx.env, "wind"),
        &20_000_000i128,
    );

    let mut required_verifications = Vec::new(&ctx.env);
    required_verifications.push_back(String::from_str(&ctx.env, "technical_specification"));

    let milestone_id = ctx.client.create_milestone(
        &project_id,
        &project_manager,
        &String::from_str(&ctx.env, "First Milestone"),
        &String::from_str(&ctx.env, "Complete initial phase"),
        &50u32,
        &required_verifications,
        &(ctx.env.ledger().timestamp() + 86400 * 60),
        &None,
        &None,
    );

    // Start the milestone
    ctx.client
        .start_milestone(&project_id, &milestone_id, &project_manager);

    let milestone = get_milestone(&ctx, project_id, milestone_id);
    assert_eq!(milestone.status, MilestoneStatus::InProgress);

    // Verify the milestone
    ctx.client.verify_milestone(
        &project_id,
        &milestone_id,
        &project_manager,
        &String::from_str(&ctx.env, "technical_specification"),
        &String::from_str(&ctx.env, "Specification approved"),
    );

    let completed_milestone = get_milestone(&ctx, project_id, milestone_id);
    assert_eq!(completed_milestone.status, MilestoneStatus::Completed);

    // Check pending release
    let pending_release = ctx.client.get_pending_release(&project_id);
    assert!(pending_release.is_some());
}

// ============ UNAUTHORIZED TESTS ============

#[test]
#[should_panic(expected = "Unauthorized")]
fn test_unauthorized_milestone_verification() {
    let ctx = setup_initialized();

    let investor = create_investor(&ctx.env);
    let project_manager = create_project_manager(&ctx.env);
    let unauthorized_user = create_investor(&ctx.env); // Different user

    let project_id = create_basic_project(&ctx, &investor, &project_manager);

    let mut required_verifications = Vec::new(&ctx.env);
    required_verifications.push_back(String::from_str(&ctx.env, "technical_specification"));

    ctx.env.mock_all_auths();

    let milestone_id = ctx.client.create_milestone(
        &project_id,
        &project_manager,
        &String::from_str(&ctx.env, "Test Milestone"),
        &String::from_str(&ctx.env, "Description"),
        &50u32,
        &required_verifications,
        &(ctx.env.ledger().timestamp() + 86400 * 60),
        &None,
        &None,
    );

    ctx.client
        .start_milestone(&project_id, &milestone_id, &project_manager);

    // Unauthorized user tries to verify
    ctx.client.verify_milestone(
        &project_id,
        &milestone_id,
        &unauthorized_user,
        &String::from_str(&ctx.env, "technical_specification"),
        &String::from_str(&ctx.env, "Unauthorized verification"),
    );
}

#[test]
#[should_panic(expected = "Invalid milestone")]
fn test_verify_invalid_milestone() {
    let ctx = setup_initialized();

    let investor = create_investor(&ctx.env);
    let project_manager = create_project_manager(&ctx.env);

    let project_id = create_basic_project(&ctx, &investor, &project_manager);

    ctx.env.mock_all_auths();

    // Try to verify non-existent milestone
    ctx.client.verify_milestone(
        &project_id,
        &999u32,
        &project_manager,
        &String::from_str(&ctx.env, "technical_specification"),
        &String::from_str(&ctx.env, "Invalid milestone"),
    );
}
