use soroban_sdk::{Env, BytesN, String, Map, Vec, Bytes};

use crate::{
    CarbonCredit, CreditStatus, CreditEvent, EventType, DataKey,
    TradingParams, RetirementParams,
};

/// Trade a carbon credit from one owner to another
pub fn trade_credit(
    env: Env,
    params: TradingParams,
) {
    // Validate that the sender is authorized
    params.from.require_auth();

    // Get the credit
    let mut credit: CarbonCredit = env.storage().instance().get(&DataKey::Credit(params.credit_id.clone()))
        .unwrap();

    // Update credit ownership
    credit.current_owner = params.to.clone();
    credit.status = CreditStatus::Traded;

    env.storage().instance().set(&DataKey::Credit(params.credit_id.clone()), &credit);

    // Record trade event
    record_trade_event(&env, &params);
}

/// Retire a carbon credit for carbon offset claims
pub fn retire_credit(
    env: Env,
    params: RetirementParams,
) {
    // Validate that the owner is authorized
    params.owner.require_auth();

    // Get the credit
    let mut credit: CarbonCredit = env.storage().instance().get(&DataKey::Credit(params.credit_id.clone()))
        .unwrap();

    // Update credit status
    credit.quantity -= params.quantity;
    if credit.quantity == 0 {
        credit.status = CreditStatus::Retired;
    }

    env.storage().instance().set(&DataKey::Credit(params.credit_id.clone()), &credit);

    // Update contract statistics
    update_retirement_stats(&env, params.quantity);

    // Record retirement event
    record_retirement_event(&env, &params);
}

/// Get trading history for a credit
pub fn get_trading_history(
    env: Env,
    credit_id: BytesN<32>,
) -> Vec<CreditEvent> {
    let event_count: u32 = env.storage().instance().get(&DataKey::EventCount(credit_id.clone()))
        .unwrap_or(0);

    let mut trading_events = Vec::new(&env);
    for i in 0..event_count {
        if let Some(event) = env.storage().instance().get::<DataKey, crate::CreditEvent>(&DataKey::CreditEvent(credit_id.clone(), i)) {
            match event.event_type {
                EventType::Trade | EventType::Retirement => {
                    trading_events.push_back(event);
                }
                _ => {}
            }
        }
    }

    trading_events
}

/// Record a trade event
fn record_trade_event(
    env: &Env,
    params: &TradingParams,
) {
    let event_count: u32 = env.storage().instance().get(&DataKey::EventCount(params.credit_id.clone()))
        .unwrap_or(0);

    let mut metadata = Map::new(env);
    metadata.set(String::from_str(env, "price"), String::from_str(env, "50")); // Simplified for now
    metadata.set(String::from_str(env, "payment_token"), String::from_str(env, "token"));

    let event = CreditEvent {
        event_type: EventType::Trade,
        credit_id: params.credit_id.clone(),
        timestamp: env.ledger().timestamp(),
        from: Some(params.from.clone()),
        to: Some(params.to.clone()),
        quantity: params.quantity,
        transaction_hash: BytesN::from_array(env, &[0u8; 32]),
        metadata,
    };

    env.storage().instance().set(&DataKey::CreditEvent(params.credit_id.clone(), event_count), &event);
    env.storage().instance().set(&DataKey::EventCount(params.credit_id.clone()), &(event_count + 1));
}

/// Record a retirement event
fn record_retirement_event(
    env: &Env,
    params: &RetirementParams,
) {
    let event_count: u32 = env.storage().instance().get(&DataKey::EventCount(params.credit_id.clone()))
        .unwrap_or(0);

    let mut metadata = Map::new(env);
    metadata.set(String::from_str(env, "reason"), params.retirement_reason.clone());
    metadata.set(String::from_str(env, "certificate"), String::from_str(env, "cert"));

    let event = CreditEvent {
        event_type: EventType::Retirement,
        credit_id: params.credit_id.clone(),
        timestamp: env.ledger().timestamp(),
        from: Some(params.owner.clone()),
        to: None,
        quantity: params.quantity,
        transaction_hash: BytesN::from_array(env, &[0u8; 32]),
        metadata,
    };

    env.storage().instance().set(&DataKey::CreditEvent(params.credit_id.clone(), event_count), &event);
    env.storage().instance().set(&DataKey::EventCount(params.credit_id.clone()), &(event_count + 1));
}

/// Update retirement statistics
fn update_retirement_stats(
    env: &Env,
    retired_quantity: i128,
) {
    let mut total_retired: i128 = env.storage().instance().get(&DataKey::TotalCreditsRetired)
        .unwrap_or(0);

    total_retired += retired_quantity;
    env.storage().instance().set(&DataKey::TotalCreditsRetired, &total_retired);
}