use soroban_sdk::{Address, Bytes, BytesN, Env};

/// Generates a unique REC ID based on issuer and verification hash
pub fn generate_rec_id(
    env: &Env,
    _issuer: &Address,
    verification_hash: &BytesN<32>,
) -> BytesN<32> {
    let timestamp = env.ledger().timestamp();
    let timestamp_bytes = timestamp.to_be_bytes();

    // Combine timestamp and verification hash to create unique ID
    let mut combined = [0u8; 40];
    combined[0..8].copy_from_slice(&timestamp_bytes);
    combined[8..40].copy_from_slice(&verification_hash.to_array());

    let bytes_data = Bytes::from_slice(env, &combined);
    env.crypto().sha256(&bytes_data).into()
}

/// Validates energy capacity value
pub fn validate_capacity(capacity_mwh: i128) -> bool {
    capacity_mwh > 0
}

/// Validates production date (must not be in the future)
pub fn validate_production_date(env: &Env, production_date: u64) -> bool {
    production_date <= env.ledger().timestamp()
}

/// Validates verification hash (must not be empty)
pub fn validate_verification_hash(verification_hash: &BytesN<32>) -> bool {
    let empty_hash = [0u8; 32];
    verification_hash.to_array() != empty_hash
}

/// Checks if an address is authorized as an issuer
pub fn is_authorized_issuer(env: &Env, issuer: &Address) -> bool {
    use crate::{DataKey, IssuerInfo};

    let issuer_key = DataKey::Issuer(issuer.clone());
    if let Some(issuer_info) = env.storage().persistent().get::<DataKey, IssuerInfo>(&issuer_key)
    {
        issuer_info.authorized
    } else {
        false
    }
}

/// Validates REC transfer parameters
pub fn validate_transfer(
    capacity_mwh: i128,
    from: &Address,
    to: &Address,
) -> bool {
    capacity_mwh > 0 && from != to
}

/// Validates retirement parameters
pub fn validate_retirement(capacity_mwh: i128) -> bool {
    capacity_mwh > 0
}

/// Creates a transaction hash for events
pub fn create_transaction_hash(
    env: &Env,
    rec_id: &BytesN<32>,
    timestamp: u64,
) -> BytesN<32> {
    let timestamp_bytes = timestamp.to_be_bytes();

    let mut combined = [0u8; 40];
    combined[0..32].copy_from_slice(&rec_id.to_array());
    combined[32..40].copy_from_slice(&timestamp_bytes);

    let bytes_data = Bytes::from_slice(env, &combined);
    env.crypto().sha256(&bytes_data).into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::Address as _;

    #[test]
    fn test_validate_capacity() {
        assert!(validate_capacity(100));
        assert!(validate_capacity(1));
        assert!(!validate_capacity(0));
        assert!(!validate_capacity(-100));
    }

    #[test]
    fn test_validate_transfer() {
        use soroban_sdk::testutils::Address as _;

        let env = Env::default();
        let addr1 = Address::generate(&env);
        let addr2 = Address::generate(&env);

        assert!(validate_transfer(100, &addr1, &addr2));
        assert!(!validate_transfer(0, &addr1, &addr2));
        assert!(!validate_transfer(-100, &addr1, &addr2));
        assert!(!validate_transfer(100, &addr1, &addr1));
    }

    #[test]
    fn test_validate_retirement() {
        assert!(validate_retirement(100));
        assert!(validate_retirement(1));
        assert!(!validate_retirement(0));
        assert!(!validate_retirement(-100));
    }

    #[test]
    fn test_validate_verification_hash() {
        let env = Env::default();

        let valid_hash = BytesN::from_array(&env, &[1u8; 32]);
        assert!(validate_verification_hash(&valid_hash));

        let empty_hash = BytesN::from_array(&env, &[0u8; 32]);
        assert!(!validate_verification_hash(&empty_hash));
    }

    #[test]
    fn test_generate_rec_id() {
        use soroban_sdk::testutils::Ledger as _;

        let env = Env::default();
        let issuer = Address::generate(&env);
        let verification_hash1 = BytesN::from_array(&env, &[1u8; 32]);
        let verification_hash2 = BytesN::from_array(&env, &[2u8; 32]);

        let rec_id1 = generate_rec_id(&env, &issuer, &verification_hash1);
        let rec_id2 = generate_rec_id(&env, &issuer, &verification_hash2);

        // IDs should be different due to different verification hashes
        assert_ne!(rec_id1, rec_id2);
    }
}
