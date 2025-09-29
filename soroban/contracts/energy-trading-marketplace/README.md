# Energy Trading Marketplace Smart Contract

A Stellar Soroban smart contract that enables decentralized trading of energy units on the Stellar network, facilitating secure and transparent transactions between producers and consumers in the energy market.

## Overview

The Energy Trading Marketplace contract enables:
- Decentralized energy unit trading between producers and consumers
- Automated bid/ask order matching based on price compatibility
- Secure settlement with Stellar token transfers
- Role-based access control for market participants
- Transparent audit trail of all trades

## Features

### Core Functionality
- **Producer Registration**: Energy producers can register to sell energy units
- **Consumer Registration**: Energy consumers can register to buy energy units
- **Order Placement**: Support for buy and sell orders with quantity and price
- **Automated Matching**: Real-time matching of compatible buy/sell orders
- **Trade Settlement**: Secure payment processing using Stellar tokens
- **Trade History**: Complete transaction history for all participants

### Energy Market Features
- **Price-Based Matching**: Orders matched when buy price ≥ sell price
- **Role Verification**: Separate registration for producers and consumers
- **Grid Operator Support**: Framework for grid operator verification
- **Transparent Pricing**: All trades use seller's asking price
- **Order Management**: Full order lifecycle with cancellation support

## Contract Structure

```
energy-trading-marketplace/
├── src/
│   ├── lib.rs           # Main contract and exports
│   ├── trading.rs       # Order placement and matching logic
│   ├── settlement.rs    # Trade settlement and payment processing
│   ├── utils.rs         # Shared data structures and utilities
│   └── test.rs          # Contract tests
├── Cargo.toml           # Dependencies
├── Makefile             # Build automation
└── README.md            # Documentation
```

## Data Structures

### EnergyOrder
```rust
pub struct EnergyOrder {
    pub order_id: u64,
    pub trader: Address,
    pub order_type: OrderType,
    pub quantity_kwh: u64,
    pub price_per_kwh: u64,
    pub timestamp: u64,
    pub status: OrderStatus,
}
```

### Trade
```rust
pub struct Trade {
    pub trade_id: u64,
    pub buyer: Address,
    pub seller: Address,
    pub quantity_kwh: u64,
    pub price_per_kwh: u64,
    pub total_amount: u64,
    pub timestamp: u64,
}
```

### OrderType
- `Buy`: Purchase order for energy units
- `Sell`: Sale order for energy units

### OrderStatus
- `Active`: Order available for matching
- `Filled`: Order completely matched
- `Cancelled`: Order cancelled by trader

### TraderRole
- `Producer`: Energy producer (can sell)
- `Consumer`: Energy consumer (can buy)
- `GridOperator`: Grid infrastructure operator

## Key Functions

### Initialization
```rust
pub fn initialize(
    env: Env,
    admin: Address,
    token_contract: Address,
    min_trade_size: u64,
    max_trade_size: u64,
) -> Result<(), MarketplaceError>
```
Sets up the marketplace with admin, payment token, and trade size limits.

### Registration
```rust
pub fn register_producer(env: Env, producer: Address) -> Result<(), MarketplaceError>
```
Registers an energy producer.

```rust
pub fn register_consumer(env: Env, consumer: Address) -> Result<(), MarketplaceError>
```
Registers an energy consumer.

### Trading
```rust
pub fn place_order(
    env: Env,
    trader: Address,
    order_type: OrderType,
    quantity_kwh: u64,
    price_per_kwh: u64,
) -> Result<u64, MarketplaceError>
```
Places a buy or sell order. Returns the order ID. Automatically attempts to match with existing orders.

```rust
pub fn cancel_order(env: Env, trader: Address, order_id: u64) -> Result<(), MarketplaceError>
```
Cancels an active order.

### Settlement
```rust
pub fn settle_trade(env: Env, trade_id: u64, settler: Address) -> Result<(), MarketplaceError>
```
Settles a trade with payment transfer using Stellar tokens.

### Query Functions
```rust
pub fn get_order(env: Env, order_id: u64) -> Result<EnergyOrder, MarketplaceError>
```
Retrieves order details.

```rust
pub fn get_trade(env: Env, trade_id: u64) -> Result<Trade, MarketplaceError>
```
Retrieves trade details.

```rust
pub fn get_trade_history(env: Env, trader: Address) -> Result<Vec<Trade>, MarketplaceError>
```
Gets complete trading history for a specific trader.

## Installation and Setup

### Prerequisites
- Rust 1.70+
- Stellar CLI
- WebAssembly target support

### Build the Contract
```bash
# Standard build
make build

# Build with Stellar CLI
make soroban-build

# Optimized build for deployment
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
    MIN_TRADE_SIZE=10 \
    MAX_TRADE_SIZE=1000 \
    CONTRACT_ID=<contract_id>
```

### Register Market Participants
```bash
# Register producer
make register-producer-testnet \
    PRODUCER_ADDRESS=<producer_address> \
    CONTRACT_ID=<contract_id>

# Register consumer
make register-consumer-testnet \
    CONSUMER_ADDRESS=<consumer_address> \
    CONTRACT_ID=<contract_id>
```

