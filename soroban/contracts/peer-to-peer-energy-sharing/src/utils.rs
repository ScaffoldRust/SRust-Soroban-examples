use soroban_sdk::{contracterror, contracttype, Address, Env};

#[contracttype]
#[derive(Copy, Clone)]
#[repr(u32)]
pub enum DataKey {
    Initialized = 0,
    Admin = 1,
    Prosumers = 2,
    Agreements = 3,
    Transactions = 4,
    NextAgreementId = 5,
    NextTransactionId = 6,
    TokenContract = 7,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct EnergyAgreement {
    pub agreement_id: u64,
    pub provider: Address,
    pub consumer: Address,
    pub energy_amount_kwh: u64,
    pub price_per_kwh: u64,
    pub total_amount: u64,
    pub delivery_deadline: u64,
    pub status: AgreementStatus,
    pub created_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct EnergyTransaction {
    pub transaction_id: u64,
    pub agreement_id: u64,
    pub provider: Address,
    pub consumer: Address,
    pub energy_delivered_kwh: u64,
    pub meter_reading: u64,
    pub payment_amount: u64,
    pub delivered_at: u64,
    pub settled_at: Option<u64>,
    pub status: TransactionStatus,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
#[repr(u32)]
pub enum AgreementStatus {
    Active = 0,
    Delivered = 1,
    Settled = 2,
    Cancelled = 3,
    Expired = 4,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
#[repr(u32)]
pub enum TransactionStatus {
    Pending = 0,
    Delivered = 1,
    Settled = 2,
    Disputed = 3,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[repr(u32)]
pub enum SharingError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    NotAuthorized = 3,
    InvalidInput = 4,
    ProsumerNotRegistered = 5,
    AgreementNotFound = 6,
    TransactionNotFound = 7,
    AgreementNotActive = 8,
    InsufficientEnergy = 9,
    PaymentFailed = 10,
    DeliveryDeadlinePassed = 11,
    TransactionAlreadySettled = 12,
    SelfSharingNotAllowed = 13,
}

pub fn get_next_agreement_id(env: &Env) -> u64 {
    let current_id: u64 = env
        .storage()
        .instance()
        .get(&DataKey::NextAgreementId)
        .unwrap_or(1);
    env.storage()
        .instance()
        .set(&DataKey::NextAgreementId, &(current_id + 1));
    current_id
}

pub fn get_next_transaction_id(env: &Env) -> u64 {
    let current_id: u64 = env
        .storage()
        .instance()
        .get(&DataKey::NextTransactionId)
        .unwrap_or(1);
    env.storage()
        .instance()
        .set(&DataKey::NextTransactionId, &(current_id + 1));
    current_id
}
