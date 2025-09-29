# Energy Project Funding Escrow Smart Contract

A professional-grade Soroban smart contract for managing energy project funding through secure escrow mechanisms on the Stellar network. This contract ensures transparent, milestone-based fund release for renewable energy and carbon credit projects.

## 🎯 Features

### Core Escrow Functionality
- **Secure Fund Management**: Automated escrow with milestone-based releases
- **Multi-signature Support**: Configurable multi-sig for high-value transactions
- **Refund Protection**: Automated refund processing for failed projects
- **Role-based Access**: Investor and project manager role separation

### Energy-Specific Features
- **Multiple Energy Types**: Support for solar, wind, hydro, geothermal, biomass, nuclear, and hybrid projects
- **Performance Tracking**: Monitor actual vs. expected energy output and carbon offsets
- **Compliance Verification**: Built-in regulatory and environmental compliance checks
- **Carbon Credit Integration**: Ready for integration with carbon credit contracts

### Advanced Features
- **Milestone Management**: Create, track, and verify project milestones
- **Penalty System**: Automatic penalty calculations for overdue milestones
- **Performance Bonuses**: Reward systems for exceeding performance targets
- **Real-time Analytics**: Project progress tracking and performance metrics

## 🏗 Architecture

```
energy-project-funding-escrow/
├── src/
│   ├── lib.rs              # Main contract interface and exports
│   ├── escrow.rs           # Core escrow logic and fund management
│   ├── milestones.rs       # Milestone creation and verification
│   ├── types.rs            # Data structures and enums
│   ├── utils.rs            # Validation utilities and helpers
│   └── test.rs             # Comprehensive test suite
├── Cargo.toml              # Rust dependencies and configuration
├── Makefile                # Build and deployment automation
└── README.md               # This documentation
```

## 🚀 Quick Start

### Installation

```bash
# Clone and navigate to contract directory
cd soroban/contracts/energy-project-funding-escrow

```

### Build and Test

```bash
# Complete development workflow
make dev

# Or run individual steps:
make build        # Build the contract
make test         # Run tests
make optimize     # Optimize WASM binary
```

### Deployment

```bash
# Deploy to testnet
make deploy-testnet

# Initialize the contract
CONTRACT_ID=<your-contract-id> make init-testnet
```

## 📖 Usage Guide

### 1. Initialize a Project

```rust
// Create a new energy project
let project_id = contract.initialize_project(
    investor_address,
    project_manager_address,
    "Solar Farm Alpha",              // Project name
    "100MW solar installation",     // Description
    50_000_000_000_000i128,         // Total funding (50M XLM)
    5u32,                           // Number of milestones
    "solar",                        // Energy type
    100_000_000i128                 // Expected capacity (100MW)
);
```

### 2. Create Milestones

```rust
// Create a project milestone
let milestone_id = contract.create_milestone(
    project_id,
    project_manager,
    "Site Preparation",                           // Milestone name
    "Prepare and survey construction site",      // Description
    20u32,                                       // 20% of total funding
    vec!["environmental_impact", "safety_compliance"], // Required verifications
    due_date,                                    // Deadline timestamp
    Some(20_000_000i128),                        // Expected energy output
    Some(5_000i128)                              // Expected carbon offset
);
```

### 3. Milestone Verification and Fund Release

```rust
// Start milestone work
contract.start_milestone(project_id, milestone_id, project_manager);

// Verify milestone requirements
contract.verify_milestone(
    project_id,
    milestone_id,
    verifier_address,
    "environmental_impact",
    "Environmental impact assessment completed"
);

// Release funds upon completion
contract.release_funds(project_id, milestone_id, approver_address);
```

### 4. Refund Processing

```rust
// Request refund if project fails
contract.request_refund(
    project_id,
    investor_address,
    refund_amount,
    "Project cancelled due to permit issues"
);

// Process approved refund
contract.process_refund(project_id, approver_address);
```

## 🔧 Smart Contract Interface

### Core Functions

| Function | Description | Authorization |
|----------|-------------|---------------|
| `initialize_project` | Create new energy project with escrow | Investor |
| `deposit` | Add funds to existing project | Investor |
| `release_funds` | Release milestone funding | Investor/Manager |
| `request_refund` | Request project refund | Investor |
| `process_refund` | Process approved refund | Manager/Investor |

### Milestone Management

| Function | Description | Authorization |
|----------|-------------|---------------|
| `create_milestone` | Create project milestone | Manager/Investor |
| `start_milestone` | Begin milestone work | Manager |
| `verify_milestone` | Verify completion requirements | Manager/Investor |
| `fail_milestone` | Mark milestone as failed | Manager |
| `update_metrics` | Update performance metrics | Manager |

### Query Functions

| Function | Description | Returns |
|----------|-------------|---------|
| `get_project` | Get complete project details | `ProjectDetails` |
| `get_milestone` | Get milestone information | `MilestoneDetails` |
| `get_available_funds` | Get remaining escrow balance | `i128` |
| `get_progress` | Get project completion percentage | `u32` |
| `calculate_refund` | Calculate potential refund amount | `i128` |

