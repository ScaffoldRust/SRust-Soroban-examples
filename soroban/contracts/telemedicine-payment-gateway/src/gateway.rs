use soroban_sdk::{Env, Address, Symbol, String, Vec};

/// Storage keys (max 9 characters for short symbols)
const ADMIN: Symbol = Symbol::short("ADMIN");
const PROVIDERS: Symbol = Symbol::short("PROVIDERS");
const PROV_NAMES: Symbol = Symbol::short("PROV_NAM");
const PROV_SPEC: Symbol = Symbol::short("PROV_SPEC");
const PROV_RATES: Symbol = Symbol::short("PROV_RAT");
const PROV_CURR: Symbol = Symbol::short("PROV_CUR");
const PROV_ACT: Symbol = Symbol::short("PROV_ACT");
const FEE_PCT: Symbol = Symbol::short("FEE_PCT");
const MIN_DUR: Symbol = Symbol::short("MIN_DUR");
const MAX_DUR: Symbol = Symbol::short("MAX_DUR");
const PAUSED: Symbol = Symbol::short("PAUSED");

/// Initialize the telemedicine payment gateway
pub fn initialize(
    env: &Env,
    admin: Address,
    platform_fee_percentage: u32,
    min_session_duration: u64,
    max_session_duration: u64,
) {
    // Validate fee percentage
    if platform_fee_percentage > 1000 {
        panic!("Fee percentage cannot exceed 10% (1000 basis points)");
    }
    
    // Validate session duration constraints
    if min_session_duration >= max_session_duration {
        panic!("Minimum session duration must be less than maximum");
    }
    
    if min_session_duration < 15 {
        panic!("Minimum session duration must be at least 15 minutes");
    }
    
    if max_session_duration > 480 {
        panic!("Maximum session duration cannot exceed 8 hours");
    }

    let storage = env.storage().instance();
    
    // Set admin
    storage.set(&ADMIN, &admin);

    // Set fee schedule
    storage.set(&FEE_PCT, &platform_fee_percentage);
    storage.set(&MIN_DUR, &min_session_duration);
    storage.set(&MAX_DUR, &max_session_duration);

    // Initialize provider storage
    let providers: Vec<Address> = Vec::new(env);
    storage.set(&PROVIDERS, &providers);

    // Contract starts as active
    storage.set(&PAUSED, &false);
}

/// Register a healthcare provider
pub fn register_provider(
    env: &Env,
    provider_address: Address,
    provider_name: String,
    specialty: String,
    hourly_rate: i128,
    currency: Address,
) {
    // Check authorization
    if !is_admin(env) {
        panic!("Only admin can register providers");
    }

    // Validate provider data
    validate_provider_data(&provider_name, &specialty, hourly_rate);

    let storage = env.storage().instance();
    
    // Check if provider already exists
    let is_active: bool = storage
        .get(&(PROV_ACT, provider_address.clone()))
        .unwrap_or(false);
    
    if is_active {
        panic!("Provider already registered");
    }

    // Store provider configuration using composite keys
    storage.set(&(PROV_NAMES, provider_address.clone()), &provider_name);
    storage.set(&(PROV_SPEC, provider_address.clone()), &specialty);
    storage.set(&(PROV_RATES, provider_address.clone()), &hourly_rate);
    storage.set(&(PROV_CURR, provider_address.clone()), &currency);
    storage.set(&(PROV_ACT, provider_address.clone()), &true);

    // Add to providers list
    let mut providers: Vec<Address> = storage
        .get(&PROVIDERS)
        .unwrap_or(Vec::new(env));
    
    providers.push_back(provider_address.clone());
    storage.set(&PROVIDERS, &providers);
}

/// Get provider hourly rate
pub fn get_provider_hourly_rate(env: &Env, provider: Address) -> i128 {
    let storage = env.storage().instance();
    storage
        .get(&(PROV_RATES, provider))
        .unwrap_or(0)
}

/// Get platform fee percentage
pub fn get_platform_fee_percentage(env: &Env) -> u32 {
    let storage = env.storage().instance();
    storage
        .get(&FEE_PCT)
        .unwrap_or(200) // Default 2%
}

/// Get all active providers
pub fn get_all_providers(env: &Env) -> Vec<Address> {
    let storage = env.storage().instance();
    let providers: Vec<Address> = storage
        .get(&PROVIDERS)
        .unwrap_or(Vec::new(env));
    
    let mut active_providers = Vec::new(env);
    
    for provider in providers.iter() {
        let is_active: bool = storage
            .get(&(PROV_ACT, provider.clone()))
            .unwrap_or(false);
        
        if is_active {
            active_providers.push_back(provider.clone());
        }
    }
    
    active_providers
}

/// Pause contract operations
pub fn pause_contract(env: &Env) {
    if !is_admin(env) {
        panic!("Only admin can pause contract");
    }
    
    let storage = env.storage().instance();
    storage.set(&PAUSED, &true);
}

/// Resume contract operations
pub fn resume_contract(env: &Env) {
    if !is_admin(env) {
        panic!("Only admin can resume contract");
    }
    
    let storage = env.storage().instance();
    storage.set(&PAUSED, &false);
}

/// Check if contract is paused
pub fn is_contract_paused(env: &Env) -> bool {
    let storage = env.storage().instance();
    storage.get(&PAUSED).unwrap_or(false)
}

/// Get contract status
pub fn get_contract_status(env: &Env) -> bool {
    !is_contract_paused(env)
}

/// Validate provider data
fn validate_provider_data(provider_name: &String, specialty: &String, hourly_rate: i128) {
    // Validate provider name length
    if provider_name.len() < 2 {
        panic!("Provider name must be at least 2 characters");
    }
    
    if provider_name.len() > 100 {
        panic!("Provider name too long (maximum 100 characters)");
    }
    
    // Validate specialty length
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

/// Check if caller is admin
fn is_admin(env: &Env) -> bool {
    let storage = env.storage().instance();
    let admin: Address = storage.get(&ADMIN).unwrap();
    
    // In a real implementation, this would check the caller
    // For now, we'll assume this is called from the contract context
    true // This should be replaced with actual caller verification
}