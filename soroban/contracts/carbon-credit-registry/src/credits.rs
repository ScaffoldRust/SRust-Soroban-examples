use soroban_sdk::{Env, Address, BytesN, String, Map, Vec, Bytes};

use crate::{
    CarbonCredit, CreditStatus, CreditEvent, EventType, DataKey,
};

/// Record a credit event in the history
pub fn record_credit_event(
    env: &Env,
    credit_id: &BytesN<32>,
    event_type: EventType,
    from: Option<Address>,
    to: Option<Address>,
    quantity: i128,
) {
    let event_count: u32 = env.storage().instance().get(&DataKey::EventCount(credit_id.clone()))
        .unwrap_or(0);

    let event = CreditEvent {
        event_type,
        credit_id: credit_id.clone(),
        timestamp: env.ledger().timestamp(),
        from,
        to,
        quantity,
        transaction_hash: BytesN::from_array(env, &[0u8; 32]),
        metadata: Map::new(env),
    };

    env.storage().instance().set(&DataKey::CreditEvent(credit_id.clone(), event_count), &event);
    env.storage().instance().set(&DataKey::EventCount(credit_id.clone()), &(event_count + 1));
}

/// Get credit status and details
pub fn get_credit_status(
    env: &Env,
    credit_id: BytesN<32>,
) -> Option<CarbonCredit> {
    env.storage().instance().get(&DataKey::Credit(credit_id))
}

/// Get credit transaction history
pub fn get_credit_history(
    env: &Env,
    credit_id: BytesN<32>,
) -> Vec<CreditEvent> {
    let event_count: u32 = env.storage().instance().get(&DataKey::EventCount(credit_id.clone()))
        .unwrap_or(0);

    let mut events = Vec::new(env);
    for i in 0..event_count {
        if let Some(event) = env.storage().instance().get::<DataKey, crate::CreditEvent>(&DataKey::CreditEvent(credit_id.clone(), i)) {
            events.push_back(event);
        }
    }

    events
}