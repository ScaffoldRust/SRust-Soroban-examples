use crate::types::*;
use soroban_sdk::{vec, Address, Env, String, Vec};

const MIN_FUNDING_AMOUNT: i128 = 1_000_000; // 1 XLM (in stroops)
const MAX_FUNDING_AMOUNT: i128 = 1_000_000_000_000; // 1M XLM (in stroops)
const MIN_MILESTONES: u32 = 1;
const MAX_MILESTONES: u32 = 20;
const MAX_FUNDING_PERCENTAGE: u32 = 100;

pub fn validate_project_setup(
    env: &Env,
    investor: &Address,
    project_manager: &Address,
    total_funding: i128,
    milestone_count: u32,
) -> bool {
    if investor == project_manager {
        return false;
    }

    if !validate_funding_amount(env, total_funding) {
        return false;
    }

    if milestone_count < MIN_MILESTONES || milestone_count > MAX_MILESTONES {
        return false;
    }

    true
}

pub fn validate_funding_amount(_env: &Env, amount: i128) -> bool {
    amount >= MIN_FUNDING_AMOUNT && amount <= MAX_FUNDING_AMOUNT
}

pub fn validate_milestone_data(env: &Env, funding_percentage: u32, due_date: u64) -> bool {
    if funding_percentage == 0 || funding_percentage > MAX_FUNDING_PERCENTAGE {
        return false;
    }

    let current_time = env.ledger().timestamp();
    if due_date <= current_time {
        return false;
    }

    true
}

pub fn calculate_milestone_funding(_env: &Env, total_funding: i128, percentage: u32) -> i128 {
    if percentage > MAX_FUNDING_PERCENTAGE {
        panic!("Invalid funding percentage");
    }

    (total_funding * percentage as i128) / 100
}

pub fn validate_energy_project_data(
    env: &Env,
    energy_type: &String,
    expected_capacity: i128,
) -> bool {
    let valid_energy_types = vec![
        env,
        String::from_str(env, "solar"),
        String::from_str(env, "wind"),
        String::from_str(env, "hydro"),
        String::from_str(env, "geothermal"),
        String::from_str(env, "biomass"),
        String::from_str(env, "nuclear"),
        String::from_str(env, "hybrid"),
    ];

    let is_valid_type = valid_energy_types.iter().any(|t| t == *energy_type);

    if !is_valid_type {
        return false;
    }

    if expected_capacity <= 0 {
        return false;
    }

    true
}

pub fn calculate_refund_amount(
    _env: &Env,
    project: &ProjectDetails,
    milestones_completed: u32,
) -> i128 {
    if project.milestone_count == 0 {
        return project.total_funding - project.released_funding;
    }

    let completion_percentage = (milestones_completed * 100) / project.milestone_count;
    let earned_percentage = if completion_percentage > 100 {
        100
    } else {
        completion_percentage
    };

    let earned_amount = (project.total_funding * earned_percentage as i128) / 100;
    let refundable_amount = project.total_funding - earned_amount;

    if refundable_amount < 0 {
        0
    } else {
        refundable_amount
    }
}

pub fn validate_verification_requirements(env: &Env, required_verifications: &Vec<String>) -> bool {
    if required_verifications.len() == 0 {
        return false;
    }

    let valid_verification_types = vec![
        env,
        String::from_str(env, "environmental_impact"),
        String::from_str(env, "safety_compliance"),
        String::from_str(env, "technical_specification"),
        String::from_str(env, "financial_audit"),
        String::from_str(env, "regulatory_approval"),
        String::from_str(env, "performance_metrics"),
        String::from_str(env, "carbon_verification"),
        String::from_str(env, "grid_connection"),
        String::from_str(env, "commissioning_test"),
        String::from_str(env, "insurance_validation"),
    ];

    for verification in required_verifications.iter() {
        if !valid_verification_types.iter().any(|v| v == verification) {
            return false;
        }
    }

    true
}

