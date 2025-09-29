# Organ Donation Matching Protocol - Complete Setup Guide

## 📁 Complete File Structure

```
organ-donation-matching-protocol/
├── src/
│   ├── lib.rs          ✅ Main contract interface 
│   ├── matching.rs     ✅ Matching algorithms
│   ├── protocol.rs     ✅ Protocol rules 
│   └── utils.rs        ✅ Utility functions
|   └── test.rs         ✅ Complete test suite (AVAILABLE)
├── Cargo.toml          ✅ Project configuration (AVAILABLE)
├── Makefile            ✅ Build automation (AVAILABLE)
└── README.md           ✅ Documentation (AVAILABLE)
```

## 📝 File Contents Summary

### 1. lib.rs (Main Contract)
- **Key Features**:
  - Contract initialization
  - Donor/recipient registration
  - Match finding and confirmation
  - Profile management
  - Authorization controls

### 2. matching.rs (Matching Algorithms)
- **Key Features**:
  - Blood type compatibility (40% weight)
  - HLA tissue matching (35% weight)
  - Age compatibility (25% weight)
  - Priority scoring with urgency levels
  - UNOS-inspired matching algorithm
  - Viability time calculations
  - Success probability estimation

### 3. protocol.rs (Protocol Rules)
- **Lines**: ~350
- **Key Features**:
  - Registration validation
  - Organ-specific criteria
  - Consent verification
  - Emergency overrides
  - Compliance reporting
  - Audit logging
  - Wait time updates

### 4. utils.rs (Utility Functions)
- **Lines**: ~500
- **Key Features**:
  - Data type conversions
  - HLA allele validation
  - Age range validation
  - Risk score calculation
  - Fairness scoring

## 🔧 Key Functions Reference

### Initialization
```rust
initialize(admin, max_donors, max_recipients, urgency_weight, compatibility_threshold)
```

### Registration
```rust
register_donor(address, blood_type, organ_type, hla_profile, age, facility, consent_hash)
register_recipient(address, blood_type, organ_type, hla_profile, age, urgency, facility, priority)
```

### Matching
```rust
find_match(recipient_address) -> Vec<MatchResult>
confirm_match(match_id, medical_facility)
```

### Management
```rust
update_recipient_urgency(recipient, urgency_level, priority_score, caller)
deactivate_donor(donor_address, caller)
deactivate_recipient(recipient_address, caller)
```

## 📊 Compatibility Scoring Breakdown

### Blood Type Matrix (40% weight)
| Donor | Recipient A | Recipient B | Recipient AB | Recipient O |
|-------|-------------|-------------|--------------|-------------|
| **A** | 100         | 0           | 90           | 0           |
| **B** | 0           | 100         | 90           | 0           |
| **AB**| 0           | 0           | 100          | 0           |
| **O** | 95          | 95          | 95           | 100         |

### HLA Compatibility (35% weight)
- Compares alleles across HLA-A, HLA-B, and HLA-DR loci
- Perfect match (all alleles match): 100 points
- Partial match: Proportional scoring
- Minimum acceptable: 60 points

### Age Compatibility (25% weight)
- Heart/Lung: Max 10-year difference
- Kidney/Pancreas: Max 15-year difference
- Liver/Intestine: Max 20-year difference

## 🔒 Security Features

1. **Authorization Levels**
   - Admin: Full control
   - Medical facilities: Patient management
   - Patients: Own profile management

2. **Privacy Protection**
   - Minimal on-chain data
   - Address-based anonymization


## 📈 Performance Optimization

- **Storage**: Efficient use of Soroban storage types
- **Gas**: Optimized algorithms for minimal transaction costs
- **Scalability**: Supports thousands of registered users
- **Speed**: Fast matching algorithms (O(n log n) complexity)
