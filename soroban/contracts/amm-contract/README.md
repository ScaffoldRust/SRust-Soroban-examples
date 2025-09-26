# Production-Ready Advanced DEX/AMM Protocol

A complete, production-ready smart contract implementation of an Advanced Decentralized Exchange (DEX) and Automated Market Maker (AMM) Protocol built on Soroban. This contract enables users to provide liquidity, trade tokens, and earn fees through concentrated liquidity pools and optimized swap paths with enterprise-grade precision and security.

## Features

### Core AMM Functionality
- ✅ Create token pools with variable fee tiers (0.05%, 0.30%, 1.00%)
- ✅ Add/remove liquidity to/from pools with concentrated liquidity support
- ✅ Execute token swaps with slippage protection and deadline constraints
- ✅ Set and manage custom liquidity position bounds
- ✅ Optimize multi-hop swap paths for best execution
- ✅ Track and distribute accrued fees to liquidity providers

### Advanced Features
- 🎯 **Concentrated Liquidity**: Provide liquidity within specific price ranges for capital efficiency
- 🔄 **Multi-hop Swaps**: Automatic pathfinding through multiple pools for optimal rates
- 💰 **Fee Collection**: Automated fee accrual and collection for liquidity providers
- 📊 **Position Management**: Comprehensive position tracking with NFT-like functionality
- 🛡️ **Slippage Protection**: Built-in slippage checks and deadline enforcement
- ⚡ **Gas Optimization**: Tick-based pricing system for efficient price updates

## Contract Structure

```
amm-contract/
├── src/
│   ├── lib.rs              # Main contract interface and data structures
│   ├── pool.rs             # Pool creation and management logic
│   ├── liquidity.rs        # Liquidity provisioning and removal
│   ├── swap.rs             # Token swap and pathfinding logic
│   ├── fees.rs             # Fee collection and accounting
│   ├── position.rs         # Position management and bounds
│   └── test.rs             # Comprehensive test suite
├── Cargo.toml              # Rust dependencies and configuration
├── Makefile                # Build and deployment automation
└── README.md               # This documentation
```

## Key Data Structures

### Pool
```rust
struct Pool {
    token_a: Address,                    // First token in the pair
    token_b: Address,                    // Second token in the pair
    reserve_a: i128,                     // Reserve amount of token A
    reserve_b: i128,                     // Reserve amount of token B
    fee_tier: u32,                       // Fee tier in basis points
    tick_spacing: u32,                   // Spacing between ticks
    sqrt_price_x96: u128,               // Current price (sqrt format)
    liquidity: u128,                     // Total active liquidity
    fee_growth_global_0_x128: u128,     // Global fee growth for token 0
    fee_growth_global_1_x128: u128,     // Global fee growth for token 1
}
```

### Position
```rust
struct Position {
    owner: Address,                      // Position owner
    pool_id: BytesN<32>,                // Associated pool ID
    liquidity: u128,                     // Liquidity amount
    price_lower: u128,                   // Lower price bound
    price_upper: u128,                   // Upper price bound
    fee_growth_inside_0_last_x128: u128, // Last fee growth snapshot
    fee_growth_inside_1_last_x128: u128, // Last fee growth snapshot
    tokens_owed_0: u128,                 // Uncollected fees token 0
    tokens_owed_1: u128,                 // Uncollected fees token 1
}
```

## Core Functions

### Pool Management
```rust
// Create a new liquidity pool
create_pool(token_a, token_b, fee_tier, sqrt_price_x96) -> BytesN<32>

// Get pool information
get_pool(pool_id) -> Pool

// Get current pool price
get_pool_price(pool_id) -> u128
```

### Liquidity Operations
```rust
// Add liquidity to a pool within price bounds
add_liquidity(
    pool_id,
    amount_a_desired,
    amount_b_desired,
    amount_a_min,
    amount_b_min,
    price_lower,
    price_upper,
    recipient,
    deadline
) -> (position_id, liquidity, amount_0, amount_1)

// Remove liquidity from a position
remove_liquidity(
    position_id,
    liquidity,
    amount_0_min,
    amount_1_min,
    deadline
) -> (amount_0, amount_1)
```

