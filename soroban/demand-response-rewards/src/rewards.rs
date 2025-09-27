use soroban_sdk::{contracterror, Env, Address, Map, Vec, symbol_short};

use crate::Participation;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[repr(u32)]
pub enum RewardError {
    NotAuthorized = 1,
    EventNotFound = 2,
    EventNotCompleted = 3,
    NoParticipation = 4,
    InsufficientRewards = 5,
    MathError = 6,
}



pub fn calculate_total_reduction(env: &Env, event_id: u64) -> i128 {
    let participations: Map<Address, Vec<Participation>> = env.storage().instance().get(&crate::DataKey::Participations).unwrap_or_else(|| Map::new(env));
    let mut total = 0i128;
    for (_consumer, parts) in participations.iter() {
        for part in parts.iter() {
            if part.event_id == event_id && part.status == symbol_short!("verif") {
                total += part.reduction;
            }
        }
    }
    total
}



// Safe multiplication with overflow check
pub fn safe_mul(a: i128, b: i128) -> Result<i128, RewardError> {
    let product = a.checked_mul(b).ok_or(RewardError::MathError)?;
    Ok(product)
}

// Safe division
pub fn safe_div(a: i128, b: i128) -> Result<i128, RewardError> {
    if b == 0 {
        return Err(RewardError::MathError);
    }
    let quotient = a.checked_div(b).ok_or(RewardError::MathError)?;
    Ok(quotient)
}
