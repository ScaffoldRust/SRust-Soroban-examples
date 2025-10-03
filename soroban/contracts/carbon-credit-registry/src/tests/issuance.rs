#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _},
    Address, BytesN, Map, String, Vec,
};

use crate::{CreditStatus};

use super::utils::*;

// ============ INITIALIZATION TESTS ============

#[test]
fn test_contract_initialization() {
    let ctx = setup_test_env();

    // Verify contract stats after initialization
    assert_contract_stats(&ctx, 0, 0, 0, 0);
}

// ============ ISSUER REGISTRATION TESTS ============

#[test]
fn test_register_issuer() {
    let ctx = setup_test_env();
    let issuer = Address::generate(&ctx.env);

    let issuer_name = String::from_str(&ctx.env, "Verra Verified Issuer");
    let mut standards = Vec::new(&ctx.env);
    standards.push_back(String::from_str(&ctx.env, "Verra"));
    standards.push_back(String::from_str(&ctx.env, "Gold Standard"));

    ctx.client.register_issuer(&issuer, &issuer_name, &standards);

    // Verify issuer profile
    let profile = ctx.client.get_issuer_profile(&issuer);
    assert!(profile.is_some(), "Issuer profile should exist");

    let profile_data = profile.unwrap();
    assert_eq!(profile_data.address, issuer);
    assert_eq!(profile_data.name, issuer_name);
    assert_eq!(profile_data.is_active, true);
    assert_eq!(profile_data.total_issued, 0);
    assert_eq!(profile_data.total_retired, 0);

    // Verify issuer count increased
    let (issuer_count, _, _, _) = ctx.client.get_contract_stats();
    assert_eq!(issuer_count, 1);
}

#[test]
fn test_register_multiple_issuers() {
    let ctx = setup_test_env();

    let issuer1 = Address::generate(&ctx.env);
    let issuer2 = Address::generate(&ctx.env);
    let issuer3 = Address::generate(&ctx.env);

    let mut standards = Vec::new(&ctx.env);
    standards.push_back(String::from_str(&ctx.env, "Verra"));

    ctx.client.register_issuer(&issuer1, &String::from_str(&ctx.env, "Issuer 1"), &standards);
    ctx.client.register_issuer(&issuer2, &String::from_str(&ctx.env, "Issuer 2"), &standards);
    ctx.client.register_issuer(&issuer3, &String::from_str(&ctx.env, "Issuer 3"), &standards);

    // Verify all issuers registered
    let (issuer_count, _, _, _) = ctx.client.get_contract_stats();
    assert_eq!(issuer_count, 3);
}

// ============ CREDIT ISSUANCE TESTS ============

#[test]
fn test_issue_credit() {
    let (ctx, issuer) = setup_with_issuer();

    let credit_id = issue_test_credit(&ctx, &issuer, 1000);

    // Verify credit was issued
    let credit_opt = ctx.client.get_credit_status(&credit_id);
    assert!(credit_opt.is_some(), "Credit should exist");

    let credit = credit_opt.unwrap();
    assert_eq!(credit.issuer, issuer);
    assert_eq!(credit.quantity, 1000);
    assert_eq!(credit.status, CreditStatus::Issued);
    assert_eq!(credit.current_owner, issuer);
    assert_eq!(credit.vintage_year, 2024);

    // Verify contract stats updated
    let (_, credit_count, _, _) = ctx.client.get_contract_stats();
    assert_eq!(credit_count, 1);
}

#[test]
fn test_issue_multiple_credits() {
    let (ctx, issuer) = setup_with_issuer();

    // Issue multiple credits
    let credit1 = issue_test_credit(&ctx, &issuer, 500);
    let credit2 = issue_test_credit(&ctx, &issuer, 1000);
    let credit3 = issue_test_credit(&ctx, &issuer, 1500);

    // Verify all credits exist
    assert!(ctx.client.get_credit_status(&credit1).is_some());
    assert!(ctx.client.get_credit_status(&credit2).is_some());
    assert!(ctx.client.get_credit_status(&credit3).is_some());

    // Verify credit count
    let (_, credit_count, _, _) = ctx.client.get_contract_stats();
    assert_eq!(credit_count, 3);
}

