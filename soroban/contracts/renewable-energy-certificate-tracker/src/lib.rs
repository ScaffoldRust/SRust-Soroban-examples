#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Address, BytesN, Env, Map, String, Vec};

pub mod certificate;
pub mod transfer;
pub mod utils;

#[cfg(test)]
mod tests;

// Core data structures for Renewable Energy Certificates (RECs)
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct REC {
    pub id: BytesN<32>,
    pub issuer: Address,
    pub energy_source: EnergySource,
    pub production_date: u64,
    pub production_location: String,
    pub capacity_mwh: i128, // Megawatt-hours
    pub current_owner: Address,
    pub status: RECStatus,
    pub verification_standard: String, // I-REC, RE100, etc.
    pub verification_hash: BytesN<32>, // Hash of verification documents
    pub issuance_date: u64,
    pub metadata: Map<String, String>, // Additional metadata
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EnergySource {
    Solar,
    Wind,
    Hydro,
    Geothermal,
    Biomass,
    Tidal,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RECStatus {
    Issued,
    Transferred,
    Retired,
    Suspended,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RECEvent {
    pub event_type: EventType,
    pub rec_id: BytesN<32>,
    pub timestamp: u64,
    pub from: Option<Address>,
    pub to: Option<Address>,
    pub capacity_mwh: i128,
    pub transaction_hash: BytesN<32>,
    pub notes: String,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EventType {
    Issuance,
    Transfer,
    Retirement,
    Suspension,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IssuerInfo {
    pub address: Address,
    pub name: String,
    pub authorized: bool,
    pub registration_date: u64,
    pub total_issued: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransferParams {
    pub rec_id: BytesN<32>,
    pub from: Address,
    pub to: Address,
    pub capacity_mwh: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetirementParams {
    pub rec_id: BytesN<32>,
    pub owner: Address,
    pub capacity_mwh: i128,
    pub retirement_reason: String,
}

// Storage keys
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Initialized,
    REC(BytesN<32>),
    Issuer(Address),
    RecHistory(BytesN<32>),
    Stats,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractStats {
    pub total_recs_issued: i128,
    pub total_capacity_issued_mwh: i128,
    pub total_capacity_transferred_mwh: i128,
    pub total_capacity_retired_mwh: i128,
}

#[contract]
pub struct RenewableEnergyCertificateTracker;

#[contractimpl]
impl RenewableEnergyCertificateTracker {
    /// Initialize the contract with admin address
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Initialized) {
            panic!("Already initialized");
        }

        admin.require_auth();

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Initialized, &true);

        let stats = ContractStats {
            total_recs_issued: 0,
            total_capacity_issued_mwh: 0,
            total_capacity_transferred_mwh: 0,
            total_capacity_retired_mwh: 0,
        };
        env.storage().instance().set(&DataKey::Stats, &stats);
    }

    /// Register an authorized issuer
    pub fn register_issuer(env: Env, issuer: Address, name: String) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        let issuer_info = IssuerInfo {
            address: issuer.clone(),
            name,
            authorized: true,
            registration_date: env.ledger().timestamp(),
            total_issued: 0,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Issuer(issuer), &issuer_info);
    }

    /// Issue a new REC for verified renewable energy production
    pub fn issue_rec(
        env: Env,
        issuer: Address,
        energy_source: EnergySource,
        production_date: u64,
        production_location: String,
        capacity_mwh: i128,
        verification_standard: String,
        verification_hash: BytesN<32>,
        metadata: Map<String, String>,
    ) -> BytesN<32> {
        certificate::issue_rec(
            &env,
            issuer,
            energy_source,
            production_date,
            production_location,
            capacity_mwh,
            verification_standard,
            verification_hash,
            metadata,
        )
    }

    /// Transfer a REC to a new owner
    pub fn transfer_rec(
        env: Env,
        rec_id: BytesN<32>,
        from: Address,
        to: Address,
        capacity_mwh: i128,
    ) -> bool {
        transfer::transfer_rec(&env, rec_id, from, to, capacity_mwh)
    }

    /// Retire a REC to claim renewable energy usage
    pub fn retire_rec(
        env: Env,
        rec_id: BytesN<32>,
        owner: Address,
        capacity_mwh: i128,
        retirement_reason: String,
    ) -> bool {
        transfer::retire_rec(&env, rec_id, owner, capacity_mwh, retirement_reason)
    }

    /// Get the status and details of a REC
    pub fn get_rec_status(env: Env, rec_id: BytesN<32>) -> REC {
        let rec_key = DataKey::REC(rec_id);
        env.storage().persistent().get(&rec_key).unwrap()
    }

    /// Get the history of a REC
    pub fn get_rec_history(env: Env, rec_id: BytesN<32>) -> Vec<RECEvent> {
        let history_key = DataKey::RecHistory(rec_id);
        env.storage()
            .persistent()
            .get(&history_key)
            .unwrap_or(Vec::new(&env))
    }

    /// Get issuer information
    pub fn get_issuer_info(env: Env, issuer: Address) -> IssuerInfo {
        env.storage()
            .persistent()
            .get(&DataKey::Issuer(issuer))
            .unwrap()
    }

    /// Get contract statistics
    pub fn get_contract_stats(env: Env) -> (i128, i128, i128, i128) {
        let stats: ContractStats = env.storage().instance().get(&DataKey::Stats).unwrap();
        (
            stats.total_recs_issued,
            stats.total_capacity_issued_mwh,
            stats.total_capacity_transferred_mwh,
            stats.total_capacity_retired_mwh,
        )
    }

    /// Suspend a REC (admin only)
    pub fn suspend_rec(env: Env, rec_id: BytesN<32>) -> bool {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        let rec_key = DataKey::REC(rec_id.clone());
        let mut rec: REC = env.storage().persistent().get(&rec_key).unwrap();

        rec.status = RECStatus::Suspended;
        env.storage().persistent().set(&rec_key, &rec);

        // Log suspension event
        let event = RECEvent {
            event_type: EventType::Suspension,
            rec_id,
            timestamp: env.ledger().timestamp(),
            from: Some(rec.current_owner.clone()),
            to: None,
            capacity_mwh: rec.capacity_mwh,
            transaction_hash: BytesN::from_array(&env, &[0u8; 32]),
            notes: String::from_str(&env, "REC suspended by admin"),
        };

        let history_key = DataKey::RecHistory(rec.id.clone());
        let mut history: Vec<RECEvent> = env
            .storage()
            .persistent()
            .get(&history_key)
            .unwrap_or(Vec::new(&env));
        history.push_back(event);
        env.storage().persistent().set(&history_key, &history);

        true
    }
}
