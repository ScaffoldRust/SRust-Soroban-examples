use soroban_sdk::{contracttype, Address, String, Vec};

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum DataKey {
    NextProjectId,
    Project(u64),
    Milestone(u64, u32),
    PendingRelease(u64),
    RefundRequest(u64),
    MultisigRequirement(u64),
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum ProjectStatus {
    Active,
    Completed,
    Cancelled,
    Refunded,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum MilestoneStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum EscrowStatus {
    Funded,
    PartiallyReleased,
    FullyReleased,
    Refunded,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct ProjectDetails {
    pub id: u64,
    pub name: String,
    pub description: String,
    pub investor: Address,
    pub project_manager: Address,
    pub total_funding: i128,
    pub released_funding: i128,
    pub milestone_count: u32,
    pub status: ProjectStatus,
    pub escrow_status: EscrowStatus,
    pub created_at: u64,
    pub energy_type: String,
    pub expected_capacity: i128,
    pub compliance_verified: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct MilestoneDetails {
    pub project_id: u64,
    pub milestone_id: u32,
    pub name: String,
    pub description: String,
    pub funding_percentage: u32,
    pub funding_amount: i128,
    pub status: MilestoneStatus,
    pub required_verifications: Vec<String>,
    pub completed_verifications: Vec<String>,
    pub due_date: u64,
    pub completed_at: Option<u64>,
    pub energy_output_target: Option<i128>,
    pub carbon_offset_target: Option<i128>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct PendingRelease {
    pub project_id: u64,
    pub milestone_id: u32,
    pub amount: i128,
    pub requested_by: Address,
    pub requested_at: u64,
    pub required_approvals: u32,
    pub current_approvals: u32,
    pub approvers: Vec<Address>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct RefundRequest {
    pub project_id: u64,
    pub amount: i128,
    pub reason: String,
    pub requested_by: Address,
    pub requested_at: u64,
    pub approved: bool,
    pub processed: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct MultisigConfig {
    pub project_id: u64,
    pub required_signatures: u32,
    pub authorized_signers: Vec<Address>,
    pub threshold_amount: i128,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct ProjectMetrics {
    pub actual_energy_output: i128,
    pub actual_carbon_offset: i128,
    pub efficiency_rating: u32,
    pub last_updated: u64,
}
