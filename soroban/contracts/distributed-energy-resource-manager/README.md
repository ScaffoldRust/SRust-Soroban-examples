# Distributed Energy Resource Manager

A comprehensive smart contract for managing distributed energy resources (DERs) on the Stellar network using Soroban. This contract enables coordination and optimization of resources like solar panels, wind turbines, batteries, and other distributed energy assets.

## 🎯 Overview

The Distributed Energy Resource Manager provides a complete solution for:
- **Resource Registration**: Register and manage various types of DERs
- **Grid Integration**: Ensure compliance with grid integration standards
- **Optimization**: Advanced algorithms for resource coordination
- **Emergency Management**: Emergency resource allocation capabilities
- **Performance Tracking**: Comprehensive metrics and reporting

## 🏗 Contract Architecture

```
distributed-energy-resource-manager/
├── src/
│   ├── lib.rs                # Main contract logic and exports
│   ├── resource.rs           # Core DER management functionality
│   ├── optimization.rs       # Advanced optimization algorithms
│   └── utils.rs              # Utility functions and validation
├── Cargo.toml                # Dependencies and configuration
├── Makefile                  # Build and deployment automation
└── README.md                 # This documentation
```

## 🔧 Features

### Resource Management
- **Multi-Type Support**: Solar, Wind, Battery, Hydro, Geothermal, Fuel Cell
- **Capacity Validation**: Ensures DERs meet grid integration standards
- **Status Tracking**: Real-time status updates (Online, Offline, Maintenance, Emergency, Optimized)
- **Location Management**: Geographic tracking and regional optimization

### Optimization Engine
- **Grid Stability**: Advanced algorithms for frequency and voltage stability
- **Cost Optimization**: Economic dispatch based on energy prices
- **Renewable Maximization**: Weather-based renewable energy optimization
- **Emergency Response**: Critical grid situation handling

### Grid Integration
- **IEEE 1547 Compliance**: Meets distributed energy resource standards
- **Role-Based Access**: DER owners, grid operators, and administrators
- **Real-Time Coordination**: Live optimization and dispatch
- **Performance Metrics**: Comprehensive reporting and analytics

## 🚀 Quick Start

### Prerequisites

1. **Install Rust** (1.70+)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   rustup target add wasm32-unknown-unknown
   ```

2. **Install Soroban CLI**
   ```bash
   cargo install --locked soroban-cli
   ```

3. **Install Stellar CLI** (optional)
   ```bash
   cargo install stellar-cli
   ```

### Building the Contract

```bash
# Clone and navigate to the contract
cd soroban/contracts/distributed-energy-resource-manager

# Build the contract
make build

# Run tests
make test

# Build with optimization
make optimize
```

### Deployment

```bash
# Deploy to testnet
make deploy-testnet

# Initialize the contract
make init-testnet

# Register a DER
make register-der-testnet
```

## 📖 Usage Guide

### 1. Contract Initialization

```rust
// Initialize with admin address
contract.initialize(admin_address);
```

### 2. Register a Distributed Energy Resource

```rust
// Register a solar panel
contract.register_der(
    caller,
    "SOLAR_001".to_string(),
    ResourceType::Solar,
    1000, // 1 MW
    "San Francisco, CA".to_string()
);
```

### 3. Update DER Status

```rust
// Update status to maintenance
contract.update_status(
    caller,
    "SOLAR_001".to_string(),
    DERStatus::Maintenance
);
```

### 4. Optimize Resources

```rust
// Optimize for grid stability
let schedules = contract.optimize_resources(grid_operator);
```

### 5. Emergency Allocation

```rust
// Emergency power allocation
contract.emergency_allocation(
    grid_operator,
    "BATTERY_001".to_string(),
    500, // 500 kW
    3600 // 1 hour duration
);
```

## 🔑 Key Functions

### Core Functions

| Function | Description | Access Level |
|----------|-------------|--------------|
| `initialize()` | Initialize contract with admin | Admin only |
| `register_der()` | Register new DER | DER Owner |
| `update_status()` | Update DER status | Owner/Operator |
| `get_der_info()` | Query DER information | Public |
| `optimize_resources()` | Optimize DER coordination | Grid Operator |
| `emergency_allocation()` | Emergency resource allocation | Grid Operator |

### Query Functions

| Function | Description | Returns |
|----------|-------------|---------|
| `get_owner_ders()` | Get all DERs owned by address | `Vec<String>` |
| `get_optimization_schedules()` | Get current optimization schedules | `Vec<OptimizationSchedule>` |
| `get_stats()` | Get contract statistics | `ContractStats` |

## 📊 Data Structures

### DERInfo
```rust
pub struct DERInfo {
    pub owner: Address,
    pub resource_type: ResourceType,
    pub capacity: u32,        // in kW
    pub status: DERStatus,
    pub location: String,
    pub registration_time: u64,
    pub last_update: u64,
}
```

### OptimizationSchedule
```rust
pub struct OptimizationSchedule {
    pub der_id: String,
    pub timestamp: u64,
    pub power_output: i32,    // in kW, negative for consumption
    pub priority: u8,         // 1-10, higher is more critical
    pub grid_stability_score: u8, // 1-10, higher is better for grid
}
```

### Resource Types
- **Solar**: Photovoltaic panels
- **Wind**: Wind turbines
- **Battery**: Energy storage systems
- **Hydro**: Hydroelectric generators
- **Geothermal**: Geothermal power plants
- **FuelCell**: Fuel cell systems

## 🔒 Security Features

### Access Control
- **Role-based permissions**: Admin, Grid Operator, DER Owner
- **Authentication required**: All state-changing operations
- **Authority validation**: Grid operators can override certain operations

### Validation
- **Input validation**: All parameters validated before processing
- **Capacity limits**: Enforced per resource type
- **Status transitions**: Validated state machine
- **Emergency protocols**: Secure emergency allocation

### Compliance
- **IEEE 1547**: Distributed energy resource standards
- **Grid integration**: Meets utility interconnection requirements
- **Safety protocols**: Emergency shutdown and isolation

## 📈 Performance Optimization

### Storage Optimization
- **Efficient data structures**: Minimized storage footprint
- **Batch operations**: Reduced transaction costs
- **Lazy loading**: On-demand data retrieval

### Gas Optimization
- **Optimized algorithms**: Efficient computation
- **Minimal storage writes**: Reduced transaction fees
- **Batch processing**: Multiple operations per transaction

## 🧪 Testing

### Unit Tests
```bash
# Run all tests
cargo test

