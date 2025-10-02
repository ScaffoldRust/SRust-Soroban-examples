use soroban_sdk::{contracterror, Address, Bytes, Env};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[repr(u32)]
pub enum UtilsError {
    InvalidAddress = 1,
    InvalidOracleData = 2,
    MathOverflow = 3,
}

pub mod safe_math {
    use super::UtilsError;
    use core::result::Result;

    pub fn mul_i128(a: i128, b: i128, scale: i128) -> Result<i128, UtilsError> {
        let product = a.checked_mul(b).ok_or(UtilsError::MathOverflow)?;
        let scaled = product.checked_div(scale).ok_or(UtilsError::MathOverflow)?;
        Ok(scaled)
    }

    pub fn div_i128(a: i128, b: i128) -> Result<i128, UtilsError> {
        if b == 0 {
            return Err(UtilsError::MathOverflow);
        }
        a.checked_div(b).ok_or(UtilsError::MathOverflow)
    }

    pub fn add_i128(a: i128, b: i128) -> Result<i128, UtilsError> {
        a.checked_add(b).ok_or(UtilsError::MathOverflow)
    }

    pub fn sub_i128(a: i128, b: i128) -> Result<i128, UtilsError> {
        a.checked_sub(b).ok_or(UtilsError::MathOverflow)
    }
}

pub fn validate_address(_env: &Env, _addr: Address) {
    // Address is always valid in Soroban
}

pub fn validate_oracle_data(
    _env: &Env,
    oracle_data: Bytes,
    _event_id: u64,
    _consumer: Address,
    reduction: i128,
) {
    // Placeholder: In production, verify signature in oracle_data
    // For example, parse oracle_data for signature, timestamp, etc.
    if oracle_data.len() == 0 || reduction <= 0 {
        panic!("{}", crate::ContractError::InvalidInput as u32);
    }
    // Mock validation: assume valid if data is non-empty
    // Real impl: use crypto primitives if available, or cross-contract to oracle
}

pub fn generate_event_id(env: &Env) -> u64 {
    let key = crate::DataKey::EventCount;
    let count: u64 = env.storage().instance().get(&key).unwrap_or(0u64);
    let new_count = count + 1;
    env.storage().instance().set(&key, &new_count);
    new_count
}

pub fn is_operator(env: &Env, caller: Address) -> bool {
    let operators: soroban_sdk::Map<Address, bool> = env
        .storage()
        .instance()
        .get(&crate::DataKey::Operators)
        .unwrap_or_else(|| soroban_sdk::Map::new(env));
    operators.get(caller).unwrap_or(false)
}

pub fn is_consumer(env: &Env, caller: Address) -> bool {
    let consumers: soroban_sdk::Map<Address, bool> = env
        .storage()
        .instance()
        .get(&crate::DataKey::Consumers)
        .unwrap_or_else(|| soroban_sdk::Map::new(env));
    consumers.get(caller).unwrap_or(false)
}
