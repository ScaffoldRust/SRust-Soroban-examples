# Telemedicine Payment Gateway

A secure, compliant smart contract for processing telemedicine payments on the Stellar network using Soroban. This contract facilitates instant transactions between patients and healthcare providers while ensuring compliance with financial and healthcare regulations.

## 🏗 Contract Architecture

```
telemedicine-payment-gateway/
├── src/
│   ├── lib.rs           # Main contract interface and exports
│   ├── payment.rs       # Core payment processing logic
│   ├── gateway.rs       # Telemedicine session integration
│   └── utils.rs         # Shared utilities and validation
├── Cargo.toml           # Contract dependencies
├── Makefile            # Build and deployment automation
└── README.md           # This documentation
```

## 🚀 Features

### Payment Gateway
- **Secure Payment Processing**: Compliant with financial regulations
- **Instant Payments**: Real-time transaction processing
- **Fee Management**: Configurable platform and provider fees
- **Refund Processing**: Automated refunds for incomplete sessions
- **Dispute Resolution**: Built-in mechanisms for payment disputes

### Telemedicine-Specific Features
- **Role-Based Transactions**: Patients initiate, providers receive
- **Session Management**: Complete session lifecycle tracking
- **Consent Integration**: Links to patient consent contracts
- **Provider Registry**: Comprehensive provider management
- **Specialty Support**: Medical specialty categorization

### Compliance & Security
- **HIPAA Guidelines**: Designed with healthcare compliance in mind
- **Financial Regulations**: Built for regulatory compliance
- **Audit Trail**: Complete transaction history
- **Access Control**: Role-based permissions
- **Data Privacy**: Secure handling of sensitive information

## 📦 Key Data Structures

### PaymentSession
```rust
pub struct PaymentSession {
    pub payment_id: BytesN<32>,
    pub session_id: BytesN<32>,
    pub patient: Address,
    pub provider: Address,
    pub amount: i128,
    pub currency: Address,
    pub status: PaymentStatus,
    pub estimated_duration: u64,
    pub actual_duration: Option<u64>,
    pub created_at: u64,
    pub completed_at: Option<u64>,
    pub consent_hash: BytesN<32>,
    pub session_notes: Option<String>,
    pub platform_fee: i128,
    pub provider_fee: i128,
    pub refund_reason: Option<String>,
}
```

### ProviderConfig
```rust
pub struct ProviderConfig {
    pub provider_address: Address,
    pub provider_name: String,
    pub specialty: String,
    pub hourly_rate: i128,
    pub currency: Address,
    pub active: bool,
    pub registered_at: u64,
    pub last_updated: u64,
}
```

### FeeSchedule
```rust
pub struct FeeSchedule {
    pub platform_fee_percentage: u32,
    pub min_session_duration: u64,
    pub max_session_duration: u64,
    pub emergency_fee_percentage: u32,
    pub dispute_resolution_fee: i128,
}
```

## 🔑 Key Functions

### Initialization
```rust
pub fn initialize(
    env: Env,
    admin: Address,
    platform_fee_percentage: u32,
    min_session_duration: u64,
    max_session_duration: u64,
)
```

### Provider Management
```rust
pub fn register_provider(
    env: Env,
    provider_address: Address,
    provider_name: String,
    specialty: String,
    hourly_rate: i128,
    currency: Address,
)

pub fn update_provider_config(
    env: Env,
    provider_address: Address,
    new_hourly_rate: Option<i128>,
    new_currency: Option<Address>,
    active: Option<bool>,
)
```

### Payment Processing
```rust
pub fn initiate_payment(
    env: Env,
    patient: Address,
    provider: Address,
    session_id: BytesN<32>,
    estimated_duration: u64,
    consent_hash: BytesN<32>,
) -> BytesN<32>

pub fn confirm_session(
    env: Env,
    provider: Address,
    payment_id: BytesN<32>,
    actual_duration: u64,
    session_notes: String,
)

pub fn refund_payment(
    env: Env,
    caller: Address,
    payment_id: BytesN<32>,
    reason: String,
)
```

### Query Functions
```rust
pub fn get_payment_details(env: Env, payment_id: BytesN<32>) -> PaymentSession
pub fn get_provider_config(env: Env, provider: Address) -> ProviderConfig
pub fn get_balance(env: Env, address: Address) -> (i128, i128)
pub fn get_contract_status(env: Env) -> bool
```

## 🛠 Installation & Setup

### Prerequisites
- Rust (latest stable version)
- Soroban CLI
- Stellar CLI (optional)

### Build the Contract
```bash
# Clone the repository
git clone <repository-url>
cd telemedicine-payment-gateway

# Build the contract
make build

# Run tests
make test

# Optimize for deployment
make optimize
```

### Development Workflow
```bash
# Complete development workflow
make dev

# Production build
make prod

# Run security audit
make audit
```

## 🧪 Testing

### Unit Tests
```bash
# Run all tests
cargo test

# Run tests with verbose output
make test-verbose

# Run specific test module
cargo test payment::tests
```

### Integration Tests
```bash
# Deploy to testnet and run integration tests
make test-integration
```

## 🚀 Deployment

### Testnet Deployment
```bash
# Deploy to Stellar testnet
make deploy-testnet
```

### Futurenet Deployment
```bash
# Deploy to Stellar futurenet
make deploy-futurenet
```

### Contract Validation
```bash
# Validate contract before deployment
make validate
```

## 📊 Usage Examples

