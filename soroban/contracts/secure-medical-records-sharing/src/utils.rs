use soroban_sdk::Env;

pub fn now(env: &Env) -> u64 { env.ledger().timestamp() }

// (push_capped removed; audit capping handled inline for type simplicity)
