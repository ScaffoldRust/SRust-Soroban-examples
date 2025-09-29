#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String, Vec};

#[test]
fn test_contract_initialization() {
    let env = Env::default();
    let contract_id = env.register(EnergyProjectFundingEscrow, ());
    let client = EnergyProjectFundingEscrowClient::new(&env, &contract_id);

    client.initialize();

    // Test that next project ID is set to 1
    let next_id: u64 = env.as_contract(&contract_id, || {
        env.storage().instance().get(&DataKey::NextProjectId).unwrap()
    });
    assert_eq!(next_id, 1);
}

#[test]
fn test_project_initialization() {
    let env = Env::default();
    let contract_id = env.register(EnergyProjectFundingEscrow, ());
    let client = EnergyProjectFundingEscrowClient::new(&env, &contract_id);

    client.initialize();

    let investor = Address::generate(&env);
    let project_manager = Address::generate(&env);
    let name = String::from_str(&env, "Solar Farm Alpha");
    let description = String::from_str(&env, "100MW solar installation");
    let total_funding = 500_000_000_000i128; // 500K XLM (within limits)
    let milestone_count = 5u32;
    let energy_type = String::from_str(&env, "solar");
    let expected_capacity = 100_000_000i128; // 100MW

    env.mock_all_auths();

    let project_id = client.initialize_project(
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

    let project = client.get_project(&project_id);
    assert_eq!(project.investor, investor);
    assert_eq!(project.project_manager, project_manager);
    assert_eq!(project.total_funding, total_funding);
    assert_eq!(project.milestone_count, milestone_count);
    assert_eq!(project.status, ProjectStatus::Active);
}

#[test]
fn test_milestone_creation() {
    let env = Env::default();
    let contract_id = env.register(EnergyProjectFundingEscrow, ());
    let client = EnergyProjectFundingEscrowClient::new(&env, &contract_id);

    client.initialize();

    let investor = Address::generate(&env);
    let project_manager = Address::generate(&env);
    env.mock_all_auths();

    let project_id = client.initialize_project(
        &investor,
        &project_manager,
        &String::from_str(&env, "Test Project"),
        &String::from_str(&env, "Test Description"),
        &10_000_000_000i128,
        &3u32,
        &String::from_str(&env, "solar"),
        &10_000_000i128,
    );

    let milestone_name = String::from_str(&env, "Site Preparation");
    let milestone_desc = String::from_str(&env, "Prepare construction site");
    let funding_percentage = 30u32;
    let due_date = env.ledger().timestamp() + 86400 * 30; // 30 days from now

    let mut required_verifications = Vec::new(&env);
    required_verifications.push_back(String::from_str(&env, "environmental_impact"));
    required_verifications.push_back(String::from_str(&env, "safety_compliance"));

    let milestone_id = client.create_milestone(
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

    let milestone = client.get_milestone(&project_id, &milestone_id);
    assert_eq!(milestone.funding_percentage, funding_percentage);
    assert_eq!(milestone.status, MilestoneStatus::Pending);
}

#[test]
fn test_milestone_verification_and_completion() {
    let env = Env::default();
    let contract_id = env.register(EnergyProjectFundingEscrow, ());
    let client = EnergyProjectFundingEscrowClient::new(&env, &contract_id);

    client.initialize();

    let investor = Address::generate(&env);
    let project_manager = Address::generate(&env);
    env.mock_all_auths();

    let project_id = client.initialize_project(
        &investor,
        &project_manager,
        &String::from_str(&env, "Test Project"),
        &String::from_str(&env, "Test Description"),
        &10_000_000_000i128,
        &2u32,
        &String::from_str(&env, "wind"),
        &20_000_000i128,
    );

    let mut required_verifications = Vec::new(&env);
    required_verifications.push_back(String::from_str(&env, "technical_specification"));

    let milestone_id = client.create_milestone(
        &project_id,
        &project_manager,
        &String::from_str(&env, "First Milestone"),
        &String::from_str(&env, "Complete initial phase"),
        &50u32,
        &required_verifications,
        &(env.ledger().timestamp() + 86400 * 60),
        &None,
        &None,
    );

    // Start the milestone
    client.start_milestone(&project_id, &milestone_id, &project_manager);

    let milestone = client.get_milestone(&project_id, &milestone_id);
    assert_eq!(milestone.status, MilestoneStatus::InProgress);

    // Verify the milestone
    client.verify_milestone(
        &project_id,
        &milestone_id,
        &project_manager,
        &String::from_str(&env, "technical_specification"),
        &String::from_str(&env, "Specification approved"),
    );

    let completed_milestone = client.get_milestone(&project_id, &milestone_id);
    assert_eq!(completed_milestone.status, MilestoneStatus::Completed);

    // Check pending release
    let pending_release = client.get_pending_release(&project_id);
    assert!(pending_release.is_some());
}

#[test]
fn test_funds_release() {
    let env = Env::default();
    let contract_id = env.register(EnergyProjectFundingEscrow, ());
    let client = EnergyProjectFundingEscrowClient::new(&env, &contract_id);

    client.initialize();

    let investor = Address::generate(&env);
    let project_manager = Address::generate(&env);
    let total_funding = 20_000_000_000i128;

    env.mock_all_auths();

    let project_id = client.initialize_project(
        &investor,
        &project_manager,
        &String::from_str(&env, "Funding Test Project"),
        &String::from_str(&env, "Testing fund release"),
        &total_funding,
        &2u32,
        &String::from_str(&env, "hydro"),
        &5_000_000i128,
    );

    let mut required_verifications = Vec::new(&env);
    required_verifications.push_back(String::from_str(&env, "performance_metrics"));

    let milestone_id = client.create_milestone(
        &project_id,
        &project_manager,
        &String::from_str(&env, "Performance Milestone"),
        &String::from_str(&env, "Achieve performance targets"),
        &40u32,
        &required_verifications,
        &(env.ledger().timestamp() + 86400 * 90),
        &Some(2_000_000i128),
        &Some(500i128),
    );

    client.start_milestone(&project_id, &milestone_id, &project_manager);

    client.verify_milestone(
        &project_id,
        &milestone_id,
        &project_manager,
        &String::from_str(&env, "performance_metrics"),
        &String::from_str(&env, "Performance targets achieved"),
    );

    let initial_available = client.get_available_funds(&project_id);
    assert_eq!(initial_available, total_funding);

    // Release funds
    client.release_funds(&project_id, &milestone_id, &investor);

    let project = client.get_project(&project_id);
    let expected_released = (total_funding * 40) / 100; // 40% of total funding
    assert_eq!(project.released_funding, expected_released);

    let remaining_available = client.get_available_funds(&project_id);
    assert_eq!(remaining_available, total_funding - expected_released);
}

#[test]
fn test_refund_request_and_processing() {
    let env = Env::default();
    let contract_id = env.register(EnergyProjectFundingEscrow, ());
    let client = EnergyProjectFundingEscrowClient::new(&env, &contract_id);

    client.initialize();

    let investor = Address::generate(&env);
    let project_manager = Address::generate(&env);
    let total_funding = 15_000_000_000i128;

    env.mock_all_auths();

    let project_id = client.initialize_project(
        &investor,
        &project_manager,
        &String::from_str(&env, "Refund Test Project"),
        &String::from_str(&env, "Testing refund process"),
        &total_funding,
        &3u32,
        &String::from_str(&env, "geothermal"),
        &8_000_000i128,
    );

    let refund_amount = 5_000_000_000i128;
    let reason = String::from_str(&env, "Project delays due to permit issues");

    client.request_refund(&project_id, &investor, &refund_amount, &reason);

    // Process the refund
    client.process_refund(&project_id, &project_manager);

    let project = client.get_project(&project_id);
    assert_eq!(project.status, ProjectStatus::Refunded);
    assert_eq!(project.escrow_status, EscrowStatus::Refunded);
}

#[test]
fn test_validation_functions() {
    let env = Env::default();
    let contract_id = env.register(EnergyProjectFundingEscrow, ());
    let client = EnergyProjectFundingEscrowClient::new(&env, &contract_id);

    let investor = Address::generate(&env);
    let project_manager = Address::generate(&env);

    // Test valid project validation
    let is_valid = client.validate_project(
        &investor,
        &project_manager,
        &10_000_000_000i128,
        &5u32,
        &String::from_str(&env, "solar"),
        &50_000_000i128,
    );
    assert!(is_valid);

    // Test milestone funding calculation
    let total_funding = 100_000_000_000i128;
    let percentage = 25u32;
    let expected_amount = client.calculate_milestone_funding(&total_funding, &percentage);
    assert_eq!(expected_amount, 25_000_000_000i128);
}

#[test]
#[should_panic(expected = "Invalid project setup parameters")]
fn test_invalid_project_setup() {
    let env = Env::default();
    let contract_id = env.register(EnergyProjectFundingEscrow, ());
    let client = EnergyProjectFundingEscrowClient::new(&env, &contract_id);

    client.initialize();

    let same_address = Address::generate(&env);

    env.mock_all_auths();

    // Should panic because investor and project manager are the same
    client.initialize_project(
        &same_address,
        &same_address,
        &String::from_str(&env, "Invalid Project"),
        &String::from_str(&env, "This should fail"),
        &10_000_000_000i128,
        &3u32,
        &String::from_str(&env, "solar"),
        &10_000_000i128,
    );
}