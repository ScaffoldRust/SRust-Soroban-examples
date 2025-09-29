#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Vec, BytesN, String, Map};

pub mod credits;
pub mod trading;
pub mod utils;

#[cfg(test)]
mod test;

// Core data structures for carbon credits
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CarbonCredit {
    pub id: BytesN<32>,
    pub issuer: Address,
    pub project_type: String,
    pub project_location: String,
    pub verification_standard: String, // Verra, Gold Standard, etc.
    pub issuance_date: u64,
    pub vintage_year: u32,
    pub quantity: i128, // CO2 equivalent in tons
    pub current_owner: Address,
    pub status: CreditStatus,
    pub verification_hash: BytesN<32>, // Hash of verification documents
    pub metadata: Map<String, String>, // Additional project metadata
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CreditStatus {
    Issued,
    Traded,
    Retired,
    Suspended,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreditEvent {
    pub event_type: EventType,
    pub credit_id: BytesN<32>,
    pub timestamp: u64,
    pub from: Option<Address>,
    pub to: Option<Address>,
    pub quantity: i128,
    pub transaction_hash: BytesN<32>,
    pub metadata: Map<String, String>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EventType {
    Issuance,
    Trade,
    Retirement,
    Suspension,
    Verification,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IssuerProfile {
    pub address: Address,
    pub name: String,
    pub verification_standards: Vec<String>,
    pub is_active: bool,
    pub total_issued: i128,
    pub total_retired: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TradingParams {
    pub credit_id: BytesN<32>,
    pub from: Address,
    pub to: Address,
    pub quantity: i128,
    pub price: i128, // Price per ton of CO2
    pub payment_token: Address, // Payment token address
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetirementParams {
    pub credit_id: BytesN<32>,
    pub owner: Address,
    pub quantity: i128,
    pub retirement_reason: String,
    pub retirement_certificate: BytesN<32>, // Hash of retirement certificate
}

// Storage keys
#[contracttype]
pub enum DataKey {
    Credit(BytesN<32>),
    CreditEvent(BytesN<32>, u32), // credit_id, event_index
    Issuer(Address),
    IssuerCount,
    CreditCount,
    EventCount(BytesN<32>), // credit_id -> event_count
    Admin,
    TradingFeeRate, // Fee rate for trading (in basis points)
    RetirementFeeRate, // Fee rate for retirement (in basis points)
    TotalCreditsIssued,
    TotalCreditsRetired,
}

#[contract]
pub struct CarbonCreditRegistry;

#[contractimpl]
impl CarbonCreditRegistry {
    /// Initialize the contract with admin and fee rates
    pub fn initialize(
        env: Env,
        admin: Address,
        trading_fee_rate: u32, // in basis points (e.g., 25 = 0.25%)
        retirement_fee_rate: u32,
    ) {
        admin.require_auth();
        
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::TradingFeeRate, &trading_fee_rate);
        env.storage().instance().set(&DataKey::RetirementFeeRate, &retirement_fee_rate);
        env.storage().instance().set(&DataKey::IssuerCount, &0u32);
        env.storage().instance().set(&DataKey::CreditCount, &0u32);
        env.storage().instance().set(&DataKey::TotalCreditsIssued, &0i128);
        env.storage().instance().set(&DataKey::TotalCreditsRetired, &0i128);
    }

    /// Register a new issuer
    pub fn register_issuer(
        env: Env,
        issuer_address: Address,
        name: String,
        verification_standards: Vec<String>,
    ) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin)
            .unwrap();
        admin.require_auth();

        let issuer_profile = IssuerProfile {
            address: issuer_address.clone(),
            name,
            verification_standards,
            is_active: true,
            total_issued: 0,
            total_retired: 0,
        };

        env.storage().instance().set(&DataKey::Issuer(issuer_address), &issuer_profile);
        
        let mut issuer_count: u32 = env.storage().instance().get(&DataKey::IssuerCount)
            .unwrap_or(0);
        issuer_count += 1;
        env.storage().instance().set(&DataKey::IssuerCount, &issuer_count);
    }

    /// Issue a new carbon credit
    pub fn issue_credit(
        env: Env,
        issuer: Address,
        project_type: String,
        project_location: String,
        verification_standard: String,
        vintage_year: u32,
        quantity: i128,
        verification_hash: BytesN<32>,
        metadata: Map<String, String>,
    ) -> BytesN<32> {
        issuer.require_auth();

        // Generate unique credit ID
        let credit_id = utils::generate_credit_id(&env, &issuer, &verification_hash);

        // Create the carbon credit
        let credit = CarbonCredit {
            id: credit_id.clone(),
            issuer: issuer.clone(),
            project_type,
            project_location,
            verification_standard,
            issuance_date: env.ledger().timestamp(),
            vintage_year,
            quantity,
            current_owner: issuer.clone(),
            status: CreditStatus::Issued,
            verification_hash,
            metadata,
        };

        // Store the credit
        env.storage().instance().set(&DataKey::Credit(credit_id.clone()), &credit);

        // Record issuance event
        credits::record_credit_event(
            &env,
            &credit_id,
            EventType::Issuance,
            None,
            Some(issuer),
            quantity,
        );

        // Increment credit count
        let mut credit_count: u32 = env.storage().instance().get(&DataKey::CreditCount)
            .unwrap_or(0);
        credit_count += 1;
        env.storage().instance().set(&DataKey::CreditCount, &credit_count);

        credit_id
    }

    /// Trade a carbon credit
    pub fn trade_credit(
        env: Env,
        params: TradingParams,
    ) {
        trading::trade_credit(env, params)
    }

    /// Retire a carbon credit
    pub fn retire_credit(
        env: Env,
        params: RetirementParams,
    ) {
        trading::retire_credit(env, params)
    }

    /// Suspend a credit (admin only)
    pub fn suspend_credit(
        env: Env,
        credit_id: BytesN<32>,
        reason: String,
    ) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin)
            .unwrap();
        admin.require_auth();

        let mut credit: CarbonCredit = env.storage().instance().get(&DataKey::Credit(credit_id.clone()))
            .unwrap();

        credit.status = CreditStatus::Suspended;
        env.storage().instance().set(&DataKey::Credit(credit_id.clone()), &credit);
    }

    /// Get credit status and details
    pub fn get_credit_status(
        env: Env,
        credit_id: BytesN<32>,
    ) -> Option<CarbonCredit> {
        env.storage().instance().get(&DataKey::Credit(credit_id))
    }

    /// Get credit transaction history
    pub fn get_credit_history(
        env: Env,
        credit_id: BytesN<32>,
    ) -> Vec<CreditEvent> {
        let event_count: u32 = env.storage().instance().get(&DataKey::EventCount(credit_id.clone()))
            .unwrap_or(0);

        let mut events = Vec::new(&env);
        for i in 0..event_count {
            if let Some(event) = env.storage().instance().get(&DataKey::CreditEvent(credit_id.clone(), i)) {
                events.push_back(event);
            }
        }

        events
    }

    /// Get issuer profile
    pub fn get_issuer_profile(
        env: Env,
        issuer_address: Address,
    ) -> Option<IssuerProfile> {
        env.storage().instance().get(&DataKey::Issuer(issuer_address))
    }

    /// Get contract statistics
    pub fn get_contract_stats(env: Env) -> (u32, u32, i128, i128) {
        let issuer_count: u32 = env.storage().instance().get(&DataKey::IssuerCount)
            .unwrap_or(0);
        let credit_count: u32 = env.storage().instance().get(&DataKey::CreditCount)
            .unwrap_or(0);
        let total_issued: i128 = env.storage().instance().get(&DataKey::TotalCreditsIssued)
            .unwrap_or(0);
        let total_retired: i128 = env.storage().instance().get(&DataKey::TotalCreditsRetired)
            .unwrap_or(0);

        (issuer_count, credit_count, total_issued, total_retired)
    }
}