use soroban_sdk::{Address, Env, Map, Vec, String};
use crate::{ContractError, DataKey, ConsumptionRecord, VerificationRecord, VerificationStatus};

pub fn store_consumption_record(
    env: &Env,
    record_id: u64,
    record: ConsumptionRecord,
) -> Result<(), ContractError> {
    let mut consumption_reports: Map<u64, ConsumptionRecord> = env
        .storage()
        .instance()
        .get(&DataKey::ConsumptionReports)
        .unwrap_or_else(|| Map::new(env));

    consumption_reports.set(record_id, record);
    env.storage().instance().set(&DataKey::ConsumptionReports, &consumption_reports);

    Ok(())
}

pub fn get_consumption_record(
    env: &Env,
    record_id: u64,
) -> Result<ConsumptionRecord, ContractError> {
    let consumption_reports: Map<u64, ConsumptionRecord> = env
        .storage()
        .instance()
        .get(&DataKey::ConsumptionReports)
        .unwrap_or_else(|| Map::new(env));

    consumption_reports
        .get(record_id)
        .ok_or(ContractError::RecordNotFound)
}

pub fn store_verification_record(
    env: &Env,
    record_id: u64,
    verification: VerificationRecord,
) -> Result<(), ContractError> {
    let mut verification_status: Map<u64, VerificationRecord> = env
        .storage()
        .instance()
        .get(&DataKey::VerificationStatus)
        .unwrap_or_else(|| Map::new(env));

    verification_status.set(record_id, verification);
    env.storage().instance().set(&DataKey::VerificationStatus, &verification_status);

    Ok(())
}

pub fn get_verification_record(
    env: &Env,
    record_id: u64,
) -> Result<VerificationRecord, ContractError> {
    let verification_status: Map<u64, VerificationRecord> = env
        .storage()
        .instance()
        .get(&DataKey::VerificationStatus)
        .unwrap_or_else(|| Map::new(env));

    verification_status
        .get(record_id)
        .ok_or(ContractError::RecordNotFound)
}

pub fn update_verification_status(
    env: &Env,
    record_id: u64,
    verification: VerificationRecord,
) -> Result<(), ContractError> {
    let mut verification_status: Map<u64, VerificationRecord> = env
        .storage()
        .instance()
        .get(&DataKey::VerificationStatus)
        .unwrap_or_else(|| Map::new(env));

    if !verification_status.contains_key(record_id) {
        return Err(ContractError::RecordNotFound);
    }

    verification_status.set(record_id, verification);
    env.storage().instance().set(&DataKey::VerificationStatus, &verification_status);

    Ok(())
}

pub fn get_consumer_records(
    env: &Env,
    consumer: Address,
    offset: u32,
    limit: u32,
) -> Result<Vec<u64>, ContractError> {
    let consumption_reports: Map<u64, ConsumptionRecord> = env
        .storage()
        .instance()
        .get(&DataKey::ConsumptionReports)
        .unwrap_or_else(|| Map::new(env));

    let mut matching_records = Vec::new(env);
    let mut count = 0u32;
    let mut added = 0u32;

    for entry in consumption_reports.iter() {
        let (record_id, record) = entry;

        if record.consumer == consumer {
            if count >= offset && added < limit {
                matching_records.push_back(record_id);
                added += 1;
            }
            count += 1;
        }
    }

    Ok(matching_records)
}

pub fn get_records_by_status(
    env: &Env,
    status: VerificationStatus,
    offset: u32,
    limit: u32,
) -> Result<Vec<u64>, ContractError> {
    let verification_status_map: Map<u64, VerificationRecord> = env
        .storage()
        .instance()
        .get(&DataKey::VerificationStatus)
        .unwrap_or_else(|| Map::new(env));

    let mut matching_records = Vec::new(env);
    let mut count = 0u32;
    let mut added = 0u32;

    for entry in verification_status_map.iter() {
        let (record_id, verification) = entry;

        if verification.status == status {
            if count >= offset && added < limit {
                matching_records.push_back(record_id);
                added += 1;
            }
            count += 1;
        }
    }

    Ok(matching_records)
}

