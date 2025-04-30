use crate::compliance;
use crate::types::*;
use soroban_sdk::{Address, Env};

pub fn execute_settlement(env: Env, transfer_id: u64, org: Address) {
    org.require_auth();

    // Get transfer details
    let transfer: TransferRequest = env
        .storage()
        .instance()
        .get(&DataKey::Transfer(transfer_id))
        .unwrap_or_else(|| panic!("Transfer not found"));

    // Verify compliance for sender and recipient
    if !compliance::is_compliant(&env, &transfer.sender)
        || !compliance::is_compliant(&env, &transfer.recipient)
    {
        panic!("Compliance check failed");
    }

    // Verify current status
    let status: SettlementStatus = env
        .storage()
        .instance()
        .get(&DataKey::Settlement(transfer_id))
        .unwrap_or(SettlementStatus::Pending);
    if status != SettlementStatus::Pending {
        panic!("Transfer is not in pending state");
    }

    // Update status to Approved
    env.storage().instance().set(
        &DataKey::Settlement(transfer_id),
        &SettlementStatus::Approved,
    );

    // Simulate settlement with destination network (mocked for simplicity)
    // In practice, integrate with external network APIs
    env.storage().instance().set(
        &DataKey::Settlement(transfer_id),
        &SettlementStatus::Settled,
    );
}

pub fn refund_transfer(env: Env, transfer_id: u64, org: Address) {
    org.require_auth();

    // Verify current status
    let status: SettlementStatus = env
        .storage()
        .instance()
        .get(&DataKey::Settlement(transfer_id))
        .unwrap_or(SettlementStatus::Pending);
    if status != SettlementStatus::Pending && status != SettlementStatus::Approved {
        panic!("Transfer cannot be refunded");
    }

    // Update status to Refunded
    env.storage().instance().set(
        &DataKey::Settlement(transfer_id),
        &SettlementStatus::Refunded,
    );

    // In practice, initiate refund to sender's account
}

pub fn get_transfer_status(env: Env, transfer_id: u64) -> SettlementStatus {
    env.storage()
        .instance()
        .get(&DataKey::Settlement(transfer_id))
        .unwrap_or(SettlementStatus::Pending)
}
