use soroban_sdk::{contracttype, symbol_short, Address, BytesN, Env, String, Symbol, Vec};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ClaimStatus {
    Submitted,     // Claim submitted by patient
    UnderReview,   // Claim is being reviewed
    Approved,      // Claim approved for payout
    Rejected,      // Claim rejected
    Paid,          // Payout completed
    Disputed,      // Claim is under dispute
    Cancelled,     // Claim cancelled by patient
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Claim {
    pub claim_id: u64,
    pub patient: Address,
    pub insurer: Address,
    pub claim_amount: i128,
    pub approved_amount: i128,
    pub currency: Address,
    pub diagnosis_code: String,
    pub evidence_hash: BytesN<32>,  // Hash pointer to medical records/evidence
    pub status: ClaimStatus,
    pub submitted_at: u64,
    pub reviewed_at: Option<u64>,
    pub approved_at: Option<u64>,
    pub paid_at: Option<u64>,
    pub rejection_reason: Option<String>,
    pub auto_approved: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DisputeRecord {
    pub claim_id: u64,
    pub disputer: Address,
    pub reason: String,
    pub disputed_at: u64,
    pub resolved_at: Option<u64>,
    pub resolution: Option<String>,
}

// Storage keys
const CLAIMS: Symbol = symbol_short!("CLAIMS");
const PATIENT_CLAIMS: Symbol = symbol_short!("PAT_CLM");
const INSURER_CLAIMS: Symbol = symbol_short!("INS_CLM");
const PENDING_QUEUE: Symbol = symbol_short!("PEND_Q");
const DISPUTES: Symbol = symbol_short!("DISPUTES");
pub const ADMIN: Symbol = symbol_short!("ADMIN");
pub const NEXT_CLAIM_ID: Symbol = symbol_short!("NEXT_CID");

pub fn submit_claim(
    env: &Env,
    patient: Address,
    insurer: Address,
    claim_amount: i128,
    currency: Address,
    diagnosis_code: String,
    evidence_hash: BytesN<32>,
) -> u64 {
    // Validate inputs
    patient.require_auth();

    if claim_amount <= 0 {
        panic!("Invalid claim amount");
    }

    if diagnosis_code.len() == 0 {
        panic!("Diagnosis code required");
    }

    // Get next claim ID
    let claim_id = get_next_claim_id(env);

    let claim = Claim {
        claim_id,
        patient: patient.clone(),
        insurer: insurer.clone(),
        claim_amount,
        approved_amount: 0,
        currency,
        diagnosis_code,
        evidence_hash,
        status: ClaimStatus::Submitted,
        submitted_at: env.ledger().timestamp(),
        reviewed_at: None,
        approved_at: None,
        paid_at: None,
        rejection_reason: None,
        auto_approved: false,
    };

    // Store claim
    env.storage()
        .persistent()
        .set(&storage_key_claim(claim_id), &claim);

    // Add to patient's claims list
    add_patient_claim(env, &patient, claim_id);

    // Add to insurer's claims list
    add_insurer_claim(env, &insurer, claim_id);

    // Add to pending queue for processing
    add_to_pending_queue(env, claim_id);

    claim_id
}

pub fn update_claim_status(
    env: &Env,
    claim_id: u64,
    new_status: ClaimStatus,
    approved_amount: Option<i128>,
    rejection_reason: Option<String>,
    auto_approved: Option<bool>,
) {
    let mut claim = get_claim(env, claim_id).unwrap_or_else(|| panic!("Claim not found"));

    claim.status = new_status.clone();
    claim.reviewed_at = Some(env.ledger().timestamp());

    if let Some(is_auto) = auto_approved {
        claim.auto_approved = is_auto;
    }

    match new_status {
        ClaimStatus::Approved => {
            claim.approved_at = Some(env.ledger().timestamp());
            if let Some(amount) = approved_amount {
                claim.approved_amount = amount;
            } else {
                claim.approved_amount = claim.claim_amount;
            }
        }
        ClaimStatus::Rejected => {
            claim.rejection_reason = rejection_reason;
        }
        ClaimStatus::Paid => {
            claim.paid_at = Some(env.ledger().timestamp());
        }
        _ => {}
    }

    env.storage()
        .persistent()
        .set(&storage_key_claim(claim_id), &claim);
}

pub fn get_claim(env: &Env, claim_id: u64) -> Option<Claim> {
    env.storage()
        .persistent()
        .get(&storage_key_claim(claim_id))
}

pub fn get_claim_status(env: &Env, claim_id: u64) -> Option<ClaimStatus> {
    get_claim(env, claim_id).map(|c| c.status)
}

pub fn get_patient_claims(env: &Env, patient: Address) -> Vec<u64> {
    env.storage()
        .persistent()
        .get(&storage_key_patient_claims(&patient))
        .unwrap_or_else(|| Vec::new(&env))
}

pub fn get_insurer_claims(env: &Env, insurer: Address) -> Vec<u64> {
    env.storage()
        .persistent()
        .get(&storage_key_insurer_claims(&insurer))
        .unwrap_or_else(|| Vec::new(&env))
}

pub fn get_pending_queue(env: &Env) -> Vec<u64> {
    env.storage()
        .persistent()
        .get(&PENDING_QUEUE)
        .unwrap_or_else(|| Vec::new(&env))
}

pub fn remove_from_pending_queue(env: &Env, claim_id: u64) {
    let queue = get_pending_queue(env);
    let mut new_queue = Vec::new(&env);

    for i in 0..queue.len() {
        let id = queue.get(i).unwrap();
        if id != claim_id {
            new_queue.push_back(id);
        }
    }

    env.storage().persistent().set(&PENDING_QUEUE, &new_queue);
}

pub fn file_dispute(
    env: &Env,
    claim_id: u64,
    disputer: Address,
    reason: String,
) {
    disputer.require_auth();

    let mut claim = get_claim(env, claim_id).unwrap_or_else(|| panic!("Claim not found"));

    // Verify disputer is patient or insurer
    if disputer != claim.patient && disputer != claim.insurer {
        panic!("Unauthorized disputer");
    }

    let dispute = DisputeRecord {
        claim_id,
        disputer,
        reason,
        disputed_at: env.ledger().timestamp(),
        resolved_at: None,
        resolution: None,
    };

    claim.status = ClaimStatus::Disputed;

    env.storage()
        .persistent()
        .set(&storage_key_claim(claim_id), &claim);

    env.storage()
        .persistent()
        .set(&storage_key_dispute(claim_id), &dispute);
}

pub fn resolve_dispute(
    env: &Env,
    claim_id: u64,
    admin: Address,
    resolution: String,
    final_status: ClaimStatus,
) {
    // Verify admin
    let stored_admin: Address = env
        .storage()
        .instance()
        .get(&ADMIN)
        .unwrap_or_else(|| panic!("No admin set"));

    if stored_admin != admin {
        panic!("Unauthorized admin");
    }

    let mut dispute = get_dispute(env, claim_id).unwrap_or_else(|| panic!("Dispute not found"));
    let mut claim = get_claim(env, claim_id).unwrap_or_else(|| panic!("Claim not found"));

    dispute.resolved_at = Some(env.ledger().timestamp());
    dispute.resolution = Some(resolution);

    claim.status = final_status;

    env.storage()
        .persistent()
        .set(&storage_key_dispute(claim_id), &dispute);

    env.storage()
        .persistent()
        .set(&storage_key_claim(claim_id), &claim);
}

pub fn get_dispute(env: &Env, claim_id: u64) -> Option<DisputeRecord> {
    env.storage()
        .persistent()
        .get(&storage_key_dispute(claim_id))
}

fn get_next_claim_id(env: &Env) -> u64 {
    let current_id: u64 = env
        .storage()
        .instance()
        .get(&NEXT_CLAIM_ID)
        .unwrap_or(1);

    env.storage().instance().set(&NEXT_CLAIM_ID, &(current_id + 1));

    current_id
}

fn add_patient_claim(env: &Env, patient: &Address, claim_id: u64) {
    let mut patient_claims = get_patient_claims(env, patient.clone());
    patient_claims.push_back(claim_id);
    env.storage()
        .persistent()
        .set(&storage_key_patient_claims(patient), &patient_claims);
}

fn add_insurer_claim(env: &Env, insurer: &Address, claim_id: u64) {
    let mut insurer_claims = get_insurer_claims(env, insurer.clone());
    insurer_claims.push_back(claim_id);
    env.storage()
        .persistent()
        .set(&storage_key_insurer_claims(insurer), &insurer_claims);
}

fn add_to_pending_queue(env: &Env, claim_id: u64) {
    let mut queue = get_pending_queue(env);
    queue.push_back(claim_id);
    env.storage().persistent().set(&PENDING_QUEUE, &queue);
}

pub fn storage_key_claim(claim_id: u64) -> (Symbol, u64) {
    (CLAIMS, claim_id)
}

fn storage_key_patient_claims(patient: &Address) -> (Symbol, Address) {
    (PATIENT_CLAIMS, patient.clone())
}

fn storage_key_insurer_claims(insurer: &Address) -> (Symbol, Address) {
    (INSURER_CLAIMS, insurer.clone())
}

fn storage_key_dispute(claim_id: u64) -> (Symbol, u64) {
    (DISPUTES, claim_id)
}