### 1. Initialize the Gateway
```rust
// Initialize with 2% platform fee, 15-480 minute sessions
contract.initialize(
    admin_address,
    200,  // 2% platform fee
    15,   // 15 minutes minimum
    480   // 8 hours maximum
);
```

### 2. Register a Healthcare Provider
```rust
contract.register_provider(
    provider_address,
    "Dr. Smith".to_string(),
    "Cardiology".to_string(),
    15000,  // $150.00 per hour
    usdc_token_address
);
```

### 3. Initiate a Payment
```rust
let payment_id = contract.initiate_payment(
    patient_address,
    provider_address,
    session_id,
    60,  // 60 minutes estimated
    consent_hash
);
```

### 4. Confirm Session Completion
```rust
contract.confirm_session(
    provider_address,
    payment_id,
    45,  // 45 minutes actual duration
    "Patient consultation completed successfully".to_string()
);
```

### 5. Process Refund
```rust
contract.refund_payment(
    patient_address,
    payment_id,
    "Session cancelled due to technical issues".to_string()
);
```

## 🔒 Security Features

### Access Control
- **Admin Functions**: Only admin can register providers and update fees
- **Provider Functions**: Providers can only confirm their own sessions
- **Patient Functions**: Patients can only initiate payments and request refunds

### Data Validation
- **Amount Validation**: Ensures payment amounts are within reasonable bounds
- **Duration Validation**: Validates session durations against configured limits
- **Address Validation**: Validates all addresses before processing
- **Consent Validation**: Ensures valid consent hashes are provided

### Audit Trail
- **Complete History**: All transactions are logged with timestamps
- **Status Tracking**: Payment status changes are tracked
- **Refund Tracking**: All refunds are logged with reasons

## 📈 Performance Optimizations

### Storage Efficiency
- **Minimal Storage**: Optimized data structures to reduce storage costs
- **Batch Operations**: Support for batch processing where possible
- **Lazy Loading**: Data is loaded only when needed

### Gas Optimization
- **Efficient Calculations**: Optimized fee calculations
- **Minimal External Calls**: Reduced external contract calls
- **Batch Updates**: Multiple updates in single transaction

## 🔧 Configuration

### Fee Structure
- **Platform Fee**: Configurable percentage (default: 2%)
- **Emergency Fee**: Higher fee for emergency sessions
- **Dispute Resolution**: Fixed fee for dispute processing

### Session Limits
- **Minimum Duration**: 15 minutes (configurable)
- **Maximum Duration**: 8 hours (configurable)
- **Rate Limits**: Configurable per provider

### Supported Currencies
- **Stellar Assets**: Any Stellar asset (XLM, USDC, etc.)
- **Token Contracts**: Integration with token contracts
- **Multi-Currency**: Support for multiple currencies

## 🚨 Error Handling

### Common Errors
- **Invalid Provider**: Provider not registered or inactive
- **Insufficient Funds**: Patient doesn't have enough balance
- **Invalid Duration**: Session duration outside allowed range
- **Unauthorized Access**: Caller doesn't have required permissions
- **Contract Paused**: Contract is temporarily paused

### Error Recovery
- **Graceful Degradation**: Contract continues operating despite errors
- **Clear Error Messages**: Descriptive error messages for debugging
- **Recovery Mechanisms**: Built-in recovery for common issues

## 🔮 Future Enhancements

### Planned Features
- **Stablecoin Integration**: Direct integration with stablecoin contracts
- **Multi-Signature Support**: Enhanced security for large payments
- **Insurance Integration**: Integration with healthcare insurance
- **Analytics Dashboard**: Real-time analytics and reporting
- **Mobile SDK**: Mobile application integration

### Scalability Improvements
- **Batch Processing**: Process multiple payments in single transaction
- **Layer 2 Integration**: Integration with layer 2 solutions
- **Cross-Chain Support**: Support for other blockchain networks

## 📚 API Reference

### Complete Function List
- `initialize()` - Initialize the gateway
- `register_provider()` - Register healthcare provider
- `update_provider_config()` - Update provider settings
- `initiate_payment()` - Start payment for session
- `confirm_session()` - Confirm session completion
- `refund_payment()` - Process payment refund
- `get_payment_details()` - Get payment information
- `get_provider_config()` - Get provider configuration
- `get_balance()` - Get address balance
- `pause_contract()` - Pause contract operations
- `resume_contract()` - Resume contract operations
- `update_platform_fee()` - Update platform fee

## 🤝 Contributing

### Development Setup
1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests and linting
5. Submit a pull request

### Code Standards
- Follow Rust best practices
- Write comprehensive tests
- Document all public functions
- Use meaningful variable names
- Handle errors gracefully

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🆘 Support

### Documentation
- [Soroban Documentation](https://soroban.stellar.org/docs)
- [Stellar Developers](https://developers.stellar.org/)
- [Rust Book](https://doc.rust-lang.org/book/)

### Community
- [Stellar Discord](https://discord.gg/stellar)
- [Soroban Forum](https://forum.stellar.org/)
- [GitHub Issues](https://github.com/your-repo/issues)

## 📞 Contact

For questions, suggestions, or support:
- **Email**: support@telemedicine-gateway.com
- **GitHub**: [@your-username](https://github.com/your-username)
- **Twitter**: [@telemedicine_gateway](https://twitter.com/telemedicine_gateway)

---

**⚠️ Disclaimer**: This contract is for educational and development purposes. Please ensure compliance with all applicable regulations before using in production.
