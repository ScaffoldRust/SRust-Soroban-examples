use soroban_sdk::{Address, BytesN, Env, String, Vec};

use crate::{ContractStats, DataKey, EventType, RECEvent, RECStatus, REC};

/// Transfers a REC to a new owner
pub fn transfer_rec(
    env: &Env,
    rec_id: BytesN<32>,
    from: Address,
    to: Address,
    capacity_mwh: i128,
) -> bool {
    // Require authentication from sender
    from.require_auth();

    // Get the REC
    let rec_key = DataKey::REC(rec_id.clone());
    let mut rec: REC = env
        .storage()
        .persistent()
        .get(&rec_key)
        .expect("REC not found");

    // Verify ownership
    if rec.current_owner != from {
        panic!("Not the owner");
    }

    // Verify REC is not retired or suspended
    if rec.status == RECStatus::Retired {
        panic!("Cannot transfer retired REC");
    }

    if rec.status == RECStatus::Suspended {
        panic!("Cannot transfer suspended REC");
    }

    // Verify capacity
    if capacity_mwh <= 0 {
        panic!("Capacity must be positive");
    }

    if capacity_mwh > rec.capacity_mwh {
        panic!("Insufficient capacity");
    }

    // Verify sender and recipient are different
    if from == to {
        panic!("Cannot transfer to self");
    }

    // Update REC ownership and status
    rec.current_owner = to.clone();
    rec.status = RECStatus::Transferred;
    env.storage().persistent().set(&rec_key, &rec);

    // Create transfer event
    let event = RECEvent {
        event_type: EventType::Transfer,
        rec_id: rec_id.clone(),
        timestamp: env.ledger().timestamp(),
        from: Some(from),
        to: Some(to),
        capacity_mwh,
        transaction_hash: BytesN::from_array(env, &[2u8; 32]),
        notes: String::from_str(env, "REC transferred"),
    };

    // Add event to history
    let history_key = DataKey::RecHistory(rec_id);
    let mut history: Vec<RECEvent> = env
        .storage()
        .persistent()
        .get(&history_key)
        .unwrap_or(Vec::new(env));
    history.push_back(event);
    env.storage().persistent().set(&history_key, &history);

    // Update contract statistics
    let mut stats: ContractStats = env.storage().instance().get(&DataKey::Stats).unwrap();
    stats.total_capacity_transferred_mwh += capacity_mwh;
    env.storage().instance().set(&DataKey::Stats, &stats);

    true
}

/// Retires a REC to claim renewable energy usage
pub fn retire_rec(
    env: &Env,
    rec_id: BytesN<32>,
    owner: Address,
    capacity_mwh: i128,
    retirement_reason: String,
) -> bool {
    // Require authentication from owner
    owner.require_auth();

    // Get the REC
    let rec_key = DataKey::REC(rec_id.clone());
    let mut rec: REC = env
        .storage()
        .persistent()
        .get(&rec_key)
        .expect("REC not found");

    // Verify ownership
    if rec.current_owner != owner {
        panic!("Not the owner");
    }

    // Verify REC is not already retired
    if rec.status == RECStatus::Retired {
        panic!("REC already retired");
    }

    // Verify REC is not suspended
    if rec.status == RECStatus::Suspended {
        panic!("Cannot retire suspended REC");
    }

    // Verify capacity
    if capacity_mwh <= 0 {
        panic!("Capacity must be positive");
    }

    if capacity_mwh > rec.capacity_mwh {
        panic!("Insufficient capacity");
    }

    // Update REC status
    rec.status = RECStatus::Retired;
    env.storage().persistent().set(&rec_key, &rec);

    // Create retirement event
    let event = RECEvent {
        event_type: EventType::Retirement,
        rec_id: rec_id.clone(),
        timestamp: env.ledger().timestamp(),
        from: Some(owner),
        to: None,
        capacity_mwh,
        transaction_hash: BytesN::from_array(env, &[3u8; 32]),
        notes: retirement_reason,
    };

    // Add event to history
    let history_key = DataKey::RecHistory(rec_id);
    let mut history: Vec<RECEvent> = env
        .storage()
        .persistent()
        .get(&history_key)
        .unwrap_or(Vec::new(env));
    history.push_back(event);
    env.storage().persistent().set(&history_key, &history);

    // Update contract statistics
    let mut stats: ContractStats = env.storage().instance().get(&DataKey::Stats).unwrap();
    stats.total_capacity_retired_mwh += capacity_mwh;
    env.storage().instance().set(&DataKey::Stats, &stats);

    true
}

/// Checks if a REC can be transferred
pub fn can_transfer(env: &Env, rec_id: BytesN<32>, owner: &Address) -> bool {
    let rec_key = DataKey::REC(rec_id);

    if let Some(rec) = env.storage().persistent().get::<DataKey, REC>(&rec_key) {
        rec.current_owner == *owner
            && rec.status != RECStatus::Retired
            && rec.status != RECStatus::Suspended
    } else {
        false
    }
}

/// Checks if a REC can be retired
pub fn can_retire(env: &Env, rec_id: BytesN<32>, owner: &Address) -> bool {
    let rec_key = DataKey::REC(rec_id);

    if let Some(rec) = env.storage().persistent().get::<DataKey, REC>(&rec_key) {
        rec.current_owner == *owner
            && rec.status != RECStatus::Retired
            && rec.status != RECStatus::Suspended
    } else {
        false
    }
}
