use soroban_sdk::{Address, Env, String, Symbol, Vec};
use crate::types::*;
use crate::utils::{validate_milestone_data, calculate_milestone_funding};

pub fn create_milestone(
    env: Env,
    project_id: u64,
    creator: Address,
    name: String,
    description: String,
    funding_percentage: u32,
    required_verifications: Vec<String>,
    due_date: u64,
    energy_output_target: Option<i128>,
    carbon_offset_target: Option<i128>,
) -> u32 {
    creator.require_auth();

    let project: ProjectDetails = env.storage()
        .persistent()
        .get(&DataKey::Project(project_id))
        .unwrap_or_else(|| panic!("Project not found"));

    if creator != project.project_manager && creator != project.investor {
        panic!("Unauthorized to create milestone");
    }

    if project.status != ProjectStatus::Active {
        panic!("Project is not active");
    }

    if !validate_milestone_data(&env, funding_percentage, due_date) {
        panic!("Invalid milestone data");
    }

    let milestone_id = get_next_milestone_id(&env, project_id);
    let funding_amount = calculate_milestone_funding(&env, project.total_funding, funding_percentage);

    let milestone = MilestoneDetails {
        project_id,
        milestone_id,
        name: name.clone(),
        description: description.clone(),
        funding_percentage,
        funding_amount,
        status: MilestoneStatus::Pending,
        required_verifications: required_verifications.clone(),
        completed_verifications: Vec::new(&env),
        due_date,
        completed_at: None,
        energy_output_target,
        carbon_offset_target,
    };

    env.storage()
        .persistent()
        .set(&DataKey::Milestone(project_id, milestone_id), &milestone);

    env.events().publish(
        (Symbol::new(&env, "MilestoneCreated"),),
        (project_id, milestone_id, name, funding_percentage),
    );

    milestone_id
}

pub fn start_milestone(env: Env, project_id: u64, milestone_id: u32, starter: Address) {
    starter.require_auth();

    let project: ProjectDetails = env.storage()
        .persistent()
        .get(&DataKey::Project(project_id))
        .unwrap_or_else(|| panic!("Project not found"));

    if starter != project.project_manager {
        panic!("Only project manager can start milestones");
    }

    let mut milestone: MilestoneDetails = env.storage()
        .persistent()
        .get(&DataKey::Milestone(project_id, milestone_id))
        .unwrap_or_else(|| panic!("Milestone not found"));

    if milestone.status != MilestoneStatus::Pending {
        panic!("Milestone cannot be started");
    }

    milestone.status = MilestoneStatus::InProgress;

    env.storage()
        .persistent()
        .set(&DataKey::Milestone(project_id, milestone_id), &milestone);

    env.events().publish(
        (Symbol::new(&env, "MilestoneStarted"),),
        (project_id, milestone_id, starter),
    );
}

pub fn verify_milestone(
    env: Env,
    project_id: u64,
    milestone_id: u32,
    verifier: Address,
    verification_type: String,
    _verification_data: String,
) {
    verifier.require_auth();

    let project: ProjectDetails = env.storage()
        .persistent()
        .get(&DataKey::Project(project_id))
        .unwrap_or_else(|| panic!("Project not found"));

    if verifier != project.project_manager && verifier != project.investor {
        panic!("Unauthorized to verify milestone");
    }

    let mut milestone: MilestoneDetails = env.storage()
        .persistent()
        .get(&DataKey::Milestone(project_id, milestone_id))
        .unwrap_or_else(|| panic!("Milestone not found"));

    if milestone.status != MilestoneStatus::InProgress {
        panic!("Milestone is not in progress");
    }

    if !milestone.required_verifications.contains(&verification_type) {
        panic!("Verification type not required");
    }

    if milestone.completed_verifications.contains(&verification_type) {
        panic!("Verification already completed");
    }

    milestone.completed_verifications.push_back(verification_type.clone());

    if milestone.completed_verifications.len() == milestone.required_verifications.len() {
        milestone.status = MilestoneStatus::Completed;
        milestone.completed_at = Some(env.ledger().timestamp());

        let pending_release = PendingRelease {
            project_id,
            milestone_id,
            amount: milestone.funding_amount,
            requested_by: project.project_manager.clone(),
            requested_at: env.ledger().timestamp(),
            required_approvals: 1,
            current_approvals: 0,
            approvers: Vec::new(&env),
        };

        env.storage()
            .persistent()
            .set(&DataKey::PendingRelease(project_id), &pending_release);

        env.events().publish(
            (Symbol::new(&env, "MilestoneCompleted"),),
            (project_id, milestone_id, milestone.funding_amount),
        );
    }

    env.storage()
        .persistent()
        .set(&DataKey::Milestone(project_id, milestone_id), &milestone);

    env.events().publish(
        (Symbol::new(&env, "MilestoneVerified"),),
        (project_id, milestone_id, verification_type, verifier),
    );
}