## Usage Examples

### Complete Trading Flow
```bash
# 1. Producer places sell order
make place-order-testnet \
    TRADER_ADDRESS=<producer_address> \
    ORDER_TYPE=Sell \
    QUANTITY_KWH=100 \
    PRICE_PER_KWH=50 \
    CONTRACT_ID=<contract_id>

# 2. Consumer places matching buy order (auto-matches if compatible)
make place-order-testnet \
    TRADER_ADDRESS=<consumer_address> \
    ORDER_TYPE=Buy \
    QUANTITY_KWH=100 \
    PRICE_PER_KWH=55 \
    CONTRACT_ID=<contract_id>

# 3. Settle the resulting trade
make settle-trade-testnet \
    TRADE_ID=1 \
    SETTLER_ADDRESS=<consumer_address> \
    CONTRACT_ID=<contract_id>

# 4. View trade history
make get-trade-history-testnet \
    TRADER_ADDRESS=<consumer_address> \
    CONTRACT_ID=<contract_id>
```

### Order Management
```bash
# Get order details
make get-order-testnet \
    ORDER_ID=1 \
    CONTRACT_ID=<contract_id>

# Cancel order
make cancel-order-testnet \
    TRADER_ADDRESS=<trader_address> \
    ORDER_ID=1 \
    CONTRACT_ID=<contract_id>
```

## Integration Guide

### Smart Meter Integration
1. Register meters as producers using `register_producer()`
2. Configure automated order placement based on generation data
3. Implement real-time price optimization

### Energy Management Systems
1. Register energy management systems as consumers
2. Place orders based on demand forecasting
3. Monitor trade execution and settlement

### Payment Integration
The contract uses Stellar tokens for settlement:
- Configure token contract address during initialization
- Ensure traders have sufficient token balances
- Settlement transfers tokens from buyer to seller

### Example Integration Flow
```rust
// 1. Register participants
contract.register_producer(producer_address)?;
contract.register_consumer(consumer_address)?;

// 2. Producer places sell order
let sell_order_id = contract.place_order(
    producer_address,
    OrderType::Sell,
    100, // kWh
    50,  // price per kWh
)?;

// 3. Consumer places buy order (may auto-match)
let buy_order_id = contract.place_order(
    consumer_address,
    OrderType::Buy,
    100, // kWh
    55,  // price per kWh
)?;

// 4. Check trade history for matches
let trades = contract.get_trade_history(consumer_address)?;

// 5. Settle trade if matched
if !trades.is_empty() {
    contract.settle_trade(trades[0].trade_id, consumer_address)?;
}
```

## Security Features

### Access Control
- Role-based registration (producers vs consumers)
- Address-based authentication for all operations
- Authorization checks for order cancellation and trade settlement

### Trade Integrity
- Immutable trade records
- Transparent pricing (uses seller's price)
- Automatic order matching prevents manipulation
- Complete audit trail through events

### Payment Security
- Stellar token integration for secure transfers
- Authorization required for all payment operations
- Atomic settlement operations

## Error Handling

The contract includes comprehensive error handling:

- `AlreadyInitialized`: Contract already initialized
- `NotInitialized`: Contract not yet initialized
- `NotAuthorized`: Insufficient permissions
- `InvalidInput`: Invalid parameter values
- `OrderNotFound`: Requested order doesn't exist
- `TradeNotFound`: Requested trade doesn't exist
- `PriceOutOfRange`: Invalid price value
- `TraderNotRegistered`: Trader not registered
- `QuantityOutOfRange`: Invalid quantity value
- `PaymentFailed`: Token transfer failed

## Performance Characteristics

### Storage Efficiency
- Optimized data structures for minimal storage cost
- Efficient maps for fast order and trade lookups
- Minimal storage updates per transaction

### Transaction Costs
- Gas-efficient order matching algorithm
- Batch operations where possible
- Optimized for high-frequency trading

### Scalability
- O(n) order matching where n = active orders
- Efficient storage patterns for large order books
- Minimal state updates per operation

## Compliance and Standards

### Energy Market Standards
- Compatible with standard energy trading practices
- Price-based matching following market conventions
- Transparent settlement process

### Blockchain Standards
- Standard Soroban contract patterns
- Stellar token integration
- Event emission for transparency

## Development

### Project Structure
The contract follows standard Soroban patterns:
- Modular design with separated concerns
- Comprehensive test coverage
- Standard error handling
- Efficient storage patterns

### Testing Strategy
- Unit tests for individual functions
- Integration tests for complete trading workflows
- Edge case testing for order matching
- Settlement testing with mock payments

### Contributing
1. Fork the repository
2. Create feature branch
3. Add tests for new functionality
4. Ensure all tests pass
5. Submit pull request

## Documentation

Generate contract documentation:
```bash
make docs
```

## Support

For issues, questions, or contributions, please refer to the project repository and follow the contribution guidelines.

## License

This project is part of the SRust-Soroban-examples repository and follows the same licensing terms.