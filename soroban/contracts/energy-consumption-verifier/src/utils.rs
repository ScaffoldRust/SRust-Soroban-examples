use soroban_sdk::{Address, Env, String, Vec, Bytes};
use crate::{ContractError, DataKey, AuditLogEntry};

pub fn generate_record_id(env: &Env) -> u64 {
    let timestamp = env.ledger().timestamp();
    let sequence = env.ledger().sequence();

    let id = (timestamp << 32) | (sequence as u64 & 0xFFFFFFFF);
    if id == 0 {
        return 1;
    }
    id
}

pub fn compute_data_hash(
    env: &Env,
    _consumer: &Address,
    _meter_id: &String,
    consumption_kwh: u64,
    meter_reading: u64,
    timestamp: u64,
) -> Bytes {
    let mut data = [0u8; 24];
    data[0..8].copy_from_slice(&consumption_kwh.to_be_bytes());
    data[8..16].copy_from_slice(&meter_reading.to_be_bytes());
    data[16..24].copy_from_slice(&timestamp.to_be_bytes());

    let data_bytes = Bytes::from_slice(env, &data);
    env.crypto().sha256(&data_bytes).into()
}

pub fn log_audit_event(
    env: &Env,
    record_id: u64,
    action: String,
    actor: Address,
    details: Option<String>,
) -> Result<(), ContractError> {
    let mut audit_log: Vec<AuditLogEntry> = env
        .storage()
        .instance()
        .get(&DataKey::AuditLog)
        .unwrap_or_else(|| Vec::new(env));

    let audit_id = generate_audit_id(env);
    let timestamp = env.ledger().timestamp();

    let entry = AuditLogEntry {
        id: audit_id,
        record_id,
        action,
        actor,
        timestamp,
        details,
    };

    audit_log.push_back(entry);

    if audit_log.len() > 10000 {
        audit_log.pop_front();
    }

    env.storage().instance().set(&DataKey::AuditLog, &audit_log);

    Ok(())
}

pub fn get_audit_log(
    env: &Env,
    offset: u32,
    limit: u32,
) -> Result<Vec<AuditLogEntry>, ContractError> {
    let audit_log: Vec<AuditLogEntry> = env
        .storage()
        .instance()
        .get(&DataKey::AuditLog)
        .unwrap_or_else(|| Vec::new(env));

    let mut result = Vec::new(env);
    let start_index = offset;
    let end_index = (offset + limit).min(audit_log.len());

    for i in start_index..end_index {
        if let Some(entry) = audit_log.get(i) {
            result.push_back(entry);
        }
    }

    Ok(result)
}

pub fn generate_audit_id(env: &Env) -> u64 {
    let timestamp = env.ledger().timestamp();
    let sequence = env.ledger().sequence();

    ((timestamp as u128) * 1_000_000 + (sequence as u128)) as u64
}

pub fn validate_timestamp(env: &Env, timestamp: u64) -> Result<(), ContractError> {
    let current_time = env.ledger().timestamp();

    if timestamp > current_time {
        return Err(ContractError::InvalidTimestamp);
    }

    let max_age = 86400 * 7;
    if (current_time - timestamp) > max_age {
        return Err(ContractError::InvalidTimestamp);
    }

    Ok(())
}

pub fn calculate_energy_cost(
    _env: &Env,
    consumption_kwh: u64,
    rate_per_kwh: u64,
) -> Result<u64, ContractError> {
    consumption_kwh
        .checked_mul(rate_per_kwh)
        .ok_or(ContractError::InvalidInput)
}

pub fn format_consumption_data(
    env: &Env,
    _consumption_kwh: u64,
    _meter_reading: u64,
    _timestamp: u64,
) -> String {
    String::from_str(env, "CONSUMPTION_DATA")
}

pub fn validate_meter_id(meter_id: &String) -> Result<(), ContractError> {
    if meter_id.is_empty() {
        return Err(ContractError::InvalidInput);
    }

    Ok(())
}

pub fn is_within_normal_range(
    consumption_kwh: u64,
    previous_consumption: Option<u64>,
) -> bool {
    match previous_consumption {
        Some(prev) => {
            if prev == 0 {
                return consumption_kwh <= 1000;
            }

            let ratio = (consumption_kwh as f64) / (prev as f64);
            ratio >= 0.1 && ratio <= 10.0
        }
        None => consumption_kwh <= 1000,
    }
}

pub fn detect_anomaly(
    consumption_kwh: u64,
    temperature: Option<i32>,
    voltage: Option<u32>,
) -> bool {
    if consumption_kwh > 5000 {
        return true;
    }

    if let Some(temp) = temperature {
        if temp > 40 || temp < -10 {
            return true;
        }
    }

    if let Some(volt) = voltage {
        if volt > 245 || volt < 205 {
            return true;
        }
    }

    false
}

pub fn compress_consumption_data(
    env: &Env,
    _records: &Vec<(u64, u64, u64)>,
) -> Bytes {
    Bytes::from_slice(env, &[])
}

pub fn safe_add_u64(a: u64, b: u64) -> Result<u64, ContractError> {
    a.checked_add(b).ok_or(ContractError::InvalidInput)
}

pub fn safe_mul_u64(a: u64, b: u64) -> Result<u64, ContractError> {
    a.checked_mul(b).ok_or(ContractError::InvalidInput)
}

pub fn calculate_average_consumption(
    consumptions: &Vec<u64>,
) -> Result<u64, ContractError> {
    if consumptions.is_empty() {
        return Ok(0);
    }

    let mut total = 0u64;
    for consumption in consumptions.iter() {
        total = safe_add_u64(total, consumption)?;
    }

    Ok(total / (consumptions.len() as u64))
}