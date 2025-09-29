use soroban_sdk::{Env, Address, BytesN, Symbol, String, Vec};

/// Storage keys (max 9 characters for short symbols)
const PAY_STAT: Symbol = Symbol::short("PAY_STAT");
const PAY_AMT: Symbol = Symbol::short("PAY_AMT");
const PAY_PAT: Symbol = Symbol::short("PAY_PAT");
const PAY_PROV: Symbol = Symbol::short("PAY_PROV");
const PAY_SES: Symbol = Symbol::short("PAY_SES");
const PAY_DUR: Symbol = Symbol::short("PAY_DUR");
const PAY_CRT: Symbol = Symbol::short("PAY_CRT");
const PAY_CMP: Symbol = Symbol::short("PAY_CMP");
const PAT_PAY: Symbol = Symbol::short("PAT_PAY");
const PROV_PAY: Symbol = Symbol::short("PROV_PAY");
const PAY_CNT: Symbol = Symbol::short("PAY_CNT");

/// Payment status constants
const STATUS_PENDING: u32 = 0;
const STATUS_IN_PROGRESS: u32 = 1;
const STATUS_COMPLETED: u32 = 2;
const STATUS_REFUNDED: u32 = 3;
const STATUS_DISPUTED: u32 = 4;
const STATUS_CANCELLED: u32 = 5;

/// Initiate a payment for a telemedicine session
pub fn initiate_payment(
    env: &Env,
    patient: Address,
    provider: Address,
    session_id: BytesN<32>,
    estimated_duration: u64,
    consent_hash: BytesN<32>,
) -> BytesN<32> {
    // Check if contract is paused
    if is_contract_paused(env) {
        panic!("Contract is currently paused");
    }

    // Validate provider is registered and active
    let provider_hourly_rate = get_provider_hourly_rate(env, provider.clone());
    if provider_hourly_rate <= 0 {
        panic!("Provider not found or inactive");
    }

    // Calculate payment amount based on estimated duration
    let amount = calculate_payment_amount(provider_hourly_rate, estimated_duration);
    
    // Validate payment amount
    if amount <= 0 {
        panic!("Invalid payment amount");
    }

    // Generate unique payment ID
    let payment_id = generate_payment_id(env, &patient, &provider, &session_id);

    // Check if payment already exists for this session
    let storage = env.storage().instance();
    if storage.has(&(PAY_SES, session_id.clone())) {
        panic!("Payment already exists for this session");
    }

    // Store payment details using composite keys
    storage.set(&(PAY_STAT, payment_id.clone()), &STATUS_PENDING);
    storage.set(&(PAY_AMT, payment_id.clone()), &amount);
    storage.set(&(PAY_PAT, payment_id.clone()), &patient);
    storage.set(&(PAY_PROV, payment_id.clone()), &provider);
    storage.set(&(PAY_SES, session_id), &payment_id);
    storage.set(&(PAY_DUR, payment_id.clone()), &estimated_duration);
    storage.set(&(PAY_CRT, payment_id.clone()), &env.ledger().timestamp());

    // Add to patient and provider payment lists
    add_payment_to_address(env, &patient, &payment_id, "patient");
    add_payment_to_address(env, &provider, &payment_id, "provider");

    // Increment payment counter
    let counter: u64 = storage.get(&PAY_CNT).unwrap_or(0);
    storage.set(&PAY_CNT, &(counter + 1));

    payment_id
}

