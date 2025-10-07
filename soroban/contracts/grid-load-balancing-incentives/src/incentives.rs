use crate::utils::*;
use soroban_sdk::{symbol_short, token, Address, Env, Map, Vec};

/// Distribute rewards to participating consumers
pub fn distribute_rewards(
    env: &Env,
    event_id: u64,
    distributor: Address,
) -> Result<(), IncentiveError> {
    let events: Map<u64, DemandResponseEvent> = env
        .storage()
        .instance()
        .get(&DataKey::Events)
        .unwrap_or_else(|| Map::new(env));

    let event = events.get(event_id).ok_or(IncentiveError::EventNotFound)?;

    // Verify authorization (grid operator or admin can distribute)
    let is_grid_operator = event.grid_operator == distributor;
    let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
    let is_admin = admin == distributor;

    if !is_grid_operator && !is_admin {
        return Err(IncentiveError::NotAuthorized);
    }

    // Event must be completed or expired
    if event.status == EventStatus::Active {
        let current_time = env.ledger().timestamp();
        if current_time <= event.end_time {
            return Err(IncentiveError::EventNotActive);
        }
    }

    // Get all verified participations for this event
    let mut participations: Map<u64, ParticipationRecord> = env
        .storage()
        .instance()
        .get(&DataKey::Participations)
        .unwrap_or_else(|| Map::new(env));

    let mut total_rewards_distributed = 0u64;
    let mut participants_rewarded = 0u64;

    // Process all participations for this event
    for (participation_id, mut participation) in participations.iter() {
        if participation.event_id == event_id
            && participation.status == ParticipationStatus::Verified
        {
            // Execute reward payment
            match execute_reward_payment(env, &participation) {
                Ok(_) => {
                    participation.status = ParticipationStatus::Rewarded;
                    participations.set(participation_id, participation.clone());
                    total_rewards_distributed += participation.reward_amount;
                    participants_rewarded += 1;
                }
                Err(_) => {
                    participation.status = ParticipationStatus::Rejected;
                    participations.set(participation_id, participation);
                }
            }
        }
    }

    // Update storage
    env.storage()
        .instance()
        .set(&DataKey::Participations, &participations);

    // Emit rewards distributed event
    env.events().publish(
        (symbol_short!("rewarded"), event_id),
        (
            participants_rewarded,
            total_rewards_distributed,
            distributor,
        ),
    );

    Ok(())
}

/// Execute reward payment using Stellar tokens
fn execute_reward_payment(
    env: &Env,
    participation: &ParticipationRecord,
) -> Result<(), IncentiveError> {
    // Get token contract
    let token_contract: Address = env
        .storage()
        .instance()
        .get(&DataKey::TokenContract)
        .ok_or(IncentiveError::RewardDistributionFailed)?;

    // Get admin as the source of rewards
    let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();

    // Execute transfer from admin to consumer
    // Note: This requires admin to have authorized the contract to transfer tokens
    let token_client = token::Client::new(env, &token_contract);
    token_client.transfer(
        &admin,
        &participation.consumer,
        &(participation.reward_amount as i128),
    );

    // Emit payment event
    env.events().publish(
        (symbol_short!("payment"), participation.consumer.clone()),
        (participation.participation_id, participation.reward_amount),
    );

    Ok(())
}

/// Get participation record details
pub fn get_participation(
    env: &Env,
    participation_id: u64,
) -> Result<ParticipationRecord, IncentiveError> {
    let participations: Map<u64, ParticipationRecord> = env
        .storage()
        .instance()
        .get(&DataKey::Participations)
        .unwrap_or_else(|| Map::new(env));

    participations
        .get(participation_id)
        .ok_or(IncentiveError::ParticipationNotFound)
}

/// Get participation history for a consumer
pub fn get_consumer_participations(
    env: &Env,
    consumer: Address,
) -> Result<Vec<ParticipationRecord>, IncentiveError> {
    let participations: Map<u64, ParticipationRecord> = env
        .storage()
        .instance()
        .get(&DataKey::Participations)
        .unwrap_or_else(|| Map::new(env));

    let mut consumer_participations = Vec::new(env);
    for (_, participation) in participations.iter() {
        if participation.consumer == consumer {
            consumer_participations.push_back(participation);
        }
    }

    Ok(consumer_participations)
}

/// Audit reward distributions for transparency
pub fn audit_event_rewards(
    env: &Env,
    event_id: u64,
) -> Result<Vec<ParticipationRecord>, IncentiveError> {
    let participations: Map<u64, ParticipationRecord> = env
        .storage()
        .instance()
        .get(&DataKey::Participations)
        .unwrap_or_else(|| Map::new(env));

    let mut event_participations = Vec::new(env);
    for (_, participation) in participations.iter() {
        if participation.event_id == event_id {
            event_participations.push_back(participation);
        }
    }

    Ok(event_participations)
}

/// Reject a participation (for disputed meter readings)
pub fn reject_participation(
    env: &Env,
    participation_id: u64,
    rejector: Address,
) -> Result<(), IncentiveError> {
    let mut participations: Map<u64, ParticipationRecord> = env
        .storage()
        .instance()
        .get(&DataKey::Participations)
        .unwrap_or_else(|| Map::new(env));

    let mut participation = participations
        .get(participation_id)
        .ok_or(IncentiveError::ParticipationNotFound)?;

    // Only admin or grid operator can reject
    let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();

    let events: Map<u64, DemandResponseEvent> = env
        .storage()
        .instance()
        .get(&DataKey::Events)
        .unwrap_or_else(|| Map::new(env));

    let event = events
        .get(participation.event_id)
        .ok_or(IncentiveError::EventNotFound)?;

    if rejector != admin && rejector != event.grid_operator {
        return Err(IncentiveError::NotAuthorized);
    }

    participation.status = ParticipationStatus::Rejected;
    participations.set(participation_id, participation);
    env.storage()
        .instance()
        .set(&DataKey::Participations, &participations);

    env.events()
        .publish((symbol_short!("rejected"), participation_id), rejector);

    Ok(())
}
