use crate::utils::*;
use soroban_sdk::{symbol_short, Address, Env, Map};

/// Create an energy sharing agreement
pub fn create_agreement(
    env: &Env,
    provider: Address,
    consumer: Address,
    energy_amount_kwh: u64,
    price_per_kwh: u64,
    delivery_deadline: u64,
) -> Result<u64, SharingError> {
    let agreement_id = get_next_agreement_id(env);
    let total_amount = energy_amount_kwh * price_per_kwh;
    let current_time = env.ledger().timestamp();

    if delivery_deadline <= current_time {
        return Err(SharingError::DeliveryDeadlinePassed);
    }

    let agreement = EnergyAgreement {
        agreement_id,
        provider: provider.clone(),
        consumer: consumer.clone(),
        energy_amount_kwh,
        price_per_kwh,
        total_amount,
        delivery_deadline,
        status: AgreementStatus::Active,
        created_at: current_time,
    };

    // Store agreement
    let mut agreements: Map<u64, EnergyAgreement> = env
        .storage()
        .instance()
        .get(&DataKey::Agreements)
        .unwrap_or_else(|| Map::new(env));

    agreements.set(agreement_id, agreement);
    env.storage()
        .instance()
        .set(&DataKey::Agreements, &agreements);

    // Emit agreement created event
    env.events().publish(
        (symbol_short!("agreement"), agreement_id),
        (provider, consumer, energy_amount_kwh, total_amount),
    );

    Ok(agreement_id)
}

/// Deliver energy and record meter verification
pub fn deliver_energy(
    env: &Env,
    agreement_id: u64,
    energy_delivered_kwh: u64,
    meter_reading: u64,
    provider: Address,
) -> Result<u64, SharingError> {
    let mut agreements: Map<u64, EnergyAgreement> = env
        .storage()
        .instance()
        .get(&DataKey::Agreements)
        .unwrap_or_else(|| Map::new(env));

    let mut agreement = agreements
        .get(agreement_id)
        .ok_or(SharingError::AgreementNotFound)?;

    // Validate agreement
    if agreement.provider != provider {
        return Err(SharingError::NotAuthorized);
    }

    if agreement.status != AgreementStatus::Active {
        return Err(SharingError::AgreementNotActive);
    }

    let current_time = env.ledger().timestamp();
    if current_time > agreement.delivery_deadline {
        agreement.status = AgreementStatus::Expired;
        agreements.set(agreement_id, agreement);
        env.storage()
            .instance()
            .set(&DataKey::Agreements, &agreements);
        return Err(SharingError::DeliveryDeadlinePassed);
    }

    if energy_delivered_kwh > agreement.energy_amount_kwh {
        return Err(SharingError::InsufficientEnergy);
    }

    // Create transaction record
    let transaction_id = get_next_transaction_id(env);
    let payment_amount = energy_delivered_kwh * agreement.price_per_kwh;

    let transaction = EnergyTransaction {
        transaction_id,
        agreement_id,
        provider: agreement.provider.clone(),
        consumer: agreement.consumer.clone(),
        energy_delivered_kwh,
        meter_reading,
        payment_amount,
        delivered_at: current_time,
        settled_at: None,
        status: TransactionStatus::Delivered,
    };

    // Store transaction
    let mut transactions: Map<u64, EnergyTransaction> = env
        .storage()
        .instance()
        .get(&DataKey::Transactions)
        .unwrap_or_else(|| Map::new(env));

    transactions.set(transaction_id, transaction);
    env.storage()
        .instance()
        .set(&DataKey::Transactions, &transactions);

    // Update agreement status
    agreement.status = AgreementStatus::Delivered;
    agreements.set(agreement_id, agreement);
    env.storage()
        .instance()
        .set(&DataKey::Agreements, &agreements);

    // Emit delivery event
    env.events().publish(
        (symbol_short!("delivery"), transaction_id),
        (agreement_id, energy_delivered_kwh, meter_reading),
    );

    Ok(transaction_id)
}

/// Get agreement details
pub fn get_agreement(env: &Env, agreement_id: u64) -> Result<EnergyAgreement, SharingError> {
    let agreements: Map<u64, EnergyAgreement> = env
        .storage()
        .instance()
        .get(&DataKey::Agreements)
        .unwrap_or_else(|| Map::new(env));

    agreements
        .get(agreement_id)
        .ok_or(SharingError::AgreementNotFound)
}

/// Cancel an agreement (only by provider before delivery)
pub fn cancel_agreement(
    env: &Env,
    agreement_id: u64,
    canceller: Address,
) -> Result<(), SharingError> {
    let mut agreements: Map<u64, EnergyAgreement> = env
        .storage()
        .instance()
        .get(&DataKey::Agreements)
        .unwrap_or_else(|| Map::new(env));

    let mut agreement = agreements
        .get(agreement_id)
        .ok_or(SharingError::AgreementNotFound)?;

    if agreement.provider != canceller && agreement.consumer != canceller {
        return Err(SharingError::NotAuthorized);
    }

    if agreement.status != AgreementStatus::Active {
        return Err(SharingError::AgreementNotActive);
    }

    agreement.status = AgreementStatus::Cancelled;
    agreements.set(agreement_id, agreement);
    env.storage()
        .instance()
        .set(&DataKey::Agreements, &agreements);

    env.events()
        .publish((symbol_short!("cancelled"), agreement_id), canceller);

    Ok(())
}