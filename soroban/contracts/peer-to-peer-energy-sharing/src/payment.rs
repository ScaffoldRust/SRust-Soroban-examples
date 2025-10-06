use crate::utils::*;
use soroban_sdk::{symbol_short, token, Address, Env, Map, Vec};

/// Settle payment for delivered energy
pub fn settle_payment(
    env: &Env,
    transaction_id: u64,
    settler: Address,
) -> Result<(), SharingError> {
    let mut transactions: Map<u64, EnergyTransaction> = env
        .storage()
        .instance()
        .get(&DataKey::Transactions)
        .unwrap_or_else(|| Map::new(env));

    let mut transaction = transactions
        .get(transaction_id)
        .ok_or(SharingError::TransactionNotFound)?;

    // Verify authorization (provider or consumer can settle)
    if settler != transaction.provider && settler != transaction.consumer {
        return Err(SharingError::NotAuthorized);
    }

    // Check transaction status
    if transaction.status == TransactionStatus::Settled {
        return Err(SharingError::TransactionAlreadySettled);
    }

    if transaction.status != TransactionStatus::Delivered {
        return Err(SharingError::InvalidInput);
    }

    // Execute payment transfer
    execute_payment(env, &transaction)?;

    // Update transaction status
    transaction.status = TransactionStatus::Settled;
    transaction.settled_at = Some(env.ledger().timestamp());

    transactions.set(transaction_id, transaction.clone());
    env.storage()
        .instance()
        .set(&DataKey::Transactions, &transactions);

    // Update agreement status
    update_agreement_status(env, transaction.agreement_id)?;

    // Emit settlement event
    env.events().publish(
        (symbol_short!("settled"), transaction_id),
        (transaction.energy_delivered_kwh, transaction.payment_amount),
    );

    Ok(())
}

/// Execute payment transfer using Stellar tokens
fn execute_payment(env: &Env, transaction: &EnergyTransaction) -> Result<(), SharingError> {
    // Get token contract
    let token_contract: Address = env
        .storage()
        .instance()
        .get(&DataKey::TokenContract)
        .ok_or(SharingError::PaymentFailed)?;

    // Require consumer authorization for payment
    transaction.consumer.require_auth();

    // Execute transfer from consumer to provider
    let token_client = token::Client::new(env, &token_contract);
    token_client.transfer(
        &transaction.consumer,
        &transaction.provider,
        &(transaction.payment_amount as i128),
    );

    // Emit payment event
    env.events().publish(
        (symbol_short!("payment"), transaction.consumer.clone()),
        (transaction.provider.clone(), transaction.payment_amount),
    );

    Ok(())
}

/// Update agreement status to settled
fn update_agreement_status(env: &Env, agreement_id: u64) -> Result<(), SharingError> {
    let mut agreements: Map<u64, EnergyAgreement> = env
        .storage()
        .instance()
        .get(&DataKey::Agreements)
        .unwrap_or_else(|| Map::new(env));

    let mut agreement = agreements
        .get(agreement_id)
        .ok_or(SharingError::AgreementNotFound)?;

    agreement.status = AgreementStatus::Settled;
    agreements.set(agreement_id, agreement);
    env.storage()
        .instance()
        .set(&DataKey::Agreements, &agreements);

    Ok(())
}

/// Get transaction details
pub fn get_transaction(env: &Env, transaction_id: u64) -> Result<EnergyTransaction, SharingError> {
    let transactions: Map<u64, EnergyTransaction> = env
        .storage()
        .instance()
        .get(&DataKey::Transactions)
        .unwrap_or_else(|| Map::new(env));

    transactions
        .get(transaction_id)
        .ok_or(SharingError::TransactionNotFound)
}

/// Get transaction history for a prosumer
pub fn get_transaction_history(
    env: &Env,
    prosumer: Address,
) -> Result<Vec<EnergyTransaction>, SharingError> {
    let transactions: Map<u64, EnergyTransaction> = env
        .storage()
        .instance()
        .get(&DataKey::Transactions)
        .unwrap_or_else(|| Map::new(env));

    let mut prosumer_transactions = Vec::new(env);
    for (_, transaction) in transactions.iter() {
        if transaction.provider == prosumer || transaction.consumer == prosumer {
            prosumer_transactions.push_back(transaction);
        }
    }

    Ok(prosumer_transactions)
}

/// Dispute a transaction (for future dispute resolution)
pub fn dispute_transaction(
    env: &Env,
    transaction_id: u64,
    disputer: Address,
) -> Result<(), SharingError> {
    let mut transactions: Map<u64, EnergyTransaction> = env
        .storage()
        .instance()
        .get(&DataKey::Transactions)
        .unwrap_or_else(|| Map::new(env));

    let mut transaction = transactions
        .get(transaction_id)
        .ok_or(SharingError::TransactionNotFound)?;

    // Only provider or consumer can dispute
    if disputer != transaction.provider && disputer != transaction.consumer {
        return Err(SharingError::NotAuthorized);
    }

    // Can only dispute delivered transactions that haven't been settled
    if transaction.status != TransactionStatus::Delivered {
        return Err(SharingError::InvalidInput);
    }

    transaction.status = TransactionStatus::Disputed;
    transactions.set(transaction_id, transaction);
    env.storage()
        .instance()
        .set(&DataKey::Transactions, &transactions);

    env.events()
        .publish((symbol_short!("disputed"), transaction_id), disputer);

    Ok(())
}
