# Carbon Credit Registry Smart Contract

A comprehensive Soroban smart contract for managing carbon credits on the Stellar network, enabling the issuance, trading, and retirement of carbon credits to support carbon neutrality goals.

## 🎯 Overview

This contract provides a complete solution for carbon credit management, including:
- **Issuer Management**: Registration and verification of carbon credit issuers
- **Credit Issuance**: Creation of verified carbon credits with standards compliance
- **Trading Platform**: Secure transfer of carbon credits between parties
- **Retirement System**: Permanent retirement of credits for carbon offset claims
- **Event Tracking**: Complete audit trail of all credit lifecycle events

## 🏗 Contract Structure

```
carbon-credit-registry/
├── src/
│   ├── lib.rs                # Main contract interface and core structures
│   ├── credits.rs            # Credit issuance and lifecycle management
│   ├── trading.rs            # Trading and retirement functionality
│   ├── utils.rs              # Validation and utility functions
│   └── test.rs               # Test suite
├── Cargo.toml                # Rust package configuration
├── Makefile                  # Build and deployment automation
└── README.md                 # This documentation
```

## 📋 Prerequisites

- Rust (latest stable version)
- Stellar CLI tools (`stellar` command)
- Soroban CLI tools (`stellar contract build`)

## 🚀 Quick Start

### 1. Building the Contract

```bash
# Navigate to the contract directory
cd soroban/contracts/carbon-credit-registry

# Build the contract
cargo build

# Build for Stellar network
stellar contract build
```

### 2. Contract Initialization

```rust
// Configure admin and fee rates (in basis points)
let admin = Address::generate(env);
let trading_fee_rate = 25u32; // 0.25%
let retirement_fee_rate = 10u32; // 0.1%

client.initialize(&admin, &trading_fee_rate, &retirement_fee_rate);
```

### 3. Register an Issuer

```rust
let issuer = Address::generate(env);
let name = String::from_str(&env, "Green Energy Corp");
let mut standards = Vec::new(&env);
standards.push_back(String::from_str(&env, "VERRA"));
standards.push_back(String::from_str(&env, "GOLD_STANDARD"));

client.register_issuer(&issuer, &name, &standards);
```

### 4. Issue Carbon Credits

```rust
let project_type = String::from_str(&env, "RENEWABLE_ENERGY");
let project_location = String::from_str(&env, "Brazil");
let verification_standard = String::from_str(&env, "VERRA");
let vintage_year = 2024u32;
let quantity = 1000i128; // tons of CO2 equivalent
let verification_hash = BytesN::from_array(&env, &[/* verification data hash */]);
let mut metadata = Map::new(&env);
metadata.set(String::from_str(&env, "project_id"), String::from_str(&env, "PROJ-001"));

let credit_id = client.issue_credit(
    &issuer,
    &project_type,
    &project_location,
    &verification_standard,
    &vintage_year,
    &quantity,
    &verification_hash,
    &metadata
);
```

## 📊 Key Features

### 🔐 Role-Based Access Control

- **Admin**: Contract initialization and issuer registration
- **Issuers**: Can issue new carbon credits
- **Users**: Can trade and retire credits they own

### 📈 Supported Standards

- **VERRA** - Verified Carbon Standard
- **GOLD_STANDARD** - Gold Standard for the Global Goals
- **CARBON_CREDIT_STANDARD** - Generic carbon credit standard
- **AMERICAN_CARBON_REGISTRY** - U.S. based carbon market
- **CLIMATE_ACTION_RESERVE** - Climate Action Reserve

### 🌱 Project Types

- **RENEWABLE_ENERGY** - Solar, wind, hydro projects
- **FOREST_CONSERVATION** - Protected forest areas
- **REFORESTATION** - Tree planting initiatives
- **ENERGY_EFFICIENCY** - Energy optimization projects
- **WASTE_MANAGEMENT** - Waste reduction programs
- **TRANSPORTATION** - Low-emission transport
- **AGRICULTURE** - Sustainable farming practices
- **CARBON_CAPTURE** - Direct air capture technology

## 🔧 API Reference

### Contract Functions

