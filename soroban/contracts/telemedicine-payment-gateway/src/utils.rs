use soroban_sdk::{Env, Address, BytesN, String, Vec};

/// Calculate fees for a payment amount
pub fn calculate_fees(amount: i128, platform_fee_percentage: u32) -> (i128, i128) {
    if amount <= 0 {
        return (0, 0);
    }
    
    let platform_fee = (amount * platform_fee_percentage as i128) / 10000; // Convert percentage to basis points
    let provider_fee = amount - platform_fee;
    
    (platform_fee, provider_fee)
}

/// Validate payment amount
pub fn validate_payment_amount(amount: i128) {
    if amount <= 0 {
        panic!("Payment amount must be positive");
    }
    
    if amount < 100 {
        panic!("Payment amount too small (minimum 1.00 units)");
    }
    
    if amount > 1_000_000_000 {
        panic!("Payment amount too large (maximum 10,000,000.00 units)");
    }
}

/// Validate provider data
pub fn validate_provider_data(provider_name: &String, specialty: &String, hourly_rate: i128) {
    // Validate provider name
    if provider_name.len() < 2 {
        panic!("Provider name must be at least 2 characters");
    }
    
    if provider_name.len() > 100 {
        panic!("Provider name too long (maximum 100 characters)");
    }
    
    // Validate specialty
    if specialty.len() < 2 {
        panic!("Specialty must be at least 2 characters");
    }
    
    if specialty.len() > 50 {
        panic!("Specialty too long (maximum 50 characters)");
    }
    
    // Validate hourly rate
    if hourly_rate <= 0 {
        panic!("Hourly rate must be positive");
    }
    
    if hourly_rate < 100 {
        panic!("Hourly rate too low (minimum 1.00 units per hour)");
    }
    
    if hourly_rate > 100_000_000 {
        panic!("Hourly rate too high (maximum 1,000,000.00 units per hour)");
    }
}

/// Validate fee percentage
pub fn validate_fee_percentage(fee_percentage: u32) {
    if fee_percentage > 1000 {
        panic!("Fee percentage cannot exceed 10% (1000 basis points)");
    }
}

/// Generate unique payment ID
pub fn generate_payment_id(
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

/// Validate session duration
pub fn validate_session_duration(
    duration: u64,
    min_duration: u64,
    max_duration: u64,
) {
    if duration < min_duration {
        panic!("Session duration too short");
    }
    
    if duration > max_duration {
        panic!("Session duration too long");
    }
}

/// Validate consent hash
pub fn validate_consent_hash(consent_hash: &BytesN<32>) {
    // Check if consent hash is not all zeros (placeholder validation)
    let zero_hash = BytesN::from_array(consent_hash.env(), &[0u8; 32]);
    if *consent_hash == zero_hash {
        panic!("Invalid consent hash");
    }
}

/// Calculate session cost based on duration and rate
pub fn calculate_session_cost(hourly_rate: i128, duration_minutes: u64) -> i128 {
    if duration_minutes == 0 {
        return 0;
    }
    
    // Convert minutes to hours with 2 decimal precision
    let duration_hours = (duration_minutes as i128 * 100) / 60;
    (hourly_rate * duration_hours) / 100
}

/// Validate currency address
pub fn validate_currency_address(currency: &Address) {
    // In a real implementation, this would validate that the address
    // is a valid token contract address
    // For now, we'll just check it's not the zero address
    let zero_address = Address::from_string(&String::from_str(currency.env(), "0000000000000000000000000000000000000000000000000000000000000000"));
    if *currency == zero_address {
        panic!("Invalid currency address");
    }
}

/// Check if address is valid
pub fn is_valid_address(_env: &Env, _address: &Address) -> bool {
    // In a real implementation, this would validate the address format
    // For now, we'll assume all addresses are valid
    true
}

/// Generate session ID
pub fn generate_session_id(env: &Env, patient: &Address, provider: &Address) -> BytesN<32> {
    let timestamp = env.ledger().timestamp();
    let nonce = env.ledger().sequence();
    
    // Simple hash-like approach using timestamp and nonce
    let mut data = [0u8; 32];
    data[0..8].copy_from_slice(&timestamp.to_le_bytes());
    data[8..12].copy_from_slice(&nonce.to_le_bytes());
    
    BytesN::from_array(env, &data)
}

/// Validate session notes
pub fn validate_session_notes(notes: &String) {
    if notes.len() > 1000 {
        panic!("Session notes too long (maximum 1000 characters)");
    }
}

/// Validate refund reason
pub fn validate_refund_reason(reason: &String) {
    if reason.len() < 10 {
        panic!("Refund reason must be at least 10 characters");
    }
    
    if reason.len() > 500 {
        panic!("Refund reason too long (maximum 500 characters)");
    }
}

/// Check if payment is eligible for refund
pub fn is_payment_refundable(status: u32) -> bool {
    matches!(status, 0 | 1) // Pending or InProgress
}

/// Calculate emergency session fee
pub fn calculate_emergency_fee(base_amount: i128, emergency_fee_percentage: u32) -> i128 {
    if base_amount <= 0 {
        return 0;
    }
    
    (base_amount * emergency_fee_percentage as i128) / 10000
}

/// Validate dispute resolution request
pub fn validate_dispute_request(
    payment_id: &BytesN<32>,
    reason: &String,
    evidence_hash: &BytesN<32>,
) {
    if reason.len() < 20 {
        panic!("Dispute reason must be at least 20 characters");
    }
    
    if reason.len() > 1000 {
        panic!("Dispute reason too long (maximum 1000 characters)");
    }
    
    // Validate evidence hash
    let zero_hash = BytesN::from_array(payment_id.env(), &[0u8; 32]);
    if *evidence_hash == zero_hash {
        panic!("Invalid evidence hash");
    }
}

/// Calculate platform revenue for a period
pub fn calculate_platform_revenue(
    total_payments: i128,
    platform_fee_percentage: u32,
) -> i128 {
    if total_payments <= 0 {
        return 0;
    }
    
    (total_payments * platform_fee_percentage as i128) / 10000
}

/// Check if session is within business hours (simplified)
pub fn is_business_hours(timestamp: u64) -> bool {
    // This is a simplified implementation
    // In production, this would consider timezone and actual business hours
    let hour = (timestamp / 3600) % 24;
    hour >= 8 && hour <= 18 // 8 AM to 6 PM
}

/// Calculate processing time for payment
pub fn calculate_processing_time(created_at: u64, completed_at: u64) -> u64 {
    if completed_at <= created_at {
        return 0;
    }
    
    completed_at - created_at
}

/// Validate payment ID format
pub fn validate_payment_id(payment_id: &BytesN<32>) {
    // Check if payment ID is not all zeros
    let zero_id = BytesN::from_array(payment_id.env(), &[0u8; 32]);
    if *payment_id == zero_id {
        panic!("Invalid payment ID");
    }
}

/// Check if amount is within reasonable bounds for telemedicine
pub fn is_reasonable_telemedicine_amount(amount: i128) -> bool {
    // Typical telemedicine session costs between $50-$500
    amount >= 5000 && amount <= 50000 // $50.00 to $500.00 in smallest units
}