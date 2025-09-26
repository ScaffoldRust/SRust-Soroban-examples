#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String, BytesN};

#[test]
fn test_initialize_and_tokenize() {
    let env = Env::default();
    let contract_id = env.register_contract(None, AssetTokenizationContract);
    let client = AssetTokenizationContractClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    
    // Initialize contract
    client.initialize(&admin);
    
    // Create asset metadata
    let metadata = AssetMetadata {
        name: String::from_str(&env, "Test Real Estate"),
        symbol: String::from_str(&env, "TRE"),
        decimals: 6,
        asset_type: String::from_str(&env, "Real Estate"),
    };
    
    let regulatory_info = RegulatoryInfo {
        compliance_doc_hashes: BytesN::from_array(&env, &[0u8; 32]),
        jurisdiction: String::from_str(&env, "US"),
    };
    
    env.mock_all_auths();
    
    // Tokenize asset
    let token_id = client.tokenize(&issuer, &metadata, &regulatory_info, &1000000);
    
    // Check balance
    let balance = client.balance_of(&issuer, &token_id);
    assert_eq!(balance, 1000000);
    
    // Check token info
    let token_info = client.token_info(&token_id).unwrap();
    assert_eq!(token_info.metadata.name, metadata.name);
    assert_eq!(token_info.total_supply, 1000000);
}

#[test]
fn test_transfer() {
    let env = Env::default();
    let contract_id = env.register_contract(None, AssetTokenizationContract);
    let client = AssetTokenizationContractClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    let recipient = Address::generate(&env);
    
    // Setup
    client.initialize(&admin);
    
    let metadata = AssetMetadata {
        name: String::from_str(&env, "Test Token"),
        symbol: String::from_str(&env, "TT"),
        decimals: 6,
        asset_type: String::from_str(&env, "Commodity"),
    };
    
    let regulatory_info = RegulatoryInfo {
        compliance_doc_hashes: BytesN::from_array(&env, &[0u8; 32]),
        jurisdiction: String::from_str(&env, "US"),
    };
    
    env.mock_all_auths();
    
    let token_id = client.tokenize(&issuer, &metadata, &regulatory_info, &1000000);
    
    // Set compliance status for both parties
    client.update_compliance_status(&admin, &token_id, &issuer, &ComplianceStatus::Approved);
    client.update_compliance_status(&admin, &token_id, &recipient, &ComplianceStatus::Approved);
    
    // Transfer tokens
    client.transfer(&issuer, &recipient, &token_id, &500000);
    
    // Check balances
    assert_eq!(client.balance_of(&issuer, &token_id), 500000);
    assert_eq!(client.balance_of(&recipient, &token_id), 500000);
}

#[test]
fn test_approve_and_transfer_from() {
    let env = Env::default();
    let contract_id = env.register_contract(None, AssetTokenizationContract);
    let client = AssetTokenizationContractClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    let spender = Address::generate(&env);
    let recipient = Address::generate(&env);
    
    // Setup
    client.initialize(&admin);
    
    let metadata = AssetMetadata {
        name: String::from_str(&env, "Test Token"),
        symbol: String::from_str(&env, "TT"),
        decimals: 6,
        asset_type: String::from_str(&env, "Equity"),
    };
    
    let regulatory_info = RegulatoryInfo {
        compliance_doc_hashes: BytesN::from_array(&env, &[0u8; 32]),
        jurisdiction: String::from_str(&env, "EU"),
    };
    
    env.mock_all_auths();
    
    let token_id = client.tokenize(&issuer, &metadata, &regulatory_info, &1000000);
    
    // Set compliance status
    client.update_compliance_status(&admin, &token_id, &issuer, &ComplianceStatus::Approved);
    client.update_compliance_status(&admin, &token_id, &recipient, &ComplianceStatus::Approved);
    
    // Approve spender
    client.approve(&issuer, &spender, &token_id, &200000);
    
    // Check allowance
    assert_eq!(client.allowance(&issuer, &spender, &token_id), 200000);
    
    // Transfer from
    client.transfer_from(&spender, &issuer, &recipient, &token_id, &100000);
    
    // Check balances and remaining allowance
    assert_eq!(client.balance_of(&issuer, &token_id), 900000);
    assert_eq!(client.balance_of(&recipient, &token_id), 100000);
    assert_eq!(client.allowance(&issuer, &spender, &token_id), 100000);
}

#[test]
fn test_freeze_account() {
    let env = Env::default();
    let contract_id = env.register_contract(None, AssetTokenizationContract);
    let client = AssetTokenizationContractClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    
    client.initialize(&admin);
    env.mock_all_auths();
    
    // Freeze account
    client.freeze_account(&admin, &user, &String::from_str(&env, "Suspicious activity"));
    
    // Check if frozen
    assert_eq!(client.is_frozen(&user), true);
    
    // Unfreeze account
    client.unfreeze_account(&admin, &user);
    
    // Check if unfrozen
    assert_eq!(client.is_frozen(&user), false);
}

