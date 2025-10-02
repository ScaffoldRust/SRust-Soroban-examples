use soroban_sdk::{contracttype, symbol_short, Address, Env, String, Symbol, Vec};

use crate::claims::{self, Claim, ClaimStatus};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AutoApprovalRule {
    pub rule_id: u64,
    pub max_amount: i128,
    pub diagnosis_codes: Vec<String>,
    pub enabled: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PayoutRecord {
    pub claim_id: u64,
    pub recipient: Address,
    pub amount: i128,
    pub currency: Address,
    pub paid_at: u64,
    pub transaction_hash: Option<String>,
}

// Storage keys
const AUTO_RULES: Symbol = symbol_short!("AUTO_RLS");
const NEXT_RULE_ID: Symbol = symbol_short!("NEXT_RID");
const PAYOUTS: Symbol = symbol_short!("PAYOUTS");

pub fn initialize_automator(
    env: &Env,
    admin: Address,
    default_max_auto_amount: i128,
) {
    admin.require_auth();

    // Store admin
    env.storage().instance().set(&claims::ADMIN, &admin);

    // Initialize next rule ID
    env.storage().instance().set(&NEXT_RULE_ID, &1u64);

    // Create default auto-approval rule
    let default_rule = AutoApprovalRule {
        rule_id: 0,
        max_amount: default_max_auto_amount,
        diagnosis_codes: Vec::new(&env),
        enabled: true,
    };

    env.storage()
        .persistent()
        .set(&storage_key_rule(0), &default_rule);
}

pub fn add_auto_approval_rule(
    env: &Env,
    admin: Address,
    max_amount: i128,
    diagnosis_codes: Vec<String>,
) -> u64 {
    // Verify admin
    verify_admin(env, &admin);

    let rule_id = get_next_rule_id(env);

    let rule = AutoApprovalRule {
        rule_id,
        max_amount,
        diagnosis_codes,
        enabled: true,
    };

    env.storage()
        .persistent()
        .set(&storage_key_rule(rule_id), &rule);

    rule_id
}

pub fn update_auto_approval_rule(
    env: &Env,
    admin: Address,
    rule_id: u64,
    max_amount: Option<i128>,
    diagnosis_codes: Option<Vec<String>>,
    enabled: Option<bool>,
) {
    verify_admin(env, &admin);

    let mut rule = get_rule(env, rule_id).unwrap_or_else(|| panic!("Rule not found"));

    if let Some(amount) = max_amount {
        rule.max_amount = amount;
    }

    if let Some(codes) = diagnosis_codes {
        rule.diagnosis_codes = codes;
    }

    if let Some(is_enabled) = enabled {
        rule.enabled = is_enabled;
    }

    env.storage()
        .persistent()
        .set(&storage_key_rule(rule_id), &rule);
}

pub fn process_claim(env: &Env, claim_id: u64, insurer: Address) {
    insurer.require_auth();

    let mut claim = claims::get_claim(env, claim_id)
        .unwrap_or_else(|| panic!("Claim not found"));

    // Verify insurer
    if claim.insurer != insurer {
        panic!("Unauthorized insurer");
    }

    // Check if claim is in correct status
    if claim.status != ClaimStatus::Submitted {
        panic!("Claim not in submitted status");
    }

    // Update status to under review
    claim.status = ClaimStatus::UnderReview;
    claims::update_claim_status(
        env,
        claim_id,
        ClaimStatus::UnderReview,
        None,
        None,
        None,
    );

    // Check auto-approval rules
    if check_auto_approval(env, &claim) {
        // Auto-approve claim
        claims::update_claim_status(
            env,
            claim_id,
            ClaimStatus::Approved,
            Some(claim.claim_amount),
            None,
            Some(true),
        );

        // Remove from pending queue
        claims::remove_from_pending_queue(env, claim_id);
    }
}

pub fn manually_approve_claim(
    env: &Env,
    claim_id: u64,
    insurer: Address,
    approved_amount: i128,
) {
    insurer.require_auth();

    let claim = claims::get_claim(env, claim_id)
        .unwrap_or_else(|| panic!("Claim not found"));

    // Verify insurer
    if claim.insurer != insurer {
        panic!("Unauthorized insurer");
    }

    // Verify claim is under review
    if claim.status != ClaimStatus::UnderReview {
        panic!("Claim not under review");
    }

    // Approve claim
    claims::update_claim_status(
        env,
        claim_id,
        ClaimStatus::Approved,
        Some(approved_amount),
        None,
        None,
    );

    // Remove from pending queue
    claims::remove_from_pending_queue(env, claim_id);
}

pub fn reject_claim(
    env: &Env,
    claim_id: u64,
    insurer: Address,
    rejection_reason: String,
) {
    insurer.require_auth();

    let claim = claims::get_claim(env, claim_id)
        .unwrap_or_else(|| panic!("Claim not found"));

    // Verify insurer
    if claim.insurer != insurer {
        panic!("Unauthorized insurer");
    }

    // Verify claim is under review
    if claim.status != ClaimStatus::UnderReview {
        panic!("Claim not under review");
    }

    // Reject claim
    claims::update_claim_status(
        env,
        claim_id,
        ClaimStatus::Rejected,
        None,
        Some(rejection_reason),
        None,
    );

    // Remove from pending queue
    claims::remove_from_pending_queue(env, claim_id);
}

pub fn execute_payout(
    env: &Env,
    claim_id: u64,
    insurer: Address,
) {
    insurer.require_auth();

    let claim = claims::get_claim(env, claim_id)
        .unwrap_or_else(|| panic!("Claim not found"));

    // Verify insurer
    if claim.insurer != insurer {
        panic!("Unauthorized insurer");
    }

    // Verify claim is approved
    if claim.status != ClaimStatus::Approved {
        panic!("Claim not approved");
    }

    // In a real implementation, this would transfer tokens
    // For now, we just record the payout

    let payout = PayoutRecord {
        claim_id,
        recipient: claim.patient.clone(),
        amount: claim.approved_amount,
        currency: claim.currency.clone(),
        paid_at: env.ledger().timestamp(),
        transaction_hash: None,
    };

    env.storage()
        .persistent()
        .set(&storage_key_payout(claim_id), &payout);

    // Update claim status to paid
    claims::update_claim_status(
        env,
        claim_id,
        ClaimStatus::Paid,
        None,
        None,
        None,
    );
}

pub fn get_rule(env: &Env, rule_id: u64) -> Option<AutoApprovalRule> {
    env.storage()
        .persistent()
        .get(&storage_key_rule(rule_id))
}

pub fn get_payout(env: &Env, claim_id: u64) -> Option<PayoutRecord> {
    env.storage()
        .persistent()
        .get(&storage_key_payout(claim_id))
}

fn check_auto_approval(env: &Env, claim: &Claim) -> bool {
    // Get all rules and check if claim matches any
    // For simplicity, we'll check the default rule (rule_id: 0)
    let rule = match get_rule(env, 0) {
        Some(r) => r,
        None => return false,
    };

    if !rule.enabled {
        return false;
    }

    // Check amount threshold
    if claim.claim_amount > rule.max_amount {
        return false;
    }

    // Check diagnosis codes if specified
    if rule.diagnosis_codes.len() > 0 {
        let mut matches = false;
        for i in 0..rule.diagnosis_codes.len() {
            let code = rule.diagnosis_codes.get(i).unwrap();
            if code == claim.diagnosis_code {
                matches = true;
                break;
            }
        }
        if !matches {
            return false;
        }
    }

    true
}

fn verify_admin(env: &Env, admin: &Address) {
    admin.require_auth();

    let stored_admin: Address = env
        .storage()
        .instance()
        .get(&claims::ADMIN)
        .unwrap_or_else(|| panic!("No admin set"));

    if stored_admin != *admin {
        panic!("Unauthorized admin");
    }
}

fn get_next_rule_id(env: &Env) -> u64 {
    let current_id: u64 = env
        .storage()
        .instance()
        .get(&NEXT_RULE_ID)
        .unwrap_or(1);

    env.storage().instance().set(&NEXT_RULE_ID, &(current_id + 1));

    current_id
}

fn storage_key_rule(rule_id: u64) -> (Symbol, u64) {
    (AUTO_RULES, rule_id)
}

fn storage_key_payout(claim_id: u64) -> (Symbol, u64) {
    (PAYOUTS, claim_id)
}
