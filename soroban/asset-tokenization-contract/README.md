# Asset Tokenization Smart Contract

A comprehensive smart contract for tokenizing assets with regulatory compliance on the Stellar network using Soroban.

## Features

- **Asset Tokenization**: Convert physical or digital assets into blockchain tokens
- **Regulatory Compliance**: Built-in compliance verification and management
- **Transfer Controls**: Secure token transfers with compliance checks
- **Admin Functions**: Administrative controls for asset management and account freezing
- **Allowance System**: Approve third parties to spend tokens on your behalf

## Contract Structure

- `lib.rs` - Main contract interface and exports
- `token.rs` - Token balance and metadata management
- `issuance.rs` - Asset tokenization and redemption logic
- `transfer.rs` - Token transfer and approval mechanisms
- `compliance.rs` - Regulatory compliance verification
- `admin.rs` - Administrative functions and access control

## Key Functions

### Tokenization
- `tokenize()` - Create tokens representing real-world assets
- `redeem()` - Redeem tokens for underlying assets

### Transfers
- `transfer()` - Transfer tokens between addresses
- `approve()` - Approve spending allowances
- `transfer_from()` - Transfer tokens using allowances

### Compliance
- `verify_compliance()` - Check regulatory compliance status
- `update_compliance_status()` - Update user compliance status (admin only)

### Administration
- `freeze_account()` / `unfreeze_account()` - Account management
- `update_asset_details()` - Update token metadata

## Usage

1. Deploy the contract to Stellar network
2. Initialize with admin address
3. Tokenize assets with regulatory documentation
4. Set compliance status for users
5. Enable secure, compliant transfers

## Testing

Run tests with:
```bash
cargo test

Security Features

Authorization checks on all sensitive operations
Account freezing capabilities
Compliance verification before transfers
Admin-only functions for critical operations


## Step 13: Update Root README

Add to `soroban/README.md` (or create if it doesn't exist):
```markdown
# Asset Tokenization Contract

A new smart contract has been added: `asset-tokenization-contract/`

This contract provides comprehensive asset tokenization functionality with:
- Regulatory compliance management
- Secure token transfers
- Administrative controls
- Real-world asset representation

See `asset-tokenization-contract/README.md` for detailed documentation.