#### `initialize(admin: Address, trading_fee_rate: u32, retirement_fee_rate: u32)`
Initializes the contract with admin privileges and fee structure.

#### `register_issuer(issuer_address: Address, name: String &verification_standards: Vec<String>)`
Registers a new carbon credit issuer.

#### `issue_credit(`
- `issuer: Address`
- `project_type: String`
- `project_location: String`
- `verification_standard: String`
- `vintage_year: u32`
- `quantity: i128`
- `verification_hash: BytesN<32>`
- `metadata: Map<String, String>`
``) -> BytesN<32>`

Issues a new carbon credit and returns its unique ID.

#### `trade_credit(params: TradingParams)`
Transfers ownership of a carbon credit.

#### `retire_credit(params: RetirementParams)`
Retires a carbon credit for offset claims.

#### `get_credit_status(credit_id: BytesN<32>) -> Option<CarbonCredit>`
Retrieves detailed information about a specific credit.

#### `get_credit_history(credit_id: BytesN<32>) -> Vec<CreditEvent>`
Returns the complete transaction history for a credit.

#### `get_contract_stats() -> (u32, u32, i128, i128)`
Returns contract statistics: (issuer_count, credit_count, total_issued, total_retired).

### Data Structures

#### CarbonCredit
```rust
pub struct CarbonCredit {
    pub id: BytesN<32>,                    // Unique identifier
    pub issuer: Address,                   // Issuer address
    pub project_type: String,              // Type of project
    pub project_location: String,          // Geographic location
    pub verification_standard: String,     // Standard used
    pub issuance_date: u64,                // Unix timestamp
    pub vintage_year: u32,                 // Project vintage year
    pub quantity: i128,                    // CO2 equivalent (tons)
    pub current_owner: Address,            // Current owner
    pub status: CreditStatus,              // Current status
    pub verification_hash: BytesN<32>,     // Verification hash
    pub metadata: Map<String, String>,    // Additional data
}
```

#### CreditStatus
```rust
pub enum CreditStatus {
    Issued,    // Available for trading
    Traded,    // Previously traded
    Retired,   // Retired for offset
    Suspended, // Temporarily suspended
}
```

## 🛠 Development

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_initialization
```

### Contract Deployment

```bash
# Build optimized contract
make build-release

# Deploy to testnet (requires Stellar CLI setup)
stellar contract deploy target/soroban/carbon_credit_registry.wasm
```

## 🔍 Events and Logging

The contract maintains a complete audit trail of all operations:

- **Issuance Events**: Credit creation with full project details
- **Trade Events**: Ownership transfers with pricing data
- **Retirement Events**: Retirement for carbon offset purposes
- **Suspension Events**: Administrative actions

## 💡 Usage Examples

### Corporate Carbon Neutrality

```rust
// Company buys credits
let trading_params = TradingParams {
    credit_id: credit_id,
    from: seller,
    to: company_address,
    quantity: 500i128,
    price: 50000i128, // Price in base units
    payment_token: token_address,
};

client.trade_credit(&trading_params);

// Company retires credits for offset claims
let retirement_params = RetirementParams {
    credit_id: credit_id,
    owner: company_address,
    quantity: 500i128,
    retirement_reason: String::from_str(&env, "Scope 1 emissions offset"),
    retirement_certificate: certificate_hash,
};

client.retire_credit(&retirement_params);
```

### Verification and Compliance

```rust
// Check credit authenticity
let credit = client.get_credit_status(&credit_id).unwrap();
assert_eq!(credit.verification_standard, "VERRA");
assert_eq!(credit.status, CreditStatus::Issued);

// Verify issuer credentials
let issuer_profile = client.get_issuer_profile(&issuer_address);
assert!(issuer_profile.unwrap().is_active);
```

## 🚨 Error Handling

The contract uses structured error handling for all operations:

- **CreditNotFound**: Invalid credit ID
- **UnauthorizedIssuer**: Issuer not registered
- **InsufficientBalance**: Insufficient credit quantity
- **InvalidQuantity**: Invalid amount specified
- **CreditAlreadyRetired**: Cannot operate on retired credits
- **CreditSuspended**: Cannot operate on suspended credits

