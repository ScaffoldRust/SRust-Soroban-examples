use crate::TokenInfo;
use soroban_sdk::{symbol_short, Address, Env};

const BALANCE: soroban_sdk::Symbol = symbol_short!("BALANCE");
const ALLOWANCE: soroban_sdk::Symbol = symbol_short!("ALLOW");
const TOKEN: soroban_sdk::Symbol = symbol_short!("TOKEN");

pub fn balance_of(env: &Env, address: &Address, token_id: u64) -> i128 {
    let key = (BALANCE, address.clone(), token_id);
    env.storage().persistent().get(&key).unwrap_or(0)
}

pub fn set_balance(env: &Env, address: &Address, token_id: u64, amount: i128) {
    let key = (BALANCE, address.clone(), token_id);
    env.storage().persistent().set(&key, &amount);
}

pub fn allowance(env: &Env, owner: &Address, spender: &Address, token_id: u64) -> i128 {
    let key = (ALLOWANCE, owner.clone(), spender.clone(), token_id);
    env.storage().persistent().get(&key).unwrap_or(0)
}

pub fn set_allowance(env: &Env, owner: &Address, spender: &Address, token_id: u64, amount: i128) {
    let key = (ALLOWANCE, owner.clone(), spender.clone(), token_id);
    env.storage().persistent().set(&key, &amount);
}

pub fn token_info(env: &Env, token_id: u64) -> Option<TokenInfo> {
    let key = (TOKEN, token_id);
    env.storage().persistent().get(&key)
}

pub fn set_token_info(env: &Env, token_id: u64, info: &TokenInfo) {
    let key = (TOKEN, token_id);
    env.storage().persistent().set(&key, info);
}

pub fn add_balance(env: &Env, address: &Address, token_id: u64, amount: i128) {
    let current = balance_of(env, address, token_id);
    set_balance(env, address, token_id, current + amount);
}

pub fn subtract_balance(env: &Env, address: &Address, token_id: u64, amount: i128) -> bool {
    let current = balance_of(env, address, token_id);
    if current >= amount {
        set_balance(env, address, token_id, current - amount);
        true
    } else {
        false
    }
}