## 📊 Data Structures

### ProjectDetails
```rust
pub struct ProjectDetails {
    pub id: u64,
    pub name: String,
    pub investor: Address,
    pub project_manager: Address,
    pub total_funding: i128,
    pub released_funding: i128,
    pub milestone_count: u32,
    pub status: ProjectStatus,      // Active, Completed, Cancelled, Refunded
    pub energy_type: String,        // solar, wind, hydro, etc.
    pub expected_capacity: i128,
    pub compliance_verified: bool,
}
```

### MilestoneDetails
```rust
pub struct MilestoneDetails {
    pub project_id: u64,
    pub milestone_id: u32,
    pub name: String,
    pub funding_percentage: u32,
    pub status: MilestoneStatus,    // Pending, InProgress, Completed, Failed
    pub required_verifications: Vec<String>,
    pub due_date: u64,
    pub energy_output_target: Option<i128>,
    pub carbon_offset_target: Option<i128>,
}
```

## 🔒 Security Features

### Multi-signature Support
- Configurable signature requirements for high-value releases
- Authorized signer management
- Threshold-based approvals

### Validation & Compliance
- **Input Validation**: All parameters validated before processing
- **Role-based Access**: Strict authorization checks
- **Financial Limits**: Configurable funding limits and thresholds
- **Regulatory Compliance**: Built-in compliance verification requirements

### Audit Trail
- Complete event logging for all transactions
- Immutable milestone verification records
- Transparent fund release tracking

## 🌱 Supported Energy Types

- **Solar**: Photovoltaic and thermal installations
- **Wind**: Onshore and offshore wind farms
- **Hydro**: Small and large-scale hydroelectric projects
- **Geothermal**: Ground-source and enhanced geothermal systems
- **Biomass**: Organic waste and dedicated energy crops
- **Nuclear**: Advanced reactor technologies
- **Hybrid**: Combined renewable energy systems

## 🧪 Testing

### Run Test Suite
```bash
make test                    # Run all tests
```

### Test Coverage
- ✅ Project initialization and validation
- ✅ Milestone creation and management
- ✅ Fund deposit and release mechanisms
- ✅ Refund request and processing
- ✅ Multi-signature functionality
- ✅ Validation and error handling
- ✅ Edge cases and security scenarios

## 🛠 Development

### Code Quality
```bash
make format                 # Format code
make lint                   # Run clippy linter
make check                  # Complete quality check
make audit                  # Security audit
```

### Build Pipeline
```bash
make clean                  # Clean artifacts
make build                  # Build contract
make optimize              # Optimize WASM
make ci                    # Full CI/CD workflow
```

## 🌐 Deployment

### Testnet Deployment
```bash
# Deploy and configure on testnet
make deploy-testnet
CONTRACT_ID=<contract-id> make init-testnet
make verify
```

### Mainnet Deployment
```bash
# Production deployment (requires confirmation)
make build-prod
make deploy-mainnet
```

### Integration
```bash
# Generate TypeScript bindings
make bindings

# Generate documentation
make docs
```

## ⚡ Performance & Optimization

### Storage Optimization
- Efficient data structure packing
- Minimal storage footprint
- Optimized for Soroban fee structure

### Gas Optimization
- Streamlined function execution
- Batch operations where possible
- Minimal cross-contract calls

### Scalability
- Support for unlimited projects
- Efficient milestone querying
- Pagination for large datasets

## 🔗 Integration Examples

### Energy Credit Contracts
```rust
// Ready for integration with carbon credit contracts
let carbon_credits = calculate_carbon_credits(
    project.actual_carbon_offset,
    project.verification_status
);
```

### Token-based Funding
```rust
// Future support for tokenized project funding
let project_tokens = tokenize_project_funding(
    project_id,
    total_funding,
    milestone_structure
);
```

## 📋 Compliance & Regulations

### Supported Verification Types
- `environmental_impact` - Environmental impact assessments
- `safety_compliance` - Safety and operational compliance
- `technical_specification` - Technical performance validation
- `financial_audit` - Financial and accounting reviews
- `regulatory_approval` - Government and regulatory approvals
- `performance_metrics` - Operational performance verification
- `carbon_verification` - Carbon offset validation
- `grid_connection` - Grid interconnection compliance
- `commissioning_test` - System commissioning validation
- `insurance_validation` - Insurance coverage verification



### Code Standards
- Follow Rust formatting standards
- Include comprehensive tests
- Document public interfaces
- Security-first development

.

## 🆘 Support

### Documentation
- [Soroban Documentation](https://soroban.stellar.org/docs)
- [Stellar Developer Docs](https://developers.stellar.org/)
- [Rust Book](https://doc.rust-lang.org/book/)



**⚠️ Important Security Note**: This contract handles financial transactions. Always conduct thorough testing and security audits before mainnet deployment. Consider professional security review for production use.

**🌍 Environmental Impact**: This contract is designed to accelerate renewable energy adoption and carbon offset projects. Use responsibly to support sustainable energy initiatives.