#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, String};

fn setup_client(env: &Env) -> (AssetTokenizationContractClient<'_>, Address, Address) {
    let contract_id = env.register(AssetTokenizationContract, ());
    let client = AssetTokenizationContractClient::new(env, &contract_id);
    let admin = Address::generate(env);
    let issuer = Address::generate(env);
    client.initialize(&admin);
    env.mock_all_auths();
    (client, admin, issuer)
}

fn metadata(env: &Env, name: &str, symbol: &str, asset_type: &str) -> AssetMetadata {
    AssetMetadata {
        name: String::from_str(env, name),
        symbol: String::from_str(env, symbol),
        decimals: 6,
        asset_type: String::from_str(env, asset_type),
    }
}

fn regulatory(env: &Env, byte: u8, jurisdiction: &str) -> RegulatoryInfo {
    RegulatoryInfo {
        compliance_doc_hashes: BytesN::from_array(env, &[byte; 32]),
        jurisdiction: String::from_str(env, jurisdiction),
    }
}

fn token(env: &Env, client: &AssetTokenizationContractClient<'_>, issuer: &Address) -> u64 {
    client.tokenize(
        issuer,
        &metadata(env, "Asset", "AST", "Real Estate"),
        &regulatory(env, 7, "US"),
        &1_000,
    )
}

#[test]
fn scenario_a_real_estate_tokenization_and_sequential_ids() {
    let env = Env::default();
    let (client, admin, issuer) = setup_client(&env);
    let recipient = Address::generate(&env);
    let id1 = token(&env, &client, &issuer);
    let id2 = client.tokenize(
        &issuer,
        &metadata(&env, "Gold", "GLD", "Commodity"),
        &regulatory(&env, 9, "EU"),
        &2_000,
    );

    assert_eq!(id1, 1);
    assert_eq!(id2, 2);
    assert_eq!(client.balance_of(&issuer, &id1), 1_000);
    assert_eq!(client.balance_of(&issuer, &id2), 2_000);
    let info = client.token_info(&id2).unwrap();
    assert_eq!(info.issuer, issuer);
    assert_eq!(
        info.regulatory_info.compliance_doc_hashes,
        BytesN::from_array(&env, &[9; 32])
    );
    assert_eq!(info.metadata.symbol, String::from_str(&env, "GLD"));
    assert!(client.token_info(&99).is_none());

    client.update_compliance_status(&admin, &id1, &issuer, &ComplianceStatus::Approved);
    client.update_compliance_status(&admin, &id1, &recipient, &ComplianceStatus::Approved);
    client.transfer(&issuer, &recipient, &id1, &400);
    assert_eq!(client.balance_of(&issuer, &id1), 600);
    assert_eq!(client.balance_of(&recipient, &id1), 400);
}

#[test]
#[should_panic(expected = "Contract already initialized")]
fn double_initialize_panics() {
    let env = Env::default();
    let (client, admin, _) = setup_client(&env);
    client.initialize(&admin);
}

#[test]
fn compliance_pending_approved_rejected_and_transfer_gate() {
    let env = Env::default();
    let (client, admin, issuer) = setup_client(&env);
    let recipient = Address::generate(&env);
    let id = token(&env, &client, &issuer);

    assert!(matches!(
        client.verify_compliance(&id, &issuer),
        ComplianceStatus::Pending
    ));
    assert!(matches!(
        client.verify_compliance(&id, &recipient),
        ComplianceStatus::Pending
    ));
    client.update_compliance_status(&admin, &id, &issuer, &ComplianceStatus::Approved);
    client.update_compliance_status(&admin, &id, &recipient, &ComplianceStatus::Rejected);
    assert!(matches!(
        client.verify_compliance(&id, &issuer),
        ComplianceStatus::Approved
    ));
    assert!(matches!(
        client.verify_compliance(&id, &recipient),
        ComplianceStatus::Rejected
    ));
}

#[test]
#[should_panic(expected = "Compliance check failed")]
fn scenario_b_unverified_pending_user_is_blocked_in_transfer() {
    let env = Env::default();
    let (client, admin, issuer) = setup_client(&env);
    let recipient = Address::generate(&env);
    let id = token(&env, &client, &issuer);
    client.update_compliance_status(&admin, &id, &issuer, &ComplianceStatus::Approved);
    client.transfer(&issuer, &recipient, &id, &1);
}

