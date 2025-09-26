use crate::{admin, compliance, token, ComplianceStatus};
use soroban_sdk::{symbol_short, Address, Env};

pub fn transfer(env: &Env, from: &Address, to: &Address, token_id: u64, amount: i128) -> bool {
    from.require_auth();

    // Check if accounts are not frozen
    if admin::is_frozen(env, from) || admin::is_frozen(env, to) {
        panic!("One or both accounts are frozen");
    }

    // Check compliance for both parties
    let from_compliance = compliance::verify_compliance(env, token_id, from);
    let to_compliance = compliance::verify_compliance(env, token_id, to);

    match (from_compliance, to_compliance) {
        (ComplianceStatus::Approved, ComplianceStatus::Approved) => {}
        _ => panic!("Compliance check failed for one or both parties"),
    }

    // Execute transfer
    execute_transfer(env, from, to, token_id, amount)
}

pub fn approve(env: &Env, owner: &Address, spender: &Address, token_id: u64, amount: i128) -> bool {
    owner.require_auth();

    // Check if owner is not frozen
    if admin::is_frozen(env, owner) {
        panic!("Owner account is frozen");
    }

    token::set_allowance(env, owner, spender, token_id, amount);

    // Emit event
    env.events().publish(
        (symbol_short!("approve"), owner.clone()),
        (spender.clone(), token_id, amount),
    );

    true
}

pub fn transfer_from(
    env: &Env,
    spender: &Address,
    owner: &Address,
    receiver: &Address,
    token_id: u64,
    amount: i128,
) -> bool {
    spender.require_auth();

    // Check if accounts are not frozen
    if admin::is_frozen(env, owner)
        || admin::is_frozen(env, receiver)
        || admin::is_frozen(env, spender)
    {
        panic!("One or more accounts are frozen");
    }

    // Check allowance
    let current_allowance = token::allowance(env, owner, spender, token_id);
    if current_allowance < amount {
        panic!("Insufficient allowance");
    }

    // Check compliance
    let owner_compliance = compliance::verify_compliance(env, token_id, owner);
    let receiver_compliance = compliance::verify_compliance(env, token_id, receiver);

    match (owner_compliance, receiver_compliance) {
        (ComplianceStatus::Approved, ComplianceStatus::Approved) => {}
        _ => panic!("Compliance check failed for one or both parties"),
    }

    // Update allowance
    token::set_allowance(env, owner, spender, token_id, current_allowance - amount);

    // Execute transfer
    execute_transfer(env, owner, receiver, token_id, amount)
}

fn execute_transfer(env: &Env, from: &Address, to: &Address, token_id: u64, amount: i128) -> bool {
    // Check if token exists
    token::token_info(env, token_id).expect("Token not found");

    // Execute balance changes
    if !token::subtract_balance(env, from, token_id, amount) {
        panic!("Insufficient balance");
    }

    token::add_balance(env, to, token_id, amount);

    // Emit event
    env.events().publish(
        (symbol_short!("transfer"), from.clone()),
        (to.clone(), token_id, amount),
    );

    true
}
