use crate::utils::*;
use soroban_sdk::{symbol_short, Address, Env, Map};

/// Start a demand response event
pub fn start_event(
    env: &Env,
    grid_operator: Address,
    target_reduction_kw: u64,
    reward_per_kw: u64,
    duration_seconds: u64,
) -> Result<u64, IncentiveError> {
    let event_id = get_next_event_id(env);
    let current_time = env.ledger().timestamp();
    let end_time = current_time + duration_seconds;

    let event = DemandResponseEvent {
        event_id,
        grid_operator: grid_operator.clone(),
        target_reduction_kw,
        reward_per_kw,
        start_time: current_time,
        end_time,
        status: EventStatus::Active,
        total_participants: 0,
        total_reduction_achieved: 0,
    };

    // Store event
    let mut events: Map<u64, DemandResponseEvent> = env
        .storage()
        .instance()
        .get(&DataKey::Events)
        .unwrap_or_else(|| Map::new(env));

    events.set(event_id, event);
    env.storage().instance().set(&DataKey::Events, &events);

    // Emit event started event
    env.events().publish(
        (symbol_short!("event"), event_id),
        (
            grid_operator,
            target_reduction_kw,
            reward_per_kw,
            duration_seconds,
        ),
    );

    Ok(event_id)
}

/// Verify consumer load reduction and create participation record
pub fn verify_reduction(
    env: &Env,
    event_id: u64,
    consumer: Address,
    baseline_usage_kw: u64,
    actual_usage_kw: u64,
    meter_reading_start: u64,
    meter_reading_end: u64,
) -> Result<u64, IncentiveError> {
    let mut events: Map<u64, DemandResponseEvent> = env
        .storage()
        .instance()
        .get(&DataKey::Events)
        .unwrap_or_else(|| Map::new(env));

    let mut event = events.get(event_id).ok_or(IncentiveError::EventNotFound)?;

    // Validate event is active
    if event.status != EventStatus::Active {
        return Err(IncentiveError::EventNotActive);
    }

    let current_time = env.ledger().timestamp();
    if current_time > event.end_time {
        event.status = EventStatus::Completed;
        events.set(event_id, event);
        env.storage().instance().set(&DataKey::Events, &events);
        return Err(IncentiveError::EventExpired);
    }

    // Check if consumer already participating
    if is_consumer_participating(env, event_id, &consumer)? {
        return Err(IncentiveError::AlreadyParticipating);
    }

    // Calculate reduction achieved
    let reduction_achieved_kw = if baseline_usage_kw > actual_usage_kw {
        baseline_usage_kw - actual_usage_kw
    } else {
        0
    };

    // Require some minimum reduction
    if reduction_achieved_kw == 0 {
        return Err(IncentiveError::InsufficientReduction);
    }

    // Calculate reward
    let reward_amount = reduction_achieved_kw * event.reward_per_kw;

    // Create participation record
    let participation_id = generate_participation_id(env, event_id, &consumer);
    let participation = ParticipationRecord {
        participation_id,
        event_id,
        consumer: consumer.clone(),
        baseline_usage_kw,
        actual_usage_kw,
        reduction_achieved_kw,
        reward_amount,
        meter_reading_start,
        meter_reading_end,
        verified_at: current_time,
        status: ParticipationStatus::Verified,
    };

    // Store participation
    let mut participations: Map<u64, ParticipationRecord> = env
        .storage()
        .instance()
        .get(&DataKey::Participations)
        .unwrap_or_else(|| Map::new(env));

    participations.set(participation_id, participation);
    env.storage()
        .instance()
        .set(&DataKey::Participations, &participations);

    // Update event statistics
    event.total_participants += 1;
    event.total_reduction_achieved += reduction_achieved_kw;
    events.set(event_id, event);
    env.storage().instance().set(&DataKey::Events, &events);

    // Emit participation verified event
    env.events().publish(
        (symbol_short!("verified"), participation_id),
        (event_id, consumer, reduction_achieved_kw, reward_amount),
    );

    Ok(participation_id)
}

/// Get demand response event details
pub fn get_event(env: &Env, event_id: u64) -> Result<DemandResponseEvent, IncentiveError> {
    let events: Map<u64, DemandResponseEvent> = env
        .storage()
        .instance()
        .get(&DataKey::Events)
        .unwrap_or_else(|| Map::new(env));

    events.get(event_id).ok_or(IncentiveError::EventNotFound)
}

/// Complete an event (mark as completed)
pub fn complete_event(env: &Env, event_id: u64, grid_operator: Address) -> Result<(), IncentiveError> {
    let mut events: Map<u64, DemandResponseEvent> = env
        .storage()
        .instance()
        .get(&DataKey::Events)
        .unwrap_or_else(|| Map::new(env));

    let mut event = events.get(event_id).ok_or(IncentiveError::EventNotFound)?;

    // Verify authorization
    if event.grid_operator != grid_operator {
        return Err(IncentiveError::NotAuthorized);
    }

    if event.status != EventStatus::Active {
        return Err(IncentiveError::EventNotActive);
    }

    event.status = EventStatus::Completed;
    events.set(event_id, event);
    env.storage().instance().set(&DataKey::Events, &events);

    env.events()
        .publish((symbol_short!("completed"), event_id), grid_operator);

    Ok(())
}

// Helper functions
fn is_consumer_participating(env: &Env, event_id: u64, consumer: &Address) -> Result<bool, IncentiveError> {
    let participations: Map<u64, ParticipationRecord> = env
        .storage()
        .instance()
        .get(&DataKey::Participations)
        .unwrap_or_else(|| Map::new(env));

    for (_, participation) in participations.iter() {
        if participation.event_id == event_id && participation.consumer == *consumer {
            return Ok(true);
        }
    }
    Ok(false)
}

fn generate_participation_id(env: &Env, event_id: u64, consumer: &Address) -> u64 {
    // Simple ID generation based on event ID and consumer hash
    let consumer_hash = consumer.to_string().len() as u64;
    event_id * 10000 + consumer_hash + env.ledger().timestamp() % 1000
}