# Run with verbose output
cargo test -- --nocapture

# Run specific test
cargo test test_register_der
```

### Integration Tests
```bash
# Test with Soroban CLI
soroban contract test

# Test deployment
make deploy-testnet
```

### Test Coverage
- **Function coverage**: All public functions tested
- **Edge cases**: Boundary conditions and error cases
- **Integration**: End-to-end workflows
- **Performance**: Load and stress testing

## 🚨 Emergency Procedures

### Emergency Stop
```rust
// Emergency shutdown of all DERs
contract.emergency_allocation(
    emergency_operator,
    "ALL_DERS".to_string(),
    max_power,
    emergency_duration
);
```

### Maintenance Mode
```rust
// Enter maintenance mode
contract.update_status(
    admin,
    der_id,
    DERStatus::Maintenance
);
```

## 📋 Configuration

### Environment Variables
```bash
# Required for deployment
export ADMIN_KEY="your_admin_secret_key"
export ADMIN_ADDRESS="your_admin_address"
export CONTRACT_ID="deployed_contract_id"

# Optional for testing
export OWNER_KEY="der_owner_secret_key"
export OPERATOR_KEY="grid_operator_secret_key"
export DER_ID="unique_der_identifier"
export RESOURCE_TYPE="Solar"
export CAPACITY="1000"
export LOCATION="City, State"
```

### Network Configuration
- **Testnet**: `https://soroban-testnet.stellar.org:443`
- **Futurenet**: `https://rpc-futurenet.stellar.org:443`
- **Mainnet**: `https://horizon.stellar.org:443`

## 🔧 Development

### Code Style
- **Rust formatting**: `cargo fmt`
- **Clippy linting**: `cargo clippy`
- **Documentation**: Comprehensive inline docs

### Contributing
1. Fork the repository
2. Create feature branch
3. Write tests
4. Submit pull request

### Debugging
```bash
# Enable debug logging
RUST_LOG=debug cargo test

# Check contract state
soroban contract invoke --id CONTRACT_ID -- get_stats
```

## 📚 API Reference

### Events
- `der_registered`: DER successfully registered
- `der_status_updated`: DER status changed
- `resources_optimized`: Optimization completed
- `emergency_allocation`: Emergency allocation activated
- `maintenance_recorded`: Maintenance activity logged

### Error Codes
- `Invalid DER ID`: DER identifier validation failed
- `Unauthorized`: Insufficient permissions
- `DER not found`: DER doesn't exist
- `Invalid capacity`: Capacity outside valid range
- `Invalid status transition`: Invalid status change

## 🌐 Integration

### Smart Meter Integration
```rust
// Connect to smart meter contract
let meter_reading = smart_meter.get_reading(der_id);
contract.update_der_reading(der_id, meter_reading);
```

### Load Balancing
```rust
// Integrate with load balancing contract
let load_balance = load_balancer.get_balance();
contract.optimize_for_load_balance(load_balance);
```

### Weather Data
```rust
// Integrate with weather oracle
let weather = weather_oracle.get_conditions();
contract.optimize_for_weather(weather);
```

## 📊 Monitoring and Analytics

### Performance Metrics
- **Utilization Rate**: DER usage efficiency
- **Availability Score**: Uptime and reliability
- **Grid Stability Score**: Contribution to grid stability
- **Cost Efficiency**: Economic performance

### Reporting
- **Real-time Dashboards**: Live DER status
- **Historical Analysis**: Performance trends
- **Predictive Analytics**: Future optimization
- **Compliance Reports**: Regulatory reporting

## 🔮 Future Enhancements

### Planned Features
- **Machine Learning**: AI-powered optimization
- **Carbon Credits**: Environmental impact tracking
- **Market Integration**: Energy trading capabilities
- **IoT Integration**: Sensor data integration

### Scalability
- **Sharding**: Horizontal scaling
- **Layer 2**: Off-chain computation
- **Cross-chain**: Multi-blockchain support

## 📞 Support

### Documentation
- [Soroban Documentation](https://soroban.stellar.org/docs)
- [Stellar Documentation](https://developers.stellar.org/)
- [Rust Documentation](https://doc.rust-lang.org/)

### Community
- [Stellar Discord](https://discord.gg/stellar)
- [Soroban Forum](https://forum.stellar.org/)
- [GitHub Issues](https://github.com/your-repo/issues)

### Professional Support
- **Enterprise Support**: Available for commercial deployments
- **Consulting Services**: Custom implementation support
- **Training Programs**: Developer education and certification

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- **Stellar Development Foundation**: For the Soroban platform
- **Rust Community**: For the excellent language and ecosystem
- **IEEE Standards**: For DER integration guidelines
- **Open Source Contributors**: For their valuable contributions

---

**Built with ❤️ for the future of distributed energy**

*Last updated: January 2025*