#[test]
fn test_issue_credit_different_project_types() {
    let (ctx, issuer) = setup_with_issuer();

    let project_types = [
        ("Reforestation", "Brazil", "Verra"),
        ("Solar Energy", "India", "Gold Standard"),
        ("Wind Energy", "Denmark", "Verra"),
        ("Methane Capture", "USA", "Gold Standard"),
    ];

    for (project_type, location, standard) in project_types {
        let credit_id = issue_credit_with_params(&ctx, &issuer, project_type, location, standard, 2024, 1000);

        let credit = ctx.client.get_credit_status(&credit_id).unwrap();
        assert_eq!(credit.project_type, String::from_str(&ctx.env, project_type));
        assert_eq!(credit.project_location, String::from_str(&ctx.env, location));
        assert_eq!(credit.verification_standard, String::from_str(&ctx.env, standard));
    }

    let (_, credit_count, _, _) = ctx.client.get_contract_stats();
    assert_eq!(credit_count, 4);
}

#[test]
fn test_issue_credit_different_vintage_years() {
    let (ctx, issuer) = setup_with_issuer();

    let vintages = [2020, 2021, 2022, 2023, 2024];

    for vintage in vintages {
        let credit_id = issue_credit_with_params(
            &ctx,
            &issuer,
            "Reforestation",
            "Brazil",
            "Verra",
            vintage,
            1000,
        );

        let credit = ctx.client.get_credit_status(&credit_id).unwrap();
        assert_eq!(credit.vintage_year, vintage);
    }

    let (_, credit_count, _, _) = ctx.client.get_contract_stats();
    assert_eq!(credit_count, 5);
}

// ============ CREDIT HISTORY TESTS ============

#[test]
fn test_credit_history_after_issuance() {
    let (ctx, issuer) = setup_with_issuer();

    let credit_id = issue_test_credit(&ctx, &issuer, 1000);

    // Get credit history
    let history = ctx.client.get_credit_history(&credit_id);

    // Should have 1 event (issuance)
    assert_eq!(history.len(), 1);

    let event = history.get(0).unwrap();
    assert_eq!(event.quantity, 1000);
    assert_eq!(event.to.unwrap(), issuer);
}

// ============ EDGE CASE TESTS ============

#[test]
fn test_high_volume_credit_issuance() {
    let (ctx, issuer) = setup_with_issuer();

    // Issue 20 credits to test scalability
    let credit_ids = [
        "CREDIT_00", "CREDIT_01", "CREDIT_02", "CREDIT_03", "CREDIT_04",
        "CREDIT_05", "CREDIT_06", "CREDIT_07", "CREDIT_08", "CREDIT_09",
        "CREDIT_10", "CREDIT_11", "CREDIT_12", "CREDIT_13", "CREDIT_14",
        "CREDIT_15", "CREDIT_16", "CREDIT_17", "CREDIT_18", "CREDIT_19",
    ];

    for _ in credit_ids {
        issue_test_credit(&ctx, &issuer, 100);
    }

    // Verify all credits were issued
    let (_, credit_count, _, _) = ctx.client.get_contract_stats();
    assert_eq!(credit_count, 20);
}

#[test]
fn test_issue_credit_with_metadata() {
    let (ctx, issuer) = setup_with_issuer();

    let project_type = String::from_str(&ctx.env, "Reforestation");
    let project_location = String::from_str(&ctx.env, "Brazil");
    let verification_standard = String::from_str(&ctx.env, "Verra");
    let verification_hash = BytesN::from_array(&ctx.env, &[1u8; 32]);

    let mut metadata = Map::new(&ctx.env);
    metadata.set(String::from_str(&ctx.env, "project_name"), String::from_str(&ctx.env, "Amazon Reforestation"));
    metadata.set(String::from_str(&ctx.env, "sdg_goals"), String::from_str(&ctx.env, "13,15"));
    metadata.set(String::from_str(&ctx.env, "co-benefits"), String::from_str(&ctx.env, "biodiversity"));

    let credit_id = ctx.client.issue_credit(
        &issuer,
        &project_type,
        &project_location,
        &verification_standard,
        &2024,
        &1000,
        &verification_hash,
        &metadata,
    );

    // Verify credit was created
    let credit = ctx.client.get_credit_status(&credit_id).unwrap();
    assert_eq!(credit.quantity, 1000);
}