#[test]
fn transfer_and_freeze_unfreeze() {
    let env = Env::default();
    let (client, admin, issuer) = setup_client(&env);
    let recipient = Address::generate(&env);
    let id = token(&env, &client, &issuer);
    for user in [&issuer, &recipient] {
        client.update_compliance_status(&admin, &id, user, &ComplianceStatus::Approved);
    }
    client.freeze_account(&admin, &issuer, &String::from_str(&env, "review"));
    assert!(client.is_frozen(&issuer));
    client.unfreeze_account(&admin, &issuer);
    assert!(!client.is_frozen(&issuer));
    client.transfer(&issuer, &recipient, &id, &400);
    assert_eq!(client.balance_of(&issuer, &id), 600);
    assert_eq!(client.balance_of(&recipient, &id), 400);
}

#[test]
#[should_panic(expected = "One or both accounts are frozen")]
fn frozen_account_blocks_transfer() {
    let env = Env::default();
    let (client, admin, issuer) = setup_client(&env);
    let recipient = Address::generate(&env);
    let id = token(&env, &client, &issuer);
    client.update_compliance_status(&admin, &id, &issuer, &ComplianceStatus::Approved);
    client.update_compliance_status(&admin, &id, &recipient, &ComplianceStatus::Approved);
    client.freeze_account(&admin, &issuer, &String::from_str(&env, "hold"));
    client.transfer(&issuer, &recipient, &id, &1);
}

#[test]
fn allowance_transfer_from_and_redemption() {
    let env = Env::default();
    let (client, admin, issuer) = setup_client(&env);
    let spender = Address::generate(&env);
    let receiver = Address::generate(&env);
    let id = token(&env, &client, &issuer);
    for user in [&issuer, &receiver] {
        client.update_compliance_status(&admin, &id, user, &ComplianceStatus::Approved);
    }
    client.approve(&issuer, &spender, &id, &300);
    assert_eq!(client.allowance(&issuer, &spender, &id), 300);
    client.transfer_from(&spender, &issuer, &receiver, &id, &200);
    assert_eq!(client.allowance(&issuer, &spender, &id), 100);
    assert_eq!(client.balance_of(&receiver, &id), 200);
    client.redeem(&issuer, &id, &100);
    assert_eq!(client.balance_of(&issuer, &id), 700);
    assert_eq!(client.token_info(&id).unwrap().total_supply, 900);
}

#[test]
#[should_panic(expected = "Insufficient allowance")]
fn transfer_from_rejects_excess_allowance() {
    let env = Env::default();
    let (client, _, issuer) = setup_client(&env);
    let spender = Address::generate(&env);
    let receiver = Address::generate(&env);
    let id = token(&env, &client, &issuer);
    client.approve(&issuer, &spender, &id, &1);
    client.transfer_from(&spender, &issuer, &receiver, &id, &2);
}

#[test]
#[should_panic(expected = "Insufficient balance")]
fn transfer_and_redeem_reject_insufficient_balance() {
    let env = Env::default();
    let (client, admin, issuer) = setup_client(&env);
    let recipient = Address::generate(&env);
    let id = token(&env, &client, &issuer);
    for user in [&issuer, &recipient] {
        client.update_compliance_status(&admin, &id, user, &ComplianceStatus::Approved);
    }
    client.transfer(&issuer, &recipient, &id, &1_001);
}

#[test]
#[should_panic(expected = "Token not found")]
fn nonexistent_token_is_rejected_after_compliance() {
    let env = Env::default();
    let (client, admin, issuer) = setup_client(&env);
    let recipient = Address::generate(&env);
    client.update_compliance_status(&admin, &99, &issuer, &ComplianceStatus::Approved);
    client.update_compliance_status(&admin, &99, &recipient, &ComplianceStatus::Approved);
    client.transfer(&issuer, &recipient, &99, &1);
}

#[test]
#[should_panic(expected = "Unauthorized: Only admins")]
fn non_admin_cannot_update_compliance() {
    let env = Env::default();
    let (client, admin, issuer) = setup_client(&env);
    let attacker = Address::generate(&env);
    let id = token(&env, &client, &issuer);
    let _ = admin;
    client.update_compliance_status(&attacker, &id, &issuer, &ComplianceStatus::Approved);
}

#[test]
#[should_panic(expected = "Issuer account is frozen")]
fn tokenize_rejects_frozen_issuer() {
    let env = Env::default();
    let (client, admin, issuer) = setup_client(&env);
    client.freeze_account(&admin, &issuer, &String::from_str(&env, "hold"));
    client.tokenize(
        &issuer,
        &metadata(&env, "Property", "PROP", "Real Estate"),
        &regulatory(&env, 1, "US"),
        &1,
    );
}

#[test]
#[should_panic(expected = "Compliance check failed")]
fn scenario_c_rejected_jurisdiction_user_cannot_receive_transfer() {
    let env = Env::default();
    let (client, admin, issuer) = setup_client(&env);
    let recipient = Address::generate(&env);
    let id = token(&env, &client, &issuer);
    client.update_compliance_status(&admin, &id, &issuer, &ComplianceStatus::Approved);
    client.update_compliance_status(&admin, &id, &recipient, &ComplianceStatus::Rejected);
    client.transfer(&issuer, &recipient, &id, &1);
}