pub fn calculate_performance_bonus(
    _env: &Env,
    expected_output: i128,
    actual_output: i128,
    bonus_threshold: u32,
) -> i128 {
    if expected_output <= 0 || actual_output <= 0 {
        return 0;
    }

    let performance_ratio = (actual_output * 100) / expected_output;

    if performance_ratio >= bonus_threshold as i128 {
        let bonus_percentage = performance_ratio - 100;
        let max_bonus_percentage = 20; // Maximum 20% bonus

        let final_bonus_percentage = if bonus_percentage > max_bonus_percentage {
            max_bonus_percentage
        } else {
            bonus_percentage
        };

        (expected_output * final_bonus_percentage) / 100
    } else {
        0
    }
}

pub fn validate_compliance_status(
    env: &Env,
    _project: &ProjectDetails,
    compliance_documents: &Vec<String>,
) -> bool {
    if compliance_documents.len() == 0 {
        return false;
    }

    let required_documents = vec![
        env,
        String::from_str(env, "environmental_permit"),
        String::from_str(env, "construction_permit"),
        String::from_str(env, "grid_interconnection_agreement"),
        String::from_str(env, "power_purchase_agreement"),
        String::from_str(env, "insurance_certificate"),
    ];

    for doc in required_documents.iter() {
        if !compliance_documents.iter().any(|d| d == doc) {
            return false;
        }
    }

    true
}

pub fn calculate_penalty_amount(
    _env: &Env,
    milestone: &MilestoneDetails,
    days_overdue: u64,
) -> i128 {
    let base_penalty_rate = 1; // 1% per day overdue
    let max_penalty_rate = 25; // Maximum 25% penalty

    let penalty_percentage = if days_overdue > max_penalty_rate {
        max_penalty_rate as i128
    } else {
        (days_overdue * base_penalty_rate) as i128
    };

    (milestone.funding_amount * penalty_percentage) / 100
}

pub fn is_milestone_overdue(env: &Env, milestone: &MilestoneDetails) -> bool {
    let current_time = env.ledger().timestamp();
    current_time > milestone.due_date && milestone.status != MilestoneStatus::Completed
}

pub fn calculate_days_overdue(env: &Env, due_date: u64) -> u64 {
    let current_time = env.ledger().timestamp();
    if current_time > due_date {
        (current_time - due_date) / 86400 // Convert seconds to days
    } else {
        0
    }
}

pub fn validate_multisig_setup(
    _env: &Env,
    required_signatures: u32,
    authorized_signers: &Vec<Address>,
    threshold_amount: i128,
) -> bool {
    if required_signatures == 0 {
        return false;
    }

    if required_signatures > (authorized_signers.len() as u32) {
        return false;
    }

    if threshold_amount <= 0 {
        return false;
    }

    // Check for duplicate signers
    for i in 0..authorized_signers.len() {
        for j in (i + 1)..authorized_signers.len() {
            if authorized_signers.get(i).unwrap() == authorized_signers.get(j).unwrap() {
                return false;
            }
        }
    }

    true
}

pub fn format_project_status_message(status: &ProjectStatus) -> String {
    match status {
        ProjectStatus::Active => String::from_str(&Env::default(), "Project is active and funded"),
        ProjectStatus::Completed => {
            String::from_str(&Env::default(), "Project successfully completed")
        }
        ProjectStatus::Cancelled => String::from_str(&Env::default(), "Project was cancelled"),
        ProjectStatus::Refunded => String::from_str(&Env::default(), "Project funds were refunded"),
    }
}

pub fn format_milestone_status_message(status: &MilestoneStatus) -> String {
    match status {
        MilestoneStatus::Pending => String::from_str(&Env::default(), "Milestone awaiting start"),
        MilestoneStatus::InProgress => String::from_str(&Env::default(), "Milestone in progress"),
        MilestoneStatus::Completed => {
            String::from_str(&Env::default(), "Milestone completed successfully")
        }
        MilestoneStatus::Failed => {
            String::from_str(&Env::default(), "Milestone failed to complete")
        }
    }
}