### Trading
```rust
// Execute a token swap
swap(SwapParams {
    token_in,
    token_out,
    amount_in,
    min_amount_out,
    deadline,
    recipient
}) -> amount_out

// Get optimal swap path
get_optimal_swap_path(token_in, token_out, amount_in) -> SwapPath
```

### Fee Management
```rust
// Collect fees from a position
collect_fees(position_id) -> (fees_0, fees_1)

// Get position information with current fees
get_position(position_id) -> Position
```

## Fee Tiers

The protocol supports three standard fee tiers:

| Fee Tier | Percentage | Tick Spacing | Use Case         |
| -------- | ---------- | ------------ | ---------------- |
| 5        | 0.05%      | 1            | Stablecoin pairs |
| 30       | 0.30%      | 60           | Standard pairs   |
| 100      | 1.00%      | 200          | Exotic pairs     |

## Usage Examples

### Creating a Pool
```rust
let token_a = Address::from_string("CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQAOBKKG6ZHPJ");
let token_b = Address::from_string("CB64D3G7SM2RTH6JSGG34DDTFTQ5CFDKVDZJZSODMCX4NJ2HV2KN7OHT");
let fee_tier = 30u32; // 0.30%
let sqrt_price_x96 = 79228162514264337593543950336u128; // Price = 1.0

let pool_id = contract.create_pool(&token_a, &token_b, &fee_tier, &sqrt_price_x96);
```

### Adding Liquidity
```rust
let (position_id, liquidity, amount_0, amount_1) = contract.add_liquidity(
    &pool_id,
    &100000,  // amount_a_desired
    &100000,  // amount_b_desired
    &95000,   // amount_a_min (5% slippage)
    &95000,   // amount_b_min (5% slippage)
    &price_lower,
    &price_upper,
    &user_address,
    &deadline
);
```

### Executing a Swap
```rust
let swap_params = SwapParams {
    token_in: token_a,
    token_out: token_b,
    amount_in: 1000,
    min_amount_out: 990, // 1% slippage tolerance
    deadline: env.ledger().timestamp() + 3600,
    recipient: trader_address,
};

let amount_out = contract.swap(&swap_params);
```

## Building and Testing

### Prerequisites
```bash
# Install Rust and Soroban CLI
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
cargo install soroban-cli

# Add WASM target
rustup target add wasm32-unknown-unknown
```

### Build Commands
```bash
# Build the contract
cargo build

# Run tests
cargo test
```

### Testing
The contract includes comprehensive tests covering:
- Pool creation and management
- Liquidity addition and removal
- Token swaps and pathfinding
- Fee collection and distribution
- Error handling and edge cases

```bash
# Run specific test
cargo test test_create_pool -- --nocapture
```

## Security Considerations

### Implemented Protections
- ✅ **Slippage Protection**: Minimum output amounts enforced
- ✅ **Deadline Checks**: Transaction expiration prevention
- ✅ **Authorization**: Proper access control for position management
- ✅ **Overflow Protection**: Safe arithmetic operations
- ✅ **Price Validation**: Bounds checking for price ranges

### Best Practices
- Always set appropriate slippage tolerance
- Use reasonable deadline values (typically 10-30 minutes)
- Monitor position health and fee accumulation
- Be aware of impermanent loss in volatile markets

## Mathematical Foundations

The AMM uses concentrated liquidity mathematics similar to Uniswap V3:

### Price Calculation
- Prices are stored as `sqrt(price) * 2^96` for precision
- Tick-based system allows granular price movements
- Liquidity is distributed across price ranges

### Fee Calculation
- Fees accrue proportionally to liquidity provided
- Fee growth is tracked globally and per-position
- Compound fee growth using 128-bit fixed-point arithmetic

## References

- [Uniswap V3 Whitepaper](https://uniswap.org/whitepaper-v3.pdf)
- [Soroban Documentation](https://soroban.stellar.org/docs)
- [Rust Book](https://doc.rust-lang.org/book/)
- [Concentrated Liquidity Mathematics](https://atiselsts.github.io/pdfs/uniswap-v3-liquidity-math.pdf)