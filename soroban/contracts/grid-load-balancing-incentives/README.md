# Grid Load Balancing Incentives Smart Contract

A Stellar Soroban smart contract that incentivizes load balancing on the Stellar network, rewarding consumers for adjusting energy usage during peak demand periods. The contract enables grid operators to initiate demand response events and automatically distribute rewards to participating consumers.

## Overview

This contract enables:
- Grid operators to start demand response events during peak periods
- Consumers to participate by reducing their energy usage
- Automatic reward calculation based on load reduction achieved
- Secure reward distribution using Stellar tokens
- Smart meter integration for verification of energy savings

## Features

### Core Functionality
- **Grid Operator Management**: Registration and authorization of grid operators
- **Consumer Registration**: Enrollment in demand response programs
- **Demand Response Events**: Timed events with target reduction goals
- **Load Reduction Verification**: Smart meter data validation
- **Automatic Rewards**: Token-based incentive distribution
- **Audit Trail**: Complete tracking of all participations and rewards

### Grid Management Features
- **Peak Demand Response**: Events triggered during high demand periods
- **Target-based Rewards**: Rewards calculated per kW reduced
- **Real-time Verification**: Meter reading validation for participation
- **Flexible Duration**: Configurable event timeframes
- **Scalable Participation**: Support for large numbers of consumers

## Contract Structure

```
grid-load-balancing-incentives/
├── src/
│   ├── lib.rs           # Main contract and exports
│   ├── demand.rs        # Demand response event management
│   ├── incentives.rs    # Reward distribution logic
│   ├── utils.rs         # Data structures and utilities
│   └── test.rs          # Contract tests
├── Cargo.toml           # Dependencies
├── Makefile             # Build and deployment automation
└── README.md            # Documentation
```

## Data Structures

### DemandResponseEvent
```rust
pub struct DemandResponseEvent {
    pub event_id: u64,
    pub grid_operator: Address,
    pub target_reduction_kw: u64,
    pub reward_per_kw: u64,
    pub start_time: u64,
    pub end_time: u64,
    pub status: EventStatus,
    pub total_participants: u64,
    pub total_reduction_achieved: u64,
}
```

### ParticipationRecord
```rust
pub struct ParticipationRecord {
    pub participation_id: u64,
    pub event_id: u64,
    pub consumer: Address,
    pub baseline_usage_kw: u64,
    pub actual_usage_kw: u64,
    pub reduction_achieved_kw: u64,
    pub reward_amount: u64,
    pub meter_reading_start: u64,
    pub meter_reading_end: u64,
    pub verified_at: u64,
    pub status: ParticipationStatus,
}
```

### Status Types
- **EventStatus**: Active, Completed, Cancelled
- **ParticipationStatus**: Pending, Verified, Rewarded, Rejected

## Key Functions

### Initialization
```rust
pub fn initialize(
    env: Env,
    admin: Address,
    token_contract: Address,
) -> Result<(), IncentiveError>
```
Sets up the contract with admin and reward token configuration.

### Registration
```rust
pub fn register_grid_operator(env: Env, grid_operator: Address) -> Result<(), IncentiveError>
```
Registers a grid operator authorized to start demand response events.

```rust
pub fn register_consumer(env: Env, consumer: Address) -> Result<(), IncentiveError>
```
Registers a consumer for participation in demand response programs.

### Demand Response Management
```rust
pub fn start_event(
    env: Env,
    grid_operator: Address,
    target_reduction_kw: u64,
    reward_per_kw: u64,
    duration_seconds: u64,
) -> Result<u64, IncentiveError>
```
Initiates a demand response event with target reductions and reward rates.

```rust
pub fn verify_reduction(
    env: Env,
    event_id: u64,
    consumer: Address,
    baseline_usage_kw: u64,
    actual_usage_kw: u64,
    meter_reading_start: u64,
    meter_reading_end: u64,
) -> Result<u64, IncentiveError>
```
Verifies consumer load reduction with smart meter data.

### Reward Distribution
```rust
pub fn distribute_rewards(
    env: Env,
    event_id: u64,
    distributor: Address,
) -> Result<(), IncentiveError>
```
Distributes rewards to verified participants using Stellar tokens.

### Query Functions
```rust
pub fn get_event(env: Env, event_id: u64) -> Result<DemandResponseEvent, IncentiveError>
```
Retrieves demand response event details.

```rust
pub fn get_consumer_participations(
    env: Env,
    consumer: Address,
) -> Result<Vec<ParticipationRecord>, IncentiveError>
```
Gets complete participation history for a consumer.

## Installation and Setup

### Prerequisites
- Rust 1.70+
- Stellar CLI
- WebAssembly target support

### Build the Contract
```bash
# Standard build
make build

# Build with Soroban CLI
make soroban-build

# For production deployment (requires wasm32 target)
# First install wasm target: rustup target add wasm32-unknown-unknown
make optimize
```

### Run Tests
```bash
make test
```

### Code Quality Checks
```bash
# Format code
make format

# Run linter
make clippy

# Run all checks
make check-all
```

## Deployment

