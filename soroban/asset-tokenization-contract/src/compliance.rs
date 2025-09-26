use soroban_sdk::{Address, Env, symbol_short};
use crate::{ComplianceStatus, admin};

const COMPLIANCE: soroban_sdk::Symbol = symbol_short!("COMPLY");

pub fn verify_compliance(env: &Env, token_id: u64, user: &Address) -> ComplianceStatus {
    let key = (COMPLIANCE, token_id, user.clone());
    env.storage().persistent().get(&key).unwrap_or(ComplianceStatus::Pending)
}

pub fn update_compliance_status(
    env: &Env,
    admin_addr: &Address,
    token_id: u64,
    user: &Address,
    status: ComplianceStatus,
) -> bool {
    admin_addr.require_auth();
    
    // Verify admin privileges
    if !admin::is_admin(env, admin_addr) {
        panic!("Unauthorized: Only admins can update compliance status");
    }
    
    let key = (COMPLIANCE, token_id, user.clone());
    env.storage().persistent().set(&key, &status);
    
    // Emit event - shortened to max 9 chars
    env.events().publish(
        (symbol_short!("comply"), admin_addr.clone()),
        (user.clone(), token_id, status.clone())
    );
    
    true
}

pub fn set_initial_compliance(env: &Env, token_id: u64, user: &Address, status: ComplianceStatus) {
    let key = (COMPLIANCE, token_id, user.clone());
    env.storage().persistent().set(&key, &status);
}