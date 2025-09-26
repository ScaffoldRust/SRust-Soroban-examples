use soroban_sdk::{Address, Env, String, symbol_short};
use crate::{token, AssetMetadata};

const ADMIN: soroban_sdk::Symbol = symbol_short!("ADMIN");
const FROZEN: soroban_sdk::Symbol = symbol_short!("FROZEN");

pub fn initialize(env: &Env, admin: &Address) {
    if env.storage().persistent().has(&ADMIN) {
        panic!("Contract already initialized");
    }
    
    env.storage().persistent().set(&ADMIN, admin);
}

pub fn is_admin(env: &Env, address: &Address) -> bool {
    let admin: Address = env.storage().persistent().get(&ADMIN).expect("Admin not set");
    admin == *address
}

pub fn update_asset_details(
    env: &Env,
    admin_addr: &Address,
    token_id: u64,
    new_metadata: AssetMetadata,
) -> bool {
    admin_addr.require_auth();
    
    if !is_admin(env, admin_addr) {
        panic!("Unauthorized: Only admins can update asset details");
    }
    
    // Get existing token info
    let mut token_info = token::token_info(env, token_id).expect("Token not found");
    
    // Update metadata
    token_info.metadata = new_metadata;
    
    // Save updated token info
    token::set_token_info(env, token_id, &token_info);
    
    // Emit event
    env.events().publish(
        (symbol_short!("update"), admin_addr.clone()),
        token_id
    );
    
    true
}

pub fn freeze_account(env: &Env, admin_addr: &Address, account: &Address, reason: String) -> bool {
    admin_addr.require_auth();
    
    if !is_admin(env, admin_addr) {
        panic!("Unauthorized: Only admins can freeze accounts");
    }
    
    let key = (FROZEN, account.clone());
    env.storage().persistent().set(&key, &reason);
    
    // Emit event
    env.events().publish(
        (symbol_short!("freeze"), admin_addr.clone()),
        (account.clone(), reason)
    );
    
    true
}

pub fn unfreeze_account(env: &Env, admin_addr: &Address, account: &Address) -> bool {
    admin_addr.require_auth();
    
    if !is_admin(env, admin_addr) {
        panic!("Unauthorized: Only admins can unfreeze accounts");
    }
    
    let key = (FROZEN, account.clone());
    env.storage().persistent().remove(&key);
    
    // Emit event
    env.events().publish(
        (symbol_short!("unfreeze"), admin_addr.clone()),
        account.clone()
    );
    
    true
}

pub fn is_frozen(env: &Env, account: &Address) -> bool {
    let key = (FROZEN, account.clone());
    env.storage().persistent().has(&key)
}