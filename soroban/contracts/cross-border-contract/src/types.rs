use soroban_sdk::{contracttype, Address, BytesN, String};

#[contracttype]
pub enum DataKey {
    Transfer(u64),                // Transfer ID -> TransferRequest
    Compliance(Address),          // Address -> ComplianceData
    Settlement(u64),              // Transfer ID -> SettlementStatus
    Fees,                         // FeeStructure
    ExchangeRate(String, String), // (Source Currency, Target Currency) -> ExchangeRate
    NextTransferId,               // Counter for transfer IDs
    TransferHistory,              // List of all transfers
}

#[contracttype]
#[derive(Clone)]
pub struct TransferRequest {
    pub sender: Address,
    pub recipient: Address,
    pub amount: i128,
    pub currency: String,
    pub destination_network: String,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct ComplianceData {
    pub kyc_verified: bool,
    pub aml_verified: bool,
    pub verification_documents: BytesN<32>, // Hash of KYC/AML docs
}

#[contracttype]
#[derive(Clone, PartialEq, Debug)]
pub enum SettlementStatus {
    Pending,
    Approved,
    Settled,
    Refunded,
    Rejected,
}

#[contracttype]
#[derive(Clone)]
pub struct FeeStructure {
    pub base_fee: i128,
    pub percentage: u32,         // Basis points (e.g., 100 = 1%)
    pub urgency_multiplier: u32, // Multiplier for urgent transfers
}

#[contracttype]
#[derive(Clone)]
pub struct ExchangeRate {
    pub source_currency: String,
    pub target_currency: String,
    pub rate: i128, // Fixed-point rate scaled by RATE_SCALE
    pub timestamp: u64,
}

// Scale factor for fixed-point exchange rates (e.g., 1,000,000 = 6 decimal places)
pub const RATE_SCALE: i128 = 1_000_000;