#[test]
fn test_redeem() {
    let env = Env::default();
    let contract_id = env.register_contract(None, AssetTokenizationContract);
    let client = AssetTokenizationContractClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    
    client.initialize(&admin);
    
    let metadata = AssetMetadata {
        name: String::from_str(&env, "Test Token"),
        symbol: String::from_str(&env, "TT"),
        decimals: 6,
        asset_type: String::from_str(&env, "Commodity"),
    };
    
    let regulatory_info = RegulatoryInfo {
        compliance_doc_hashes: BytesN::from_array(&env, &[0u8; 32]),
        jurisdiction: String::from_str(&env, "US"),
    };
    
    env.mock_all_auths();
    
    let token_id = client.tokenize(&issuer, &metadata, &regulatory_info, &1000000);
    
    // Redeem tokens
    client.redeem(&issuer, &token_id, &200000);
    
    // Check balance and total supply
    assert_eq!(client.balance_of(&issuer, &token_id), 800000);
    let token_info = client.token_info(&token_id).unwrap();
    assert_eq!(token_info.total_supply, 800000);
}

#[test]
fn test_compliance_status_check() {
    let env = Env::default();
    let contract_id = env.register_contract(None, AssetTokenizationContract);
    let client = AssetTokenizationContractClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    let user = Address::generate(&env);
    
    client.initialize(&admin);
    
    let metadata = AssetMetadata {
        name: String::from_str(&env, "Gold Commodity"),
        symbol: String::from_str(&env, "GOLD"),
        decimals: 6,
        asset_type: String::from_str(&env, "Commodity"),
    };
    
    let regulatory_info = RegulatoryInfo {
        compliance_doc_hashes: BytesN::from_array(&env, &[0u8; 32]),
        jurisdiction: String::from_str(&env, "EU"),
    };
    
    env.mock_all_auths();
    
    let token_id = client.tokenize(&issuer, &metadata, &regulatory_info, &5000000);
    
    // Check initial compliance status (should be Pending by default)
    let user_compliance = client.verify_compliance(&token_id, &user);
    assert!(matches!(user_compliance, ComplianceStatus::Pending));
    
    // Set compliance status
    client.update_compliance_status(&admin, &token_id, &issuer, &ComplianceStatus::Approved);
    client.update_compliance_status(&admin, &token_id, &user, &ComplianceStatus::Approved);
    
    // Check updated compliance status
    let issuer_compliance = client.verify_compliance(&token_id, &issuer);
    let user_compliance = client.verify_compliance(&token_id, &user);
    
    assert!(matches!(issuer_compliance, ComplianceStatus::Approved));
    assert!(matches!(user_compliance, ComplianceStatus::Approved));
    
    // Now transfer should work
    client.transfer(&issuer, &user, &token_id, &1000000);
    
    assert_eq!(client.balance_of(&user, &token_id), 1000000);
}

#[test]
fn test_multiple_tokens() {
    let env = Env::default();
    let contract_id = env.register_contract(None, AssetTokenizationContract);
    let client = AssetTokenizationContractClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    
    client.initialize(&admin);
    env.mock_all_auths();
    
    // Create first token
    let metadata1 = AssetMetadata {
        name: String::from_str(&env, "Property A"),
        symbol: String::from_str(&env, "PROPA"),
        decimals: 6,
        asset_type: String::from_str(&env, "Real Estate"),
    };
    
    let regulatory_info1 = RegulatoryInfo {
        compliance_doc_hashes: BytesN::from_array(&env, &[1u8; 32]),
        jurisdiction: String::from_str(&env, "US"),
    };
    
    // Create second token
    let metadata2 = AssetMetadata {
        name: String::from_str(&env, "Gold Bars"),
        symbol: String::from_str(&env, "GOLD"),
        decimals: 6,
        asset_type: String::from_str(&env, "Commodity"),
    };
    
    let regulatory_info2 = RegulatoryInfo {
        compliance_doc_hashes: BytesN::from_array(&env, &[2u8; 32]),
        jurisdiction: String::from_str(&env, "EU"),
    };
    
    let token_id1 = client.tokenize(&issuer, &metadata1, &regulatory_info1, &1000000);
    let token_id2 = client.tokenize(&issuer, &metadata2, &regulatory_info2, &2000000);
    
    // Check both tokens exist and have correct balances
    assert_eq!(client.balance_of(&issuer, &token_id1), 1000000);
    assert_eq!(client.balance_of(&issuer, &token_id2), 2000000);
    
    // Check token info
    let token_info1 = client.token_info(&token_id1).unwrap();
    let token_info2 = client.token_info(&token_id2).unwrap();
    
    assert_eq!(token_info1.metadata.name, metadata1.name);
    assert_eq!(token_info2.metadata.name, metadata2.name);
    assert_ne!(token_id1, token_id2);
}