### Deploy to Testnet
```bash
make deploy-testnet
```

### Initialize Contract
```bash
make init-testnet \
    ADMIN_ADDRESS=<admin_address> \
    TOKEN_CONTRACT_ADDRESS=<token_address> \
    CONTRACT_ID=<contract_id>
```

### Register Participants
```bash
# Register grid operator
make register-grid-operator-testnet \
    GRID_OPERATOR_ADDRESS=<operator_address> \
    CONTRACT_ID=<contract_id>

# Register consumer
make register-consumer-testnet \
    CONSUMER_ADDRESS=<consumer_address> \
    CONTRACT_ID=<contract_id>
```

## Usage Examples

### Complete Demand Response Flow
```bash
# 1. Grid operator starts demand response event
make start-event-testnet \
    GRID_OPERATOR_ADDRESS=<operator_address> \
    TARGET_REDUCTION_KW=1000 \
    REWARD_PER_KW=10 \
    DURATION_SECONDS=3600 \
    CONTRACT_ID=<contract_id>

# 2. Consumer participates by reducing load
make verify-reduction-testnet \
    EVENT_ID=1 \
    CONSUMER_ADDRESS=<consumer_address> \
    BASELINE_USAGE_KW=500 \
    ACTUAL_USAGE_KW=300 \
    METER_READING_START=<start_reading> \
    METER_READING_END=<end_reading> \
    CONTRACT_ID=<contract_id>

# 3. Distribute rewards to participants
make distribute-rewards-testnet \
    EVENT_ID=1 \
    DISTRIBUTOR_ADDRESS=<operator_address> \
    CONTRACT_ID=<contract_id>

# 4. View consumer participation history
make get-participations-testnet \
    CONSUMER_ADDRESS=<consumer_address> \
    CONTRACT_ID=<contract_id>
```

### Event Management
```bash
# Get event details
make get-event-testnet \
    EVENT_ID=1 \
    CONTRACT_ID=<contract_id>
```

## Integration Guide

### Smart Meter Integration
1. Register consumers using `register_consumer()`
2. Configure automated baseline usage tracking
3. Use meter readings in `verify_reduction()` for participation verification
4. Implement real-time usage monitoring for event participation

### Grid Operator Integration
1. Register grid operators using `register_grid_operator()`
2. Monitor grid demand to determine when to start events
3. Set appropriate target reductions and reward rates
4. Distribute rewards after event completion

### Example Integration Flow
```rust
// 1. Register participants
contract.register_grid_operator(grid_operator)?;
contract.register_consumer(consumer)?;

// 2. Start demand response event during peak demand
let event_id = contract.start_event(
    grid_operator,
    1000,    // target 1000 kW reduction
    10,      // 10 tokens per kW reduced
    3600,    // 1 hour duration
)?;

// 3. Consumer reduces load and verifies participation
let participation_id = contract.verify_reduction(
    event_id,
    consumer,
    500,     // baseline usage
    300,     // actual usage (200 kW reduction)
    meter_start,
    meter_end,
)?;

// 4. Distribute rewards to participants
contract.distribute_rewards(event_id, grid_operator)?;

// 5. Check participation history
let history = contract.get_consumer_participations(consumer)?;
```

## Security Features

### Access Control
- Role-based registration for grid operators and consumers
- Address-based authentication for all operations
- Authorization checks for event management and reward distribution

### Verification Integrity
- Smart meter reading validation
- Baseline usage verification
- Load reduction calculation accuracy
- Audit trail for all participations and rewards

### Reward Security
- Stellar token integration for secure transfers
- Admin-controlled reward distribution
- Automatic calculation prevents manipulation
- Complete transparency through events

## Error Handling

The contract includes comprehensive error handling:
- `NotInitialized`: Contract not yet initialized
- `GridOperatorNotRegistered`: Grid operator not registered
- `ConsumerNotRegistered`: Consumer not registered
- `EventNotFound`: Event doesn't exist
- `EventNotActive`: Event not currently active
- `EventExpired`: Event deadline passed
- `AlreadyParticipating`: Consumer already participating in event
- `InsufficientReduction`: Load reduction below minimum threshold
- `RewardDistributionFailed`: Token transfer failed
- `InvalidMeterReading`: Invalid meter reading data

## Performance Features

### Storage Optimization
- Efficient data structures for minimal storage cost
- Optimized maps for fast event and participation lookups
- Minimal storage updates per operation

### Scalability
- Support for large-scale demand response events
- Efficient participation tracking
- Optimized for high-frequency consumer interactions

## Compliance and Standards

### Grid Management Standards
- Compatible with FERC demand response guidelines
- Transparent reward calculation and distribution
- Verifiable load reduction records

### Blockchain Standards
- Standard Soroban contract patterns
- Stellar token integration
- Event emission for transparency

## Development

### Testing Strategy
- Unit tests for individual functions
- Integration tests for complete demand response workflows
- Edge case testing for event management and reward scenarios
- Smart meter verification testing

## Support

For issues, questions, or contributions, please refer to the project repository and follow the contribution guidelines.