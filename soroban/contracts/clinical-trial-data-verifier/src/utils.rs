use soroban_sdk::{Env, String};

// Minimal hash validation used by verifier module.
pub fn validate_hash(_env: &Env, hash: &String) -> bool { hash.len() == 64 }