pub fn get_records_by_verifier(
    env: &Env,
    verifier: Address,
    offset: u32,
    limit: u32,
) -> Result<Vec<u64>, ContractError> {
    let verification_status: Map<u64, VerificationRecord> = env
        .storage()
        .instance()
        .get(&DataKey::VerificationStatus)
        .unwrap_or_else(|| Map::new(env));

    let mut matching_records = Vec::new(env);
    let mut count = 0u32;
    let mut added = 0u32;

    for entry in verification_status.iter() {
        let (record_id, verification) = entry;

        if verification.verifier == verifier {
            if count >= offset && added < limit {
                matching_records.push_back(record_id);
                added += 1;
            }
            count += 1;
        }
    }

    Ok(matching_records)
}

pub fn get_records_by_time_range(
    env: &Env,
    start_time: u64,
    end_time: u64,
    offset: u32,
    limit: u32,
) -> Result<Vec<u64>, ContractError> {
    let consumption_reports: Map<u64, ConsumptionRecord> = env
        .storage()
        .instance()
        .get(&DataKey::ConsumptionReports)
        .unwrap_or_else(|| Map::new(env));

    let mut matching_records = Vec::new(env);
    let mut count = 0u32;
    let mut added = 0u32;

    for entry in consumption_reports.iter() {
        let (record_id, record) = entry;

        if record.timestamp >= start_time && record.timestamp <= end_time {
            if count >= offset && added < limit {
                matching_records.push_back(record_id);
                added += 1;
            }
            count += 1;
        }
    }

    Ok(matching_records)
}

pub fn get_meter_records(
    env: &Env,
    meter_id: String,
    offset: u32,
    limit: u32,
) -> Result<Vec<u64>, ContractError> {
    let consumption_reports: Map<u64, ConsumptionRecord> = env
        .storage()
        .instance()
        .get(&DataKey::ConsumptionReports)
        .unwrap_or_else(|| Map::new(env));

    let mut matching_records = Vec::new(env);
    let mut count = 0u32;
    let mut added = 0u32;

    for entry in consumption_reports.iter() {
        let (record_id, record) = entry;

        if record.meter_id == meter_id {
            if count >= offset && added < limit {
                matching_records.push_back(record_id);
                added += 1;
            }
            count += 1;
        }
    }

    Ok(matching_records)
}

pub fn get_consumption_summary(
    env: &Env,
    consumer: Address,
    start_time: u64,
    end_time: u64,
) -> Result<(u64, u64), ContractError> {
    let consumption_reports: Map<u64, ConsumptionRecord> = env
        .storage()
        .instance()
        .get(&DataKey::ConsumptionReports)
        .unwrap_or_else(|| Map::new(env));

    let mut total_consumption = 0u64;
    let mut record_count = 0u64;

    for entry in consumption_reports.iter() {
        let (_record_id, record) = entry;

        if record.consumer == consumer
            && record.timestamp >= start_time
            && record.timestamp <= end_time {
            total_consumption += record.consumption_kwh;
            record_count += 1;
        }
    }

    Ok((total_consumption, record_count))
}

pub fn delete_consumption_record(
    env: &Env,
    record_id: u64,
) -> Result<(), ContractError> {
    let mut consumption_reports: Map<u64, ConsumptionRecord> = env
        .storage()
        .instance()
        .get(&DataKey::ConsumptionReports)
        .unwrap_or_else(|| Map::new(env));

    if !consumption_reports.contains_key(record_id) {
        return Err(ContractError::RecordNotFound);
    }

    consumption_reports.remove(record_id);
    env.storage().instance().set(&DataKey::ConsumptionReports, &consumption_reports);

    let mut verification_status: Map<u64, VerificationRecord> = env
        .storage()
        .instance()
        .get(&DataKey::VerificationStatus)
        .unwrap_or_else(|| Map::new(env));

    verification_status.remove(record_id);
    env.storage().instance().set(&DataKey::VerificationStatus, &verification_status);

    Ok(())
}