/// Confirm session completion and release payment
pub fn confirm_session(
    env: &Env,
    provider: Address,
    payment_id: BytesN<32>,
    actual_duration: u64,
    session_notes: String,
) {
    // Check if contract is paused
    if is_contract_paused(env) {
        panic!("Contract is currently paused");
    }

    let storage = env.storage().instance();
    
    // Get payment status
    let status: u32 = storage
        .get(&(PAY_STAT, payment_id.clone()))
        .unwrap_or_else(|| panic!("Payment not found"));

    // Verify provider authorization
    let payment_provider: Address = storage
        .get(&(PAY_PROV, payment_id.clone()))
        .unwrap_or_else(|| panic!("Payment not found"));

    if payment_provider != provider {
        panic!("Unauthorized: Only the assigned provider can confirm session");
    }

    // Check payment status
    if status != STATUS_PENDING && status != STATUS_IN_PROGRESS {
        panic!("Payment cannot be confirmed in current status");
    }

    // Calculate actual payment amount and fees
    let provider_hourly_rate = get_provider_hourly_rate(env, provider.clone());
    let actual_amount = calculate_payment_amount(provider_hourly_rate, actual_duration);
    let platform_fee_percentage = get_platform_fee_percentage(env);
    
    let (platform_fee, provider_fee) = calculate_fees(actual_amount, platform_fee_percentage);

    // Update payment details
    storage.set(&(PAY_STAT, payment_id.clone()), &STATUS_COMPLETED);
    storage.set(&(PAY_AMT, payment_id.clone()), &actual_amount);
    storage.set(&(PAY_DUR, payment_id.clone()), &actual_duration);
    storage.set(&(PAY_CMP, payment_id.clone()), &env.ledger().timestamp());
}

/// Process refund for canceled or incomplete sessions
pub fn refund_payment(
    env: &Env,
    caller: Address,
    payment_id: BytesN<32>,
    reason: String,
) {
    let storage = env.storage().instance();
    
    // Get payment status
    let status: u32 = storage
        .get(&(PAY_STAT, payment_id.clone()))
        .unwrap_or_else(|| panic!("Payment not found"));

    // Check authorization
    let is_admin = is_admin(env, &caller);
    let patient: Address = storage
        .get(&(PAY_PAT, payment_id.clone()))
        .unwrap_or_else(|| panic!("Payment not found"));
    let provider: Address = storage
        .get(&(PAY_PROV, payment_id.clone()))
        .unwrap_or_else(|| panic!("Payment not found"));

    let is_patient = patient == caller;
    let is_provider = provider == caller;

    if !is_admin && !is_patient && !is_provider {
        panic!("Unauthorized: Only patient, provider, or admin can request refund");
    }

    // Check if refund is possible
    if status == STATUS_REFUNDED {
        panic!("Payment already refunded");
    }

    if status == STATUS_COMPLETED {
        panic!("Cannot refund completed payment");
    }

    // Update payment status
    storage.set(&(PAY_STAT, payment_id.clone()), &STATUS_REFUNDED);
}

/// Get payment status
pub fn get_payment_status(env: &Env, payment_id: BytesN<32>) -> u32 {
    let storage = env.storage().instance();
    storage
        .get(&(PAY_STAT, payment_id))
        .unwrap_or_else(|| panic!("Payment not found"))
}

/// Get payment amount
pub fn get_payment_amount(env: &Env, payment_id: BytesN<32>) -> i128 {
    let storage = env.storage().instance();
    storage
        .get(&(PAY_AMT, payment_id))
        .unwrap_or_else(|| panic!("Payment not found"))
}

