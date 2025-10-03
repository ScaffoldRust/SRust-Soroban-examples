use soroban_sdk::{Address, Bytes, BytesN, Env};

/// Generate a unique credit ID based on issuer, verification hash, and timestamp
pub fn generate_credit_id(
    env: &Env,
    _issuer: &Address,
    verification_hash: &BytesN<32>,
) -> BytesN<32> {
    // Simplified approach - use timestamp and verification hash
    let timestamp = env.ledger().timestamp();
    let timestamp_bytes = timestamp.to_be_bytes();

    // Create a simple hash from timestamp and verification hash
    let mut combined = [0u8; 40];
    combined[0..8].copy_from_slice(&timestamp_bytes);
    combined[8..40].copy_from_slice(&verification_hash.to_array());

    let bytes_data = Bytes::from_slice(env, &combined);
    env.crypto().sha256(&bytes_data).into()
}
