use soroban_sdk::{Address, BytesN, Env, Map, String, Vec};

use crate::{
    utils, ContractStats, DataKey, EnergySource, EventType, IssuerInfo, RECEvent, RECStatus, REC,
};

/// Issues a new REC for verified renewable energy production
pub fn issue_rec(
    env: &Env,
    issuer: Address,
    energy_source: EnergySource,
    production_date: u64,
    production_location: String,
    capacity_mwh: i128,
    verification_standard: String,
    verification_hash: BytesN<32>,
    metadata: Map<String, String>,
) -> BytesN<32> {
    // Verify issuer is authorized
    issuer.require_auth();

    let issuer_key = DataKey::Issuer(issuer.clone());
    let mut issuer_info: IssuerInfo = env
        .storage()
        .persistent()
        .get(&issuer_key)
        .expect("Issuer not registered");

    if !issuer_info.authorized {
        panic!("Issuer not authorized");
    }

    // Validate capacity
    if capacity_mwh <= 0 {
        panic!("Capacity must be positive");
    }

    // Generate unique REC ID
    let rec_id = utils::generate_rec_id(env, &issuer, &verification_hash);

    // Check if REC already exists
    let rec_key = DataKey::REC(rec_id.clone());
    if env.storage().persistent().has(&rec_key) {
        panic!("REC already exists");
    }

    // Create the REC
    let rec = REC {
        id: rec_id.clone(),
        issuer: issuer.clone(),
        energy_source,
        production_date,
        production_location,
        capacity_mwh,
        current_owner: issuer.clone(),
        status: RECStatus::Issued,
        verification_standard,
        verification_hash,
        issuance_date: env.ledger().timestamp(),
        metadata,
    };

    // Store the REC
    env.storage().persistent().set(&rec_key, &rec);

    // Create issuance event
    let event = RECEvent {
        event_type: EventType::Issuance,
        rec_id: rec_id.clone(),
        timestamp: env.ledger().timestamp(),
        from: None,
        to: Some(issuer.clone()),
        capacity_mwh,
        transaction_hash: BytesN::from_array(env, &[1u8; 32]),
        notes: String::from_str(env, "REC issued"),
    };

    // Store event in history
    let history_key = DataKey::RecHistory(rec_id.clone());
    let mut history: Vec<RECEvent> = Vec::new(env);
    history.push_back(event);
    env.storage().persistent().set(&history_key, &history);

    // Update issuer statistics
    issuer_info.total_issued += capacity_mwh;
    env.storage().persistent().set(&issuer_key, &issuer_info);

    // Update contract statistics
    let mut stats: ContractStats = env.storage().instance().get(&DataKey::Stats).unwrap();
    stats.total_recs_issued += 1;
    stats.total_capacity_issued_mwh += capacity_mwh;
    env.storage().instance().set(&DataKey::Stats, &stats);

    rec_id
}

/// Validates REC authenticity and ownership
pub fn verify_rec(env: &Env, rec_id: BytesN<32>) -> bool {
    let rec_key = DataKey::REC(rec_id.clone());

    if !env.storage().persistent().has(&rec_key) {
        return false;
    }

    let rec: REC = env.storage().persistent().get(&rec_key).unwrap();

    // Verify issuer is still authorized
    let issuer_key = DataKey::Issuer(rec.issuer.clone());
    if let Some(issuer_info) = env
        .storage()
        .persistent()
        .get::<DataKey, IssuerInfo>(&issuer_key)
    {
        if !issuer_info.authorized {
            return false;
        }
    } else {
        return false;
    }

    // Verify REC is not suspended
    if rec.status == RECStatus::Suspended {
        return false;
    }

    true
}

/// Gets detailed information about a REC
pub fn get_rec_details(env: &Env, rec_id: BytesN<32>) -> REC {
    let rec_key = DataKey::REC(rec_id);
    env.storage()
        .persistent()
        .get(&rec_key)
        .expect("REC not found")
}

/// Checks if an address is the current owner of a REC
pub fn is_owner(env: &Env, rec_id: BytesN<32>, address: &Address) -> bool {
    let rec_key = DataKey::REC(rec_id);
    if let Some(rec) = env.storage().persistent().get::<DataKey, REC>(&rec_key) {
        rec.current_owner == *address
    } else {
        false
    }
}
