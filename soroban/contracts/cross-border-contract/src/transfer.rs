use crate::types::*;
use soroban_sdk::{Address, Env, String, Vec};

pub fn initiate_transfer(
    env: Env,
    sender: Address,
    recipient: Address,
    amount: i128,
    currency: String,
    destination_network: String,
) -> u64 {
    sender.require_auth();

    // Validate inputs
    if amount <= 0 {
        panic!("Amount must be positive");
    }

    // Get next transfer ID
    let transfer_id = next_transfer_id(&env);

    // Create transfer request
    let transfer = TransferRequest {
        sender: sender.clone(),
        recipient,
        amount,
        currency,
        destination_network,
        timestamp: env.ledger().timestamp(),
    };

    // Store transfer
    env.storage()
        .instance()
        .set(&DataKey::Transfer(transfer_id), &transfer);
    env.storage().instance().set(
        &DataKey::Settlement(transfer_id),
        &SettlementStatus::Pending,
    );

    // Add to transfer history
    let mut history: Vec<TransferRequest> = env
        .storage()
        .instance()
        .get(&DataKey::TransferHistory)
        .unwrap_or_else(|| Vec::new(&env));
    history.push_back(transfer.clone());
    env.storage()
        .instance()
        .set(&DataKey::TransferHistory, &history);

    transfer_id
}

pub fn get_transfer_details(env: Env, transfer_id: u64) -> TransferRequest {
    env.storage()
        .instance()
        .get(&DataKey::Transfer(transfer_id))
        .unwrap_or_else(|| panic!("Transfer not found"))
}

fn next_transfer_id(env: &Env) -> u64 {
    let transfer_id = env
        .storage()
        .instance()
        .get(&DataKey::NextTransferId)
        .unwrap_or(1u64);
    env.storage()
        .instance()
        .set(&DataKey::NextTransferId, &(transfer_id + 1));
    transfer_id
}
