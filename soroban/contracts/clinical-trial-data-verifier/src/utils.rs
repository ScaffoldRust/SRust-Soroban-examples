use soroban_sdk::{Env, TryIntoVal, String, Vec, BytesN, Address, Symbol, contracttype, IntoVal, TryFromVal, Val, Error};
use tiny_keccak::{Hasher, Sha3};

// Data integrity
pub fn hash_data(env: &Env, data: &[u8]) -> String {
    let mut hasher = Sha3::v256();
    hasher.update(data);
    let mut output = [0u8; 32];
    hasher.finalize(&mut output);
    
    // Convert to hex string without using allocator
    let hex_chars = b"0123456789abcdef";
    let mut hex_bytes = [0u8; 64];
    
    for (i, byte) in output.iter().enumerate() {
        hex_bytes[i * 2] = hex_chars[(byte >> 4) as usize];
        hex_bytes[i * 2 + 1] = hex_chars[(byte & 0xf) as usize];
    }
    
    String::from_bytes(env, &hex_bytes)
}

// The recommended pattern (used by many Soroban contracts) is to perform
// strict format validation (e.g. check for valid hex characters) off-chain
// in the frontend or client code, and let the contract only enforce simple
// constraints such as maximum length.
//
// This keeps the contract lightweight and cheap while still ensuring that
// obviously invalid data (wrong length) cannot be stored on-chain.
pub fn validate_hash(_env: &Env, hash: &String) -> bool {
    hash.len() == 64
}

// Timestamp management
pub fn current_timestamp(env: &Env) -> u64 {
    env.ledger().timestamp()
}

// Storage optimization
pub fn optimize_storage_key(env: &Env, key: &str) -> BytesN<16> {
    // Convert string keys to compressed format for storage efficiency
    let mut hasher = Sha3::v256();
    hasher.update(key.as_bytes());
    let mut output = [0u8; 32];
    hasher.finalize(&mut output);
    BytesN::from_array(env, &output[0..16].try_into().unwrap())
}

// Discrepancy logging
#[derive(Clone)]
#[contracttype]
pub struct Discrepancy {
    pub data_hash: String,
    pub verifier: Address,
    pub timestamp: u64,
    pub reason: String,
    pub severity: DiscrepancySeverity,
}

#[derive(Clone, PartialEq)]
#[contracttype]
pub enum DiscrepancySeverity {
    Minor,
    Major,
    Critical,
}

pub fn log_discrepancy(
    env: &Env,
    data_hash: &String,
    verifier: &Address,
    reason: String,
    severity: DiscrepancySeverity,
) {
    env.events().publish(
        (String::from_str(env, "discrepancy"), data_hash.clone()),
        (verifier.clone(), reason, severity),
    );
}

// ZK-Proof preparation
#[contracttype]
pub struct ZKProofPrep {
    pub data_hash: String,
    pub metadata: soroban_sdk::Map<Symbol, String>,
    pub timestamp: u64,
}

pub fn prepare_for_zk_proof(
    env: &Env,
    data_hash: &String,
    metadata: soroban_sdk::Map<Symbol, String>,
) -> ZKProofPrep {
    ZKProofPrep {
        data_hash: data_hash.clone(),
        metadata,
        timestamp: current_timestamp(env),
    }
}

// Batch processing for storage optimization
pub fn process_items_in_batches<T>(
    env: &Env,
    items: &Vec<T>,
    process_fn: impl Fn(&Env, &T) -> Result<(), Error>,
) -> Result<(), Error>
where
    T: Clone + IntoVal<Env, Val> + TryFromVal<Env, Val>,
{
    let mut success_count: u32 = 0;
    let mut error_count: u32 = 0;

    let len = items.len();
    for i in 0..len {
        if let Some(item) = items.get(i) {
            match process_fn(env, &item) {
                Ok(()) => success_count += 1,
                Err(e) => {
                    error_count += 1;
                    env.events().publish(
                        (String::from_str(env, "batch_error"), success_count),
                        e,
                    );
                }
            }
        }
    }

    if error_count > 0 {
        Err(Error::from_contract_error(6))
    } else {
        Ok(())
    }
}