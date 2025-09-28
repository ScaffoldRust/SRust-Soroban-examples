use soroban_sdk::{Address, Env, Map};
use crate::{ContractError, DataKey, ConsumptionRecord};

const MAX_CONSUMPTION_KWH: u64 = 100_000;
const MIN_CONSUMPTION_KWH: u64 = 0;
const MAX_VOLTAGE: u32 = 250;
const MIN_VOLTAGE: u32 = 200;
const MAX_TEMPERATURE: i32 = 60;
const MIN_TEMPERATURE: i32 = -40;

pub fn validate_consumption_data(
    _env: &Env,
    consumption_kwh: u64,
    meter_reading: u64,
    temperature: Option<i32>,
    voltage: Option<u32>,
) -> Result<(), ContractError> {
    if consumption_kwh < MIN_CONSUMPTION_KWH || consumption_kwh > MAX_CONSUMPTION_KWH {
        return Err(ContractError::InvalidMeterData);
    }

    if meter_reading == 0 {
        return Err(ContractError::InvalidMeterData);
    }

    if let Some(temp) = temperature {
        if temp < MIN_TEMPERATURE || temp > MAX_TEMPERATURE {
            return Err(ContractError::InvalidMeterData);
        }
    }

    if let Some(volt) = voltage {
        if volt < MIN_VOLTAGE || volt > MAX_VOLTAGE {
            return Err(ContractError::InvalidMeterData);
        }
    }

    Ok(())
}

pub fn verify_data_integrity(
    env: &Env,
    record: &ConsumptionRecord,
) -> Result<(), ContractError> {
    let current_timestamp = env.ledger().timestamp();

    if record.timestamp > current_timestamp {
        return Err(ContractError::InvalidTimestamp);
    }

    let time_diff = current_timestamp - record.timestamp;
    if time_diff > 86400 * 30 {
        return Err(ContractError::InvalidTimestamp);
    }

    let computed_hash = crate::utils::compute_data_hash(
        env,
        &record.consumer,
        &record.meter_id,
        record.consumption_kwh,
        record.meter_reading,
        record.timestamp,
    );

    if computed_hash != record.data_hash {
        return Err(ContractError::DataIntegrityError);
    }

    validate_consumption_data(
        env,
        record.consumption_kwh,
        record.meter_reading,
        record.temperature,
        record.voltage,
    )?;

    validate_meter_reading_consistency(env, record)?;

    Ok(())
}

pub fn is_authorized_verifier(env: &Env, verifier: &Address) -> bool {
    let verifiers: Map<Address, bool> = env
        .storage()
        .instance()
        .get(&DataKey::Verifiers)
        .unwrap_or_else(|| Map::new(env));

    verifiers.get(verifier.clone()).unwrap_or(false)
}

pub fn validate_meter_reading_consistency(
    env: &Env,
    record: &ConsumptionRecord,
) -> Result<(), ContractError> {
    let consumption_reports: Map<u64, ConsumptionRecord> = env
        .storage()
        .instance()
        .get(&DataKey::ConsumptionReports)
        .unwrap_or_else(|| Map::new(env));

    let mut previous_reading: Option<u64> = None;
    let mut latest_timestamp = 0;

    for entry in consumption_reports.iter() {
        let (_, existing_record) = entry;
        if existing_record.meter_id == record.meter_id
            && existing_record.consumer == record.consumer
            && existing_record.timestamp < record.timestamp
            && existing_record.timestamp > latest_timestamp
        {
            previous_reading = Some(existing_record.meter_reading);
            latest_timestamp = existing_record.timestamp;
        }
    }

    if let Some(prev_reading) = previous_reading {
        if record.meter_reading <= prev_reading {
            return Err(ContractError::InvalidMeterData);
        }

        let reading_diff = record.meter_reading - prev_reading;
        let time_diff_hours = (record.timestamp - latest_timestamp) / 3600;

        if time_diff_hours > 0 {
            let consumption_rate = reading_diff / time_diff_hours;
            if consumption_rate > 50 {
                return Err(ContractError::InvalidMeterData);
            }
        }
    }

    Ok(())
}

pub fn validate_environmental_conditions(
    _env: &Env,
    temperature: Option<i32>,
    voltage: Option<u32>,
) -> Result<(), ContractError> {
    if let Some(temp) = temperature {
        if temp < MIN_TEMPERATURE || temp > MAX_TEMPERATURE {
            return Err(ContractError::InvalidMeterData);
        }
    }

    if let Some(volt) = voltage {
        if volt < MIN_VOLTAGE || volt > MAX_VOLTAGE {
            return Err(ContractError::InvalidMeterData);
        }
    }

    Ok(())
}

pub fn flag_suspicious_data(
    _env: &Env,
    record: &ConsumptionRecord,
) -> Result<bool, ContractError> {
    let mut suspicious = false;

    if record.consumption_kwh > 10000 {
        suspicious = true;
    }

    if let Some(temp) = record.temperature {
        if temp > 50 || temp < -20 {
            suspicious = true;
        }
    }

    if let Some(voltage) = record.voltage {
        if voltage > 240 || voltage < 210 {
            suspicious = true;
        }
    }

    let consumption_per_hour = record.consumption_kwh;
    if consumption_per_hour > 500 {
        suspicious = true;
    }

    Ok(suspicious)
}

pub fn calculate_verification_score(
    env: &Env,
    record: &ConsumptionRecord,
) -> Result<u32, ContractError> {
    let mut score = 100u32;

    if flag_suspicious_data(env, record)? {
        score = score.saturating_sub(30);
    }

    if record.temperature.is_none() || record.voltage.is_none() {
        score = score.saturating_sub(10);
    }

    if record.consumption_kwh == 0 {
        score = 0;
    }

    let time_since_reading = env.ledger().timestamp() - record.timestamp;
    if time_since_reading > 3600 {
        score = score.saturating_sub(5);
    }

    Ok(score)
}