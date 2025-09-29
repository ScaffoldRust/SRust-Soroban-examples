# Medical Donation Escrow Contract

A comprehensive smart contract for managing medical donations via escrow on the Stellar network using Soroban. This contract ensures funds are securely held and released upon meeting donation milestones while complying with financial regulations.

## 🏗 Contract Structure

```
medical-donation-escrow/src/
├── lib.rs                # Main contract configuration and exports
├── escrow.rs             # Core escrow logic for donation management
├── release.rs            # Milestone verification and fund release
├── utils.rs              # Shared utilities, validation, and refunds
└── test.rs               # Comprehensive test suite
```

## 🎯 Features

### Core Escrow Functionality
- **Secure Fund Management**: Funds are held in escrow until conditions are met
- **Milestone-Based Releases**: Funds released only after verification of specific milestones
- **Multi-Role Support**: Donors, recipients, and administrators with appropriate permissions
- **Refund Mechanism**: Automatic and manual refund processing for failed conditions

### Medical-Specific Features
- **Role-Based Releases**: Recipients verify medical equipment delivery milestones
- **Compliance Ready**: Built with financial regulation compliance in mind
- **Integration Hooks**: Designed for linking to organ donation or vaccine distribution contracts
- **Multi-Signature Support**: High-value donations can require multiple verifications

### Advanced Features
- **Emergency Controls**: Admin can pause/resume donations
- **Comprehensive Tracking**: Full audit trail of all donation activities
- **Scalable Design**: Handles multiple simultaneous donations efficiently
- **Gas Optimized**: Minimized transaction fees for donors and recipients

## 📦 Key Data Structures

### Donation
```rust
pub struct Donation {
    pub id: u64,
    pub donor: Address,
    pub recipient: Address,
    pub amount: i128,
    pub token: Symbol,
    pub description: String,
    pub milestones: Vec<String>,
    pub status: DonationStatus,
    pub created_at: u64,
    pub funded_at: Option<u64>,
    pub completed_at: Option<u64>,
    pub refunded_at: Option<u64>,
    pub admin: Address,
}
```

### Donation Status
- `Pending`: Donation created but not funded
- `Funded`: Funds deposited, waiting for milestone verification
- `InProgress`: Some milestones verified, more pending
- `Completed`: All milestones verified, funds released
- `Refunded`: Donation refunded to donor
- `Paused`: Temporarily paused by admin
- `Cancelled`: Donation cancelled

## 🔑 Key Functions

### Core Functions
- `initialize()` - Set up a new donation escrow with donor, recipient, and milestones
- `deposit()` - Deposit funds into the escrow for a specific donation
- `verify_milestone()` - Verify and progress a donation milestone
- `release_funds()` - Release funds to recipient upon milestone completion
- `refund()` - Process refunds if milestones are not met

### Query Functions
- `get_donation()` - Retrieve complete donation details
- `get_donation_status()` - Get current status of a donation
- `get_user_donations()` - List all donations for a specific user
- `get_milestone_status()` - Check if a specific milestone is verified

### Admin Functions
- `pause_donation()` - Emergency pause for a donation (admin only)
- `resume_donation()` - Resume a paused donation (admin only)

## 🚀 Usage Examples

### 1. Initialize a Medical Equipment Donation

```rust
let donor = Address::generate(&env);
let recipient = Address::generate(&env);
let donation_id = 1u64;
let amount = 10000i128; // $10,000 worth
let token = Symbol::short("USDC");
let description = String::from_str(&env, "MRI Machine for Rural Hospital");

let mut milestones = Vec::new(&env);
milestones.push_back(String::from_str(&env, "Equipment ordered from supplier"));
milestones.push_back(String::from_str(&env, "Equipment shipped and in transit"));
milestones.push_back(String::from_str(&env, "Equipment delivered to hospital"));
milestones.push_back(String::from_str(&env, "Equipment installed and operational"));

client.initialize(
    &donation_id,
    &donor,
    &recipient,
    &amount,
    &token,
    &description,
    &milestones,
);
```

### 2. Deposit Funds

```rust
client.deposit(&donation_id, &donor, &amount, &token);
```

### 3. Verify Milestones

```rust
// Recipient verifies equipment order
client.verify_milestone(
    &donation_id,
    &recipient,
    &0u32, // First milestone
    &String::from_str(&env, "Order confirmation #MRI-2024-001"),
);

// Recipient verifies delivery
client.verify_milestone(
    &donation_id,
    &recipient,
    &2u32, // Third milestone
    &String::from_str(&env, "Delivery receipt #DEL-2024-001"),
);
```

### 4. Release Funds

```rust
// After all milestones are verified, release funds
client.release_funds(&donation_id, &recipient);
```

## 🛠 Building and Testing

### Prerequisites
- Rust 1.70+
- Stellar CLI tools
- Soroban CLI

### Build
```bash
make build
# or
cargo build --target wasm32-unknown-unknown --release
stellar contract build
```

### Test
```bash
make test
# or
cargo test
```

### Deploy
```bash
make deploy
```

### Code Quality
```bash
make check  # Runs format, lint, and test
make fmt    # Format code
make lint   # Run clippy
```

## 🔒 Security Features

### Access Control
- **Donor**: Can deposit funds and request refunds
- **Recipient**: Can verify milestones and release funds
- **Admin**: Can pause/resume donations and approve refunds

### Validation
- Amount must be positive and within reasonable limits
- Milestones must be provided and non-empty
- Only authorized users can perform specific actions
- Comprehensive input validation throughout

### Audit Trail
- All actions are logged with timestamps
- Complete history of milestone verifications
- Refund request tracking and approval process

## 🌐 Integration Examples

### Medical Equipment Donations
```rust
// Example: MRI Machine Donation
let milestones = vec![
    "Equipment ordered from GE Healthcare",
    "Equipment manufactured and tested",
    "Equipment shipped to destination",
    "Equipment delivered to hospital",
    "Equipment installed by certified technicians",
    "Equipment operational and staff trained"
];
```

### Vaccine Distribution
```rust
// Example: COVID-19 Vaccine Distribution
let milestones = vec![
    "Vaccines procured from manufacturer",
    "Cold chain logistics arranged",
    "Vaccines shipped to distribution center",
    "Vaccines distributed to local clinics",
    "Vaccination program initiated",
    "Target population coverage achieved"
];
```

### Organ Donation Support
```rust
// Example: Organ Transplant Support
let milestones = vec![
    "Recipient identified and matched",
    "Surgery scheduled with medical team",
    "Organ procurement completed",
    "Transplant surgery performed",
    "Recipient recovery progressing",
    "Post-transplant care established"
];
```

## 📊 Performance Optimizations

- **Storage Efficiency**: Optimized data structures to minimize storage costs
- **Gas Optimization**: Efficient contract calls to reduce transaction fees
- **Batch Operations**: Support for multiple milestone verifications
- **Scalable Design**: Handles hundreds of concurrent donations

## 🔮 Future Enhancements

- **Multi-Signature Verification**: For high-value donations
- **Token Integration**: Native support for various Stellar tokens
- **Oracle Integration**: External data verification for milestones
- **Governance**: Community-driven parameter updates
- **Analytics**: Donation tracking and reporting dashboard

## 📄 License

This contract is part of the SRust-Soroban-examples project and follows the same licensing terms.

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Run the test suite
6. Submit a pull request

## 📞 Support

For questions and support, please refer to the main project documentation or create an issue in the repository.
