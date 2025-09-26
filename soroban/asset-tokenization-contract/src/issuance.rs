use soroban_sdk::{Address, Env, symbol_short};
use crate::{AssetMetadata, RegulatoryInfo, TokenInfo, admin, token};

const NEXT_TOKEN_ID: soroban_sdk::Symbol = symbol_short!("NEXTID"); // Shortened from "NEXT_ID"

pub fn tokenize(
    env: &Env,
    issuer: &Address,
    asset_metadata: AssetMetadata,
    regulatory_info: RegulatoryInfo,
    initial_amount: i128,
) -> u64 {
    issuer.require_auth();
    
    // Check if issuer is not frozen
    if admin::is_frozen(env, issuer) {
        panic!("Issuer account is frozen");
    }

    // Get next token ID
    let token_id = get_next_token_id(env);
    
    // Create token info
    let token_info = TokenInfo {
        metadata: asset_metadata,
        regulatory_info,
        total_supply: initial_amount,
        issuer: issuer.clone(),
    };
    
    // Store token info
    token::set_token_info(env, token_id, &token_info);
    
    // Set initial balance for issuer
    token::set_balance(env, issuer, token_id, initial_amount);
    
    // Emit event
    env.events().publish(
        (symbol_short!("tokenize"), issuer.clone()),
        (token_id, initial_amount)
    );
    
    token_id
}

pub fn redeem(env: &Env, caller: &Address, token_id: u64, amount: i128) -> bool {
    caller.require_auth();
    
    // Check if account is not frozen
    if admin::is_frozen(env, caller) {
        panic!("Account is frozen");
    }
    
    // Check if token exists
    let mut token_info = token::token_info(env, token_id).expect("Token not found");
    
    // Check if caller has enough balance
    if !token::subtract_balance(env, caller, token_id, amount) {
        panic!("Insufficient balance");
    }
    
    // Update total supply
    token_info.total_supply -= amount;
    token::set_token_info(env, token_id, &token_info);
    
    // Emit event
    env.events().publish(
        (symbol_short!("redeem"), caller.clone()),
        (token_id, amount)
    );
    
    true
}

fn get_next_token_id(env: &Env) -> u64 {
    let current_id: u64 = env.storage().persistent().get(&NEXT_TOKEN_ID).unwrap_or(1);
    let next_id = current_id + 1;
    env.storage().persistent().set(&NEXT_TOKEN_ID, &next_id);
    current_id
}