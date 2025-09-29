use crate::escrow::{storage_key_donation, Donation, DonationStatus, ADMIN};
use soroban_sdk::{contracttype, symbol_short, Address, Env, String, Symbol, Vec};

// Storage keys
const REFUND_REQUESTS: Symbol = symbol_short!("REFUNDS");
const REFUND_QUEUE: Symbol = symbol_short!("REF_Q");

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RefundRequest {
    pub donation_id: u64,
    pub requester: Address,
    pub reason: String,
    pub requested_at: u64,
    pub approved: bool,
    pub approved_at: Option<u64>,
    pub approver: Option<Address>,
}

pub fn refund(env: &Env, donation_id: u64, refund_processor: Address) {
    let donation = crate::escrow::get_donation(env, donation_id)
        .unwrap_or_else(|| panic!("Donation not found"));

    // Validate donation can be refunded
    if !can_refund(&donation) {
        panic!("Cannot refund donation");
    }

    // Verify refund processor is authorized (donor, recipient, or admin)
    let admin = env
        .storage()
        .instance()
        .get(&ADMIN)
        .unwrap_or_else(|| panic!("No admin set"));

    if refund_processor != donation.donor
        && refund_processor != donation.recipient
        && refund_processor != admin
    {
        panic!("Unauthorized refund processor");
    }

    // Create refund request
    let refund_request = RefundRequest {
        donation_id,
        requester: refund_processor.clone(),
        reason: String::from_str(env, "Manual refund request"),
        requested_at: env.ledger().timestamp(),
        approved: true, // Auto-approve for now
        approved_at: Some(env.ledger().timestamp()),
        approver: Some(refund_processor.clone()),
    };

    // Store refund request
    env.storage()
        .persistent()
        .set(&storage_key_refund_request(donation_id), &refund_request);

    // Update donation status
    let mut updated_donation = donation.clone();
    updated_donation.status = DonationStatus::Refunded;
    updated_donation.refunded_at = Some(env.ledger().timestamp());

    env.storage()
        .persistent()
        .set(&storage_key_donation(donation_id), &updated_donation);

    // Add to refund queue for processing
    add_to_refund_queue(env, donation_id);

    // Log the refund event
    env.events().publish(
        (symbol_short!("REFUNDED"), donation_id),
        (donation.amount, donation.token, donation.donor),
    );
}

pub fn request_refund(env: &Env, donation_id: u64, requester: Address, reason: String) {
    let donation = crate::escrow::get_donation(env, donation_id)
        .unwrap_or_else(|| panic!("Donation not found"));

    // Validate requester is donor or recipient
    if requester != donation.donor && requester != donation.recipient {
        panic!("Unauthorized requester");
    }

    // Check if refund request already exists
    if get_refund_request(env, donation_id).is_some() {
        panic!("Refund request already exists");
    }

    // Create refund request
    let refund_request = RefundRequest {
        donation_id,
        requester: requester.clone(),
        reason: reason.clone(),
        requested_at: env.ledger().timestamp(),
        approved: false,
        approved_at: None,
        approver: None,
    };

    // Store refund request
    env.storage()
        .persistent()
        .set(&storage_key_refund_request(donation_id), &refund_request);

    // Log the refund request event
    env.events().publish(
        (symbol_short!("REF_REQ"), donation_id),
        (requester, reason.clone()),
    );
}

pub fn approve_refund(env: &Env, donation_id: u64, approver: Address) {
    // Verify approver is admin
    let admin = env
        .storage()
        .instance()
        .get(&ADMIN)
        .unwrap_or_else(|| panic!("No admin set"));

    if approver != admin {
        panic!("Unauthorized approver");
    }

    let mut refund_request =
        get_refund_request(env, donation_id).unwrap_or_else(|| panic!("Refund request not found"));

    if refund_request.approved {
        panic!("Refund already approved");
    }

    // Approve refund request
    refund_request.approved = true;
    refund_request.approved_at = Some(env.ledger().timestamp());
    refund_request.approver = Some(approver.clone());

    env.storage()
        .persistent()
        .set(&storage_key_refund_request(donation_id), &refund_request);

    // Process the refund
    refund(env, donation_id, approver);
}