pub fn fail_milestone(
    env: Env,
    project_id: u64,
    milestone_id: u32,
    manager: Address,
    reason: String,
) {
    manager.require_auth();

    let project: ProjectDetails = env.storage()
        .persistent()
        .get(&DataKey::Project(project_id))
        .unwrap_or_else(|| panic!("Project not found"));

    if manager != project.project_manager {
        panic!("Only project manager can fail milestones");
    }

    let mut milestone: MilestoneDetails = env.storage()
        .persistent()
        .get(&DataKey::Milestone(project_id, milestone_id))
        .unwrap_or_else(|| panic!("Milestone not found"));

    if milestone.status != MilestoneStatus::InProgress {
        panic!("Can only fail milestones in progress");
    }

    milestone.status = MilestoneStatus::Failed;

    env.storage()
        .persistent()
        .set(&DataKey::Milestone(project_id, milestone_id), &milestone);

    env.events().publish(
        (Symbol::new(&env, "MilestoneFailed"),),
        (project_id, milestone_id, reason),
    );
}

pub fn update_project_metrics(
    env: Env,
    project_id: u64,
    updater: Address,
    actual_energy_output: i128,
    actual_carbon_offset: i128,
    efficiency_rating: u32,
) {
    updater.require_auth();

    let project: ProjectDetails = env.storage()
        .persistent()
        .get(&DataKey::Project(project_id))
        .unwrap_or_else(|| panic!("Project not found"));

    if updater != project.project_manager {
        panic!("Only project manager can update metrics");
    }

    if efficiency_rating > 100 {
        panic!("Efficiency rating cannot exceed 100%");
    }

    let metrics = ProjectMetrics {
        actual_energy_output,
        actual_carbon_offset,
        efficiency_rating,
        last_updated: env.ledger().timestamp(),
    };

    env.storage()
        .persistent()
        .set(&DataKey::Project(project_id), &metrics);

    env.events().publish(
        (Symbol::new(&env, "MetricsUpdated"),),
        (project_id, actual_energy_output, actual_carbon_offset, efficiency_rating),
    );
}

pub fn get_milestone_details(env: Env, project_id: u64, milestone_id: u32) -> MilestoneDetails {
    env.storage()
        .persistent()
        .get(&DataKey::Milestone(project_id, milestone_id))
        .unwrap_or_else(|| panic!("Milestone not found"))
}

pub fn get_project_milestones(env: Env, project_id: u64) -> Vec<MilestoneDetails> {
    let project: ProjectDetails = env.storage()
        .persistent()
        .get(&DataKey::Project(project_id))
        .unwrap_or_else(|| panic!("Project not found"));

    let mut milestones = Vec::new(&env);

    for i in 1..=project.milestone_count {
        if let Some(milestone) = env.storage()
            .persistent()
            .get::<DataKey, MilestoneDetails>(&DataKey::Milestone(project_id, i)) {
            milestones.push_back(milestone);
        }
    }

    milestones
}

pub fn get_pending_release(env: Env, project_id: u64) -> Option<PendingRelease> {
    env.storage()
        .persistent()
        .get(&DataKey::PendingRelease(project_id))
}

pub fn calculate_project_progress(env: Env, project_id: u64) -> u32 {
    let project: ProjectDetails = env.storage()
        .persistent()
        .get(&DataKey::Project(project_id))
        .unwrap_or_else(|| panic!("Project not found"));

    let mut completed_milestones = 0u32;

    for i in 1..=project.milestone_count {
        if let Some(milestone) = env.storage()
            .persistent()
            .get::<DataKey, MilestoneDetails>(&DataKey::Milestone(project_id, i)) {
            if milestone.status == MilestoneStatus::Completed {
                completed_milestones += 1;
            }
        }
    }

    if project.milestone_count == 0 {
        0
    } else {
        (completed_milestones * 100) / project.milestone_count
    }
}

fn get_next_milestone_id(env: &Env, project_id: u64) -> u32 {
    let project: ProjectDetails = env.storage()
        .persistent()
        .get(&DataKey::Project(project_id))
        .unwrap_or_else(|| panic!("Project not found"));

    let mut next_id = 1u32;

    for i in 1..=project.milestone_count + 10 {
        if env.storage()
            .persistent()
            .get::<DataKey, MilestoneDetails>(&DataKey::Milestone(project_id, i))
            .is_none() {
            next_id = i;
            break;
        }
    }

    next_id
}