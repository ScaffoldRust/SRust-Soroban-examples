
use crate::{
    SecureMedicalRecordsContract, SecureMedicalRecordsContractClient,
    AccessLevel, SensitivityLevel, RecordMetadata, ProviderInfo
};
use soroban_sdk::{testutils::{Address as _, Ledger}, Address, BytesN, Env, String};

/// Test environment setup
pub struct TestEnvironment {
    pub env: Env,
    pub contract: SecureMedicalRecordsContractClient<'static>,
    pub admin: Address,
    pub patient: Address,
    pub provider1: Address,
    pub provider2: Address,
    pub emergency_contact: Address,
}

impl TestEnvironment {
    pub fn new() -> Self {
        let env = Env::default();
        env.mock_all_auths();
        
        let contract_id = env.register(SecureMedicalRecordsContract, ());
        let contract = SecureMedicalRecordsContractClient::new(&env, &contract_id);
        
        let admin = Address::generate(&env);
        let patient = Address::generate(&env);
        let provider1 = Address::generate(&env);
        let provider2 = Address::generate(&env);
        let emergency_contact = Address::generate(&env);
        
        // Initialize the contract
        contract.initialize(&admin, &None);
        
        // Register providers
        let provider1_info = create_test_provider_info(&env, "LIC001", "Dr. Alice Smith", "Cardiology", true, true);
        let provider2_info = create_test_provider_info(&env, "LIC002", "Dr. Bob Jones", "Neurology", true, false);
        
        contract.register_provider(&admin, &provider1, &provider1_info);
        contract.register_provider(&admin, &provider2, &provider2_info);
        
        TestEnvironment {
            env,
            contract,
            admin,
            patient,
            provider1,
            provider2,
            emergency_contact,
        }
    }
    
    /// Create a test medical record
    pub fn create_test_record(&self, record_type: &str, sensitivity: SensitivityLevel) -> BytesN<32> {
        let record_hash = BytesN::from_array(&self.env, &[0u8; 32]);
        let metadata = create_test_metadata(&self.env, "Test Record", "Test Description", sensitivity);
        
        self.contract.create_record(
            &self.patient,
            &record_hash,
            &String::from_str(&self.env, record_type),
            &metadata,
        )
    }
    
    /// Fast forward time by specified seconds
    pub fn advance_time(&self, seconds: u64) {
        let current_time = self.env.ledger().timestamp();
        self.env.ledger().set_timestamp(current_time + seconds);
    }
    
    /// Generate a non-existent record ID for testing
    pub fn generate_fake_record_id(&self) -> BytesN<32> {
        BytesN::from_array(&self.env, &[0xFFu8; 32])
    }
    
    /// Create multiple test records for bulk operations
    pub fn create_multiple_records(&self, count: u32, record_type: &str) -> soroban_sdk::Vec<BytesN<32>> {
        let mut records = soroban_sdk::Vec::new(&self.env);
        
        for i in 0..count {
            let mut hash = [0u8; 32];
            hash[0] = (i & 0xFF) as u8;
            hash[1] = ((i >> 8) & 0xFF) as u8;
            
            let record_hash = BytesN::from_array(&self.env, &hash);
            let metadata = create_test_metadata(&self.env, "Test Record", "Test Description", SensitivityLevel::Medium);
            
            let record_id = self.contract.create_record(
                &self.patient,
                &record_hash,
                &String::from_str(&self.env, record_type),
                &metadata,
            );
            
            records.push_back(record_id);
        }
        
        records
    }
}

/// Create test provider information
pub fn create_test_provider_info(
    env: &Env,
    license: &str,
    name: &str,
    specialty: &str,
    verified: bool,
    emergency_contact: bool,
) -> ProviderInfo {
    ProviderInfo {
        license_number: String::from_str(env, license),
        name: String::from_str(env, name),
        specialty: String::from_str(env, specialty),
        organization: String::from_str(env, "Test Hospital"),
        verified,
        verification_date: env.ledger().timestamp(),
        emergency_contact,
    }
}

/// Create test record metadata
pub fn create_test_metadata(
    env: &Env,
    title: &str,
    description: &str,
    sensitivity: SensitivityLevel,
) -> RecordMetadata {
    RecordMetadata {
        title: String::from_str(env, title),
        description: String::from_str(env, description),
        provider_id: None,
        category: String::from_str(env, "General"),
        sensitivity_level: sensitivity,
        retention_period: 365 * 24 * 60 * 60, // 1 year in seconds
        patient_notes: Some(String::from_str(env, "Patient notes")),
    }
}

/// Create test record with specific hash
pub fn create_record_with_hash(env: &Env, hash_bytes: [u8; 32]) -> BytesN<32> {
    BytesN::from_array(env, &hash_bytes)
}

/// Helper to generate unique record hashes for testing
pub fn generate_test_hash(seed: u32) -> [u8; 32] {
    let mut hash = [0u8; 32];
    hash[0] = (seed & 0xFF) as u8;
    hash[1] = ((seed >> 8) & 0xFF) as u8;
    hash[2] = ((seed >> 16) & 0xFF) as u8;
    hash[3] = ((seed >> 24) & 0xFF) as u8;
    hash
}

/// Constants for testing
pub const ONE_HOUR: u64 = 60 * 60;
pub const ONE_DAY: u64 = 24 * 60 * 60;
pub const ONE_WEEK: u64 = 7 * 24 * 60 * 60;
pub const ONE_MONTH: u64 = 30 * 24 * 60 * 60;
pub const ONE_YEAR: u64 = 365 * 24 * 60 * 60;

/// Test record types
pub const RECORD_TYPE_LAB_RESULT: &str = "lab_result";
pub const RECORD_TYPE_PRESCRIPTION: &str = "prescription";
pub const RECORD_TYPE_DIAGNOSIS: &str = "diagnosis";
pub const RECORD_TYPE_EMERGENCY: &str = "emergency_record";

/// Helper to create unregistered provider address
pub fn create_unregistered_provider(env: &Env) -> Address {
    Address::generate(env)
}