pub fn get_refund_request(env: &Env, donation_id: u64) -> Option<RefundRequest> {
    env.storage()
        .persistent()
        .get(&storage_key_refund_request(donation_id))
}

pub fn get_refund_queue(env: &Env) -> Vec<u64> {
    env.storage()
        .persistent()
        .get(&REFUND_QUEUE)
        .unwrap_or_else(|| Vec::new(&env))
}

pub fn validate_donation_parameters(
    env: &Env,
    amount: i128,
    _token: Symbol,
    milestones: &Vec<String>,
) -> Result<(), String> {
    if amount <= 0 {
        return Err(String::from_str(env, "Invalid amount"));
    }

    if milestones.len() == 0 {
        return Err(String::from_str(env, "No milestones provided"));
    }

    // Additional validation for medical donations
    if amount > 1_000_000_000_000_000_000 {
        // 1M tokens max
        return Err(String::from_str(env, "Amount too large"));
    }

    Ok(())
}

pub fn calculate_refund_amount(_env: &Env, donation: &Donation, completed_milestones: u32) -> i128 {
    let total_milestones = donation.milestones.len() as u32;

    if completed_milestones == 0 {
        // Full refund if no milestones completed
        return donation.amount;
    }

    if completed_milestones >= total_milestones {
        // No refund if all milestones completed
        return 0;
    }

    // Partial refund based on completed milestones
    let refund_percentage =
        (total_milestones - completed_milestones) as i128 * 100 / total_milestones as i128;
    donation.amount * refund_percentage / 100
}

pub fn get_donation_metrics(env: &Env, donation_id: u64) -> Option<DonationMetrics> {
    let donation = crate::escrow::get_donation(env, donation_id)?;

    let completed_milestones = count_completed_milestones(env, donation_id);
    let total_milestones = donation.milestones.len() as u32;
    let progress_percentage = if total_milestones > 0 {
        (completed_milestones as i128 * 100) / total_milestones as i128
    } else {
        0
    };

    Some(DonationMetrics {
        donation_id,
        total_milestones,
        completed_milestones,
        progress_percentage,
        days_since_creation: calculate_days_since(env, donation.created_at),
        days_since_funding: donation.funded_at.map(|t| calculate_days_since(env, t)),
    })
}

fn can_refund(donation: &Donation) -> bool {
    matches!(
        donation.status,
        DonationStatus::Pending
            | DonationStatus::Funded
            | DonationStatus::InProgress
            | DonationStatus::Paused
    )
}

fn count_completed_milestones(env: &Env, donation_id: u64) -> u32 {
    let donation = crate::escrow::get_donation(env, donation_id)
        .unwrap_or_else(|| panic!("Donation not found"));

    let mut completed = 0;
    for i in 0..donation.milestones.len() {
        if crate::release::get_milestone_status(env, donation_id, i as u32) {
            completed += 1;
        }
    }
    completed
}

fn calculate_days_since(env: &Env, timestamp: u64) -> u64 {
    let current_time = env.ledger().timestamp();
    (current_time - timestamp) / (24 * 60 * 60) // Convert to days
}

fn add_to_refund_queue(env: &Env, donation_id: u64) {
    let mut queue = get_refund_queue(env);
    queue.push_back(donation_id);
    env.storage().persistent().set(&REFUND_QUEUE, &queue);
}

fn storage_key_refund_request(donation_id: u64) -> (Symbol, u64) {
    (REFUND_REQUESTS, donation_id)
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DonationMetrics {
    pub donation_id: u64,
    pub total_milestones: u32,
    pub completed_milestones: u32,
    pub progress_percentage: i128,
    pub days_since_creation: u64,
    pub days_since_funding: Option<u64>,
}