#[test]
#[should_panic(expected = "One or both accounts are frozen")]
fn transfer_rejects_frozen_recipient() {
    let env = Env::default();
    let (client, admin, issuer) = setup_client(&env);
    let recipient = Address::generate(&env);
    let id = token(&env, &client, &issuer);
    client.update_compliance_status(&admin, &id, &issuer, &ComplianceStatus::Approved);
    client.update_compliance_status(&admin, &id, &recipient, &ComplianceStatus::Approved);
    client.freeze_account(&admin, &recipient, &String::from_str(&env, "hold"));
    client.transfer(&issuer, &recipient, &id, &1);
}

#[test]
fn admin_updates_asset_details_and_metadata_is_persisted() {
    let env = Env::default();
    let (client, admin, issuer) = setup_client(&env);
    let id = token(&env, &client, &issuer);
    client.update_asset_details(
        &admin,
        &id,
        &metadata(&env, "Updated Estate", "UEST", "Real Estate"),
    );
    let info = client.token_info(&id).unwrap();
    assert_eq!(info.metadata.name, String::from_str(&env, "Updated Estate"));
    assert_eq!(info.metadata.symbol, String::from_str(&env, "UEST"));
}

#[test]
#[should_panic(expected = "Unauthorized: Only admins")]
fn non_admin_cannot_update_asset_details() {
    let env = Env::default();
    let (client, _, issuer) = setup_client(&env);
    let attacker = Address::generate(&env);
    let id = token(&env, &client, &issuer);
    client.update_asset_details(&attacker, &id, &metadata(&env, "Fake", "FAKE", "Other"));
}

#[test]
#[should_panic(expected = "Unauthorized: Only admins")]
fn non_admin_cannot_freeze_account() {
    let env = Env::default();
    let (client, _, _) = setup_client(&env);
    let attacker = Address::generate(&env);
    let account = Address::generate(&env);
    client.freeze_account(&attacker, &account, &String::from_str(&env, "fake"));
}

#[test]
#[should_panic(expected = "Unauthorized: Only admins")]
fn non_admin_cannot_unfreeze_account() {
    let env = Env::default();
    let (client, admin, _) = setup_client(&env);
    let account = Address::generate(&env);
    let attacker = Address::generate(&env);
    client.freeze_account(&admin, &account, &String::from_str(&env, "hold"));
    client.unfreeze_account(&attacker, &account);
}

#[test]
#[should_panic(expected = "Owner account is frozen")]
fn approve_rejects_frozen_owner() {
    let env = Env::default();
    let (client, admin, issuer) = setup_client(&env);
    let spender = Address::generate(&env);
    let id = token(&env, &client, &issuer);
    client.freeze_account(&admin, &issuer, &String::from_str(&env, "hold"));
    client.approve(&issuer, &spender, &id, &1);
}

#[test]
#[should_panic(expected = "One or more accounts are frozen")]
fn transfer_from_rejects_frozen_owner() {
    let env = Env::default();
    let (client, admin, issuer) = setup_client(&env);
    let spender = Address::generate(&env);
    let receiver = Address::generate(&env);
    let id = token(&env, &client, &issuer);
    client.approve(&issuer, &spender, &id, &1);
    client.freeze_account(&admin, &issuer, &String::from_str(&env, "hold"));
    client.transfer_from(&spender, &issuer, &receiver, &id, &1);
}

#[test]
#[should_panic(expected = "One or more accounts are frozen")]
fn transfer_from_rejects_frozen_receiver() {
    let env = Env::default();
    let (client, admin, issuer) = setup_client(&env);
    let spender = Address::generate(&env);
    let receiver = Address::generate(&env);
    let id = token(&env, &client, &issuer);
    client.approve(&issuer, &spender, &id, &1);
    client.freeze_account(&admin, &receiver, &String::from_str(&env, "hold"));
    client.transfer_from(&spender, &issuer, &receiver, &id, &1);
}

#[test]
#[should_panic(expected = "One or more accounts are frozen")]
fn transfer_from_rejects_frozen_spender() {
    let env = Env::default();
    let (client, admin, issuer) = setup_client(&env);
    let spender = Address::generate(&env);
    let receiver = Address::generate(&env);
    let id = token(&env, &client, &issuer);
    client.approve(&issuer, &spender, &id, &1);
    client.freeze_account(&admin, &spender, &String::from_str(&env, "hold"));
    client.transfer_from(&spender, &issuer, &receiver, &id, &1);
}