/// Get balance for an address
pub fn get_balance(env: &Env, address: Address) -> (i128, i128) {
    let storage = env.storage().instance();
    
    // Get all payments for the address
    let patient_payments: Vec<BytesN<32>> = storage
        .get(&(PAT_PAY, address.clone()))
        .unwrap_or(Vec::new(env));
    
    let provider_payments: Vec<BytesN<32>> = storage
        .get(&(PROV_PAY, address.clone()))
        .unwrap_or(Vec::new(env));

    let mut pending_amount = 0i128;
    let mut completed_amount = 0i128;

    // Calculate pending amounts (as patient)
    for payment_id in patient_payments.iter() {
        let status: u32 = storage
            .get(&(PAY_STAT, payment_id.clone()))
            .unwrap_or(0);
        let amount: i128 = storage
            .get(&(PAY_AMT, payment_id.clone()))
            .unwrap_or(0);

        match status {
            STATUS_PENDING | STATUS_IN_PROGRESS => {
                pending_amount += amount;
            }
            STATUS_COMPLETED => {
                completed_amount += amount;
            }
            _ => {}
        }
    }

    // Calculate completed amounts (as provider)
    for payment_id in provider_payments.iter() {
        let status: u32 = storage
            .get(&(PAY_STAT, payment_id.clone()))
            .unwrap_or(0);
        let amount: i128 = storage
            .get(&(PAY_AMT, payment_id.clone()))
            .unwrap_or(0);

        if status == STATUS_COMPLETED {
            // Calculate provider fee (simplified)
            let platform_fee_percentage = get_platform_fee_percentage(env);
            let (_, provider_fee) = calculate_fees(amount, platform_fee_percentage);
            completed_amount += provider_fee;
        }
    }

    (pending_amount, completed_amount)
}

/// Calculate payment amount based on hourly rate and duration
fn calculate_payment_amount(hourly_rate: i128, duration_minutes: u64) -> i128 {
    if duration_minutes == 0 {
        return 0;
    }
    
    // Convert duration from minutes to hours with 2 decimal precision
    let duration_hours = (duration_minutes as i128 * 100) / 60; // Convert to hundredths of hours
    (hourly_rate * duration_hours) / 100
}

/// Calculate fees for a payment amount
fn calculate_fees(amount: i128, platform_fee_percentage: u32) -> (i128, i128) {
    if amount <= 0 {
        return (0, 0);
    }
    
    let platform_fee = (amount * platform_fee_percentage as i128) / 10000; // Convert percentage to basis points
    let provider_fee = amount - platform_fee;
    
    (platform_fee, provider_fee)
}

/// Add payment to address payment list
fn add_payment_to_address(env: &Env, address: &Address, payment_id: &BytesN<32>, role: &str) {
    let storage = env.storage().instance();
    let key = if role == "patient" {
        &PAT_PAY
    } else {
        &PROV_PAY
    };

    let mut payments: Vec<BytesN<32>> = storage
        .get(&(key.clone(), address.clone()))
        .unwrap_or(Vec::new(env));

    payments.push_back(payment_id.clone());
    storage.set(&(key.clone(), address.clone()), &payments);
}

/// Generate unique payment ID
fn generate_payment_id(
    env: &Env,
    patient: &Address,
    provider: &Address,
    session_id: &BytesN<32>,
) -> BytesN<32> {
    // Create a deterministic payment ID based on patient, provider, session, and timestamp
    let timestamp = env.ledger().timestamp();
    let nonce = env.ledger().sequence();
    
    // Simple hash-like approach using timestamp and nonce
    let mut data = [0u8; 32];
    data[0..8].copy_from_slice(&timestamp.to_le_bytes());
    data[8..12].copy_from_slice(&nonce.to_le_bytes());
    
    BytesN::from_array(env, &data)
}

/// Check if caller is admin
fn is_admin(env: &Env, caller: &Address) -> bool {
    let storage = env.storage().instance();
    let admin: Address = storage.get(&Symbol::short("ADMIN")).unwrap();
    admin == *caller
}

/// Check if contract is paused
fn is_contract_paused(env: &Env) -> bool {
    let storage = env.storage().instance();
    storage.get(&Symbol::short("PAUSED")).unwrap_or(false)
}

/// Get provider hourly rate
fn get_provider_hourly_rate(env: &Env, provider: Address) -> i128 {
    let storage = env.storage().instance();
        storage
            .get(&(Symbol::short("PROV_RAT"), provider))
            .unwrap_or(0)
}

/// Get platform fee percentage
fn get_platform_fee_percentage(env: &Env) -> u32 {
    let storage = env.storage().instance();
    storage
        .get(&Symbol::short("FEE_PCT"))
        .unwrap_or(200) // Default 2%
}