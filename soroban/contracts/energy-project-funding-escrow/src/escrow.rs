use crate::types::*;
use crate::utils::{validate_funding_amount, validate_project_setup};
use soroban_sdk::{Address, Env, String, Symbol, Vec};

pub fn initialize_project(
    env: Env,
    investor: Address,
    project_manager: Address,
    name: String,
    description: String,
    total_funding: i128,
    milestone_count: u32,
    energy_type: String,
    expected_capacity: i128,
) -> u64 {
    investor.require_auth();

    if !validate_project_setup(
        &env,
        &investor,
        &project_manager,
        total_funding,
        milestone_count,
    ) {
        panic!("Invalid project setup parameters");
    }

    let project_id: u64 = env
        .storage()
        .instance()
        .get(&DataKey::NextProjectId)
        .unwrap_or(1);

    let project_details = ProjectDetails {
        id: project_id,
        name: name.clone(),
        description: description.clone(),
        investor: investor.clone(),
        project_manager: project_manager.clone(),
        total_funding,
        released_funding: 0,
        milestone_count,
        status: ProjectStatus::Active,
        escrow_status: EscrowStatus::Funded,
        created_at: env.ledger().timestamp(),
        energy_type: energy_type.clone(),
        expected_capacity,
        compliance_verified: false,
    };

    env.storage()
        .persistent()
        .set(&DataKey::Project(project_id), &project_details);

    env.storage()
        .instance()
        .set(&DataKey::NextProjectId, &(project_id + 1));

    env.events().publish(
        (Symbol::new(&env, "ProjectInitialized"),),
        (project_id, investor, project_manager, total_funding),
    );

    project_id
}

pub fn deposit_funds(env: Env, project_id: u64, depositor: Address, amount: i128) {
    depositor.require_auth();

    if !validate_funding_amount(&env, amount) {
        panic!("Invalid funding amount");
    }

    let mut project: ProjectDetails = env
        .storage()
        .persistent()
        .get(&DataKey::Project(project_id))
        .unwrap_or_else(|| panic!("Project not found"));

    if project.investor != depositor {
        panic!("Only project investor can deposit funds");
    }

    if project.status != ProjectStatus::Active {
        panic!("Project is not active");
    }

    project.total_funding += amount;

    env.storage()
        .persistent()
        .set(&DataKey::Project(project_id), &project);

    env.events().publish(
        (Symbol::new(&env, "FundsDeposited"),),
        (project_id, depositor, amount, project.total_funding),
    );
}

pub fn release_funds(env: Env, project_id: u64, milestone_id: u32, approver: Address) {
    approver.require_auth();

    let mut project: ProjectDetails = env
        .storage()
        .persistent()
        .get(&DataKey::Project(project_id))
        .unwrap_or_else(|| panic!("Project not found"));

    if project.status != ProjectStatus::Active {
        panic!("Project is not active");
    }

    let milestone: MilestoneDetails = env
        .storage()
        .persistent()
        .get(&DataKey::Milestone(project_id, milestone_id))
        .unwrap_or_else(|| panic!("Milestone not found"));

    if milestone.status != MilestoneStatus::Completed {
        panic!("Milestone not completed");
    }

    let mut pending_release: PendingRelease = env
        .storage()
        .persistent()
        .get(&DataKey::PendingRelease(project_id))
        .unwrap_or_else(|| panic!("No pending release found"));

    if pending_release.milestone_id != milestone_id {
        panic!("Milestone mismatch");
    }

    let multisig_config: Option<MultisigConfig> = env
        .storage()
        .persistent()
        .get(&DataKey::MultisigRequirement(project_id));

    if let Some(config) = multisig_config {
        if pending_release.amount >= config.threshold_amount {
            if !config.authorized_signers.contains(&approver) {
                panic!("Unauthorized signer");
            }

            if !pending_release.approvers.contains(&approver) {
                pending_release.approvers.push_back(approver.clone());
                pending_release.current_approvals += 1;
            }

            if pending_release.current_approvals < pending_release.required_approvals {
                env.storage()
                    .persistent()
                    .set(&DataKey::PendingRelease(project_id), &pending_release);

                env.events().publish(
                    (Symbol::new(&env, "ApprovalAdded"),),
                    (
                        project_id,
                        milestone_id,
                        approver,
                        pending_release.current_approvals,
                    ),
                );
                return;
            }
        }
    }

    if approver != project.investor && approver != project.project_manager {
        panic!("Unauthorized to release funds");
    }

    let release_amount = pending_release.amount;

    if release_amount > (project.total_funding - project.released_funding) {
        panic!("Insufficient funds in escrow");
    }

    project.released_funding += release_amount;

    if project.released_funding >= project.total_funding {
        project.escrow_status = EscrowStatus::FullyReleased;
        project.status = ProjectStatus::Completed;
    } else {
        project.escrow_status = EscrowStatus::PartiallyReleased;
    }

    env.storage()
        .persistent()
        .set(&DataKey::Project(project_id), &project);

    env.storage()
        .persistent()
        .remove(&DataKey::PendingRelease(project_id));

    env.events().publish(
        (Symbol::new(&env, "FundsReleased"),),
        (
            project_id,
            milestone_id,
            project.project_manager,
            release_amount,
        ),
    );
}

pub fn request_refund(env: Env, project_id: u64, requester: Address, amount: i128, reason: String) {
    requester.require_auth();

    let project: ProjectDetails = env
        .storage()
        .persistent()
        .get(&DataKey::Project(project_id))
        .unwrap_or_else(|| panic!("Project not found"));

    if requester != project.investor {
        panic!("Only investor can request refund");
    }

    if project.status == ProjectStatus::Completed {
        panic!("Cannot refund completed project");
    }

    let available_funds = project.total_funding - project.released_funding;
    if amount > available_funds {
        panic!("Refund amount exceeds available funds");
    }

    let refund_request = RefundRequest {
        project_id,
        amount,
        reason,
        requested_by: requester.clone(),
        requested_at: env.ledger().timestamp(),
        approved: false,
        processed: false,
    };

    env.storage()
        .persistent()
        .set(&DataKey::RefundRequest(project_id), &refund_request);

    env.events().publish(
        (Symbol::new(&env, "RefundRequested"),),
        (project_id, requester, amount),
    );
}

pub fn process_refund(env: Env, project_id: u64, approver: Address) {
    approver.require_auth();

    let mut project: ProjectDetails = env
        .storage()
        .persistent()
        .get(&DataKey::Project(project_id))
        .unwrap_or_else(|| panic!("Project not found"));

    let mut refund_request: RefundRequest = env
        .storage()
        .persistent()
        .get(&DataKey::RefundRequest(project_id))
        .unwrap_or_else(|| panic!("No refund request found"));

    if refund_request.processed {
        panic!("Refund already processed");
    }

    if approver != project.project_manager && approver != project.investor {
        panic!("Unauthorized to approve refund");
    }

    refund_request.approved = true;
    refund_request.processed = true;

    project.status = ProjectStatus::Refunded;
    project.escrow_status = EscrowStatus::Refunded;

    env.storage()
        .persistent()
        .set(&DataKey::Project(project_id), &project);

    env.storage()
        .persistent()
        .set(&DataKey::RefundRequest(project_id), &refund_request);

    env.events().publish(
        (Symbol::new(&env, "RefundProcessed"),),
        (project_id, project.investor, refund_request.amount),
    );
}

pub fn get_project_details(env: Env, project_id: u64) -> ProjectDetails {
    env.storage()
        .persistent()
        .get(&DataKey::Project(project_id))
        .unwrap_or_else(|| panic!("Project not found"))
}

pub fn get_available_funds(env: Env, project_id: u64) -> i128 {
    let project: ProjectDetails = env
        .storage()
        .persistent()
        .get(&DataKey::Project(project_id))
        .unwrap_or_else(|| panic!("Project not found"));

    project.total_funding - project.released_funding
}

pub fn setup_multisig(
    env: Env,
    project_id: u64,
    setup_by: Address,
    required_signatures: u32,
    authorized_signers: Vec<Address>,
    threshold_amount: i128,
) {
    setup_by.require_auth();

    let project: ProjectDetails = env
        .storage()
        .persistent()
        .get(&DataKey::Project(project_id))
        .unwrap_or_else(|| panic!("Project not found"));

    if setup_by != project.investor {
        panic!("Only investor can setup multisig");
    }

    if required_signatures == 0 || required_signatures > (authorized_signers.len() as u32) {
        panic!("Invalid signature requirements");
    }

    let multisig_config = MultisigConfig {
        project_id,
        required_signatures,
        authorized_signers,
        threshold_amount,
    };

    env.storage()
        .persistent()
        .set(&DataKey::MultisigRequirement(project_id), &multisig_config);

    env.events().publish(
        (Symbol::new(&env, "MultisigConfigured"),),
        (project_id, required_signatures, threshold_amount),
    );
}