#[test]
#[should_panic(expected = "Compliance check failed")]
fn transfer_from_rejects_pending_owner() {
    let env = Env::default();
    let (client, admin, issuer) = setup_client(&env);
    let spender = Address::generate(&env);
    let receiver = Address::generate(&env);
    let id = token(&env, &client, &issuer);
    client.approve(&issuer, &spender, &id, &1);
    client.update_compliance_status(&admin, &id, &receiver, &ComplianceStatus::Approved);
    client.transfer_from(&spender, &issuer, &receiver, &id, &1);
}

#[test]
#[should_panic(expected = "Compliance check failed")]
fn transfer_from_rejects_rejected_owner() {
    let env = Env::default();
    let (client, admin, issuer) = setup_client(&env);
    let spender = Address::generate(&env);
    let receiver = Address::generate(&env);
    let id = token(&env, &client, &issuer);
    client.approve(&issuer, &spender, &id, &1);
    client.update_compliance_status(&admin, &id, &issuer, &ComplianceStatus::Rejected);
    client.update_compliance_status(&admin, &id, &receiver, &ComplianceStatus::Approved);
    client.transfer_from(&spender, &issuer, &receiver, &id, &1);
}

#[test]
#[should_panic(expected = "Compliance check failed")]
fn transfer_from_rejects_pending_receiver() {
    let env = Env::default();
    let (client, admin, issuer) = setup_client(&env);
    let spender = Address::generate(&env);
    let receiver = Address::generate(&env);
    let id = token(&env, &client, &issuer);
    client.approve(&issuer, &spender, &id, &1);
    client.update_compliance_status(&admin, &id, &issuer, &ComplianceStatus::Approved);
    client.transfer_from(&spender, &issuer, &receiver, &id, &1);
}

#[test]
#[should_panic(expected = "Compliance check failed")]
fn transfer_from_rejects_rejected_receiver() {
    let env = Env::default();
    let (client, admin, issuer) = setup_client(&env);
    let spender = Address::generate(&env);
    let receiver = Address::generate(&env);
    let id = token(&env, &client, &issuer);
    client.approve(&issuer, &spender, &id, &1);
    client.update_compliance_status(&admin, &id, &issuer, &ComplianceStatus::Approved);
    client.update_compliance_status(&admin, &id, &receiver, &ComplianceStatus::Rejected);
    client.transfer_from(&spender, &issuer, &receiver, &id, &1);
}

#[test]
#[should_panic(expected = "Account is frozen")]
fn redeem_rejects_frozen_account() {
    let env = Env::default();
    let (client, admin, issuer) = setup_client(&env);
    let id = token(&env, &client, &issuer);
    client.freeze_account(&admin, &issuer, &String::from_str(&env, "hold"));
    client.redeem(&issuer, &id, &1);
}

#[test]
#[should_panic(expected = "Insufficient balance")]
fn redeem_rejects_amount_above_balance() {
    let env = Env::default();
    let (client, _, issuer) = setup_client(&env);
    let id = token(&env, &client, &issuer);
    client.redeem(&issuer, &id, &1_001);
}

#[test]
#[should_panic(expected = "Token not found")]
fn redeem_rejects_nonexistent_token() {
    let env = Env::default();
    let (client, _, issuer) = setup_client(&env);
    client.redeem(&issuer, &99, &1);
}

#[test]
#[should_panic(expected = "Insufficient balance")]
fn transfer_rejects_amount_above_balance() {
    let env = Env::default();
    let (client, admin, issuer) = setup_client(&env);
    let receiver = Address::generate(&env);
    let id = token(&env, &client, &issuer);
    client.update_compliance_status(&admin, &id, &issuer, &ComplianceStatus::Approved);
    client.update_compliance_status(&admin, &id, &receiver, &ComplianceStatus::Approved);
    client.transfer(&issuer, &receiver, &id, &1_001);
}

#[test]
fn approved_transfer_from_and_redemption_leave_consistent_supply() {
    let env = Env::default();
    let (client, admin, issuer) = setup_client(&env);
    let spender = Address::generate(&env);
    let receiver = Address::generate(&env);
    let id = token(&env, &client, &issuer);
    client.update_compliance_status(&admin, &id, &issuer, &ComplianceStatus::Approved);
    client.update_compliance_status(&admin, &id, &receiver, &ComplianceStatus::Approved);
    client.approve(&issuer, &spender, &id, &200);
    client.transfer_from(&spender, &issuer, &receiver, &id, &200);
    client.redeem(&issuer, &id, &100);
    assert_eq!(client.balance_of(&issuer, &id), 700);
    assert_eq!(client.balance_of(&receiver, &id), 200);
    assert_eq!(client.token_info(&id).unwrap().total_supply, 900);
}
