#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _},
    Address,
};

use crate::{CreditStatus};

use super::utils::*;

// ============ CREDIT RETIREMENT TESTS ============

#[test]
fn test_retire_credit() {
    let (ctx, issuer) = setup_with_issuer();

    // Issue a credit
    let credit_id = issue_test_credit(&ctx, &issuer, 1000);

    // Retire the credit
    retire_test_credit(&ctx, credit_id.clone(), &issuer, 1000);

    // Verify status changed to Retired
    assert_credit_status(&ctx, &credit_id, CreditStatus::Retired);

    // Verify quantity is now 0
    assert_credit_quantity(&ctx, &credit_id, 0);
}

#[test]
fn test_retire_partial_credit() {
    let (ctx, issuer) = setup_with_issuer();

    // Issue a credit with 1000 quantity
    let credit_id = issue_test_credit(&ctx, &issuer, 1000);

    // Retire partial quantity (500 out of 1000)
    retire_test_credit(&ctx, credit_id.clone(), &issuer, 500);

    // Verify quantity reduced
    assert_credit_quantity(&ctx, &credit_id, 500);

    // Status should still be Issued (not fully retired)
    assert_credit_status(&ctx, &credit_id, CreditStatus::Issued);
}

#[test]
fn test_retire_credit_in_stages() {
    let (ctx, issuer) = setup_with_issuer();

    // Issue a credit with 1000 quantity
    let credit_id = issue_test_credit(&ctx, &issuer, 1000);

    // Retire in stages: 300, 200, 500 (total 1000)
    retire_test_credit(&ctx, credit_id.clone(), &issuer, 300);
    assert_credit_quantity(&ctx, &credit_id, 700);

    retire_test_credit(&ctx, credit_id.clone(), &issuer, 200);
    assert_credit_quantity(&ctx, &credit_id, 500);

    retire_test_credit(&ctx, credit_id.clone(), &issuer, 500);
    assert_credit_quantity(&ctx, &credit_id, 0);

    // Now should be fully retired
    assert_credit_status(&ctx, &credit_id, CreditStatus::Retired);
}

#[test]
fn test_retire_traded_credit() {
    let (ctx, issuer) = setup_with_issuer();

    // Issue a credit
    let credit_id = issue_test_credit(&ctx, &issuer, 1000);

    // Trade to buyer
    let buyer = Address::generate(&ctx.env);
    trade_test_credit(&ctx, credit_id.clone(), &issuer, &buyer, 1000);

    // Buyer retires the credit
    retire_test_credit(&ctx, credit_id.clone(), &buyer, 1000);

    // Verify retired
    assert_credit_status(&ctx, &credit_id, CreditStatus::Retired);
    assert_credit_quantity(&ctx, &credit_id, 0);
}

#[test]
fn test_retirement_updates_stats() {
    let (ctx, issuer) = setup_with_issuer();

    // Issue credits
    let credit1 = issue_test_credit(&ctx, &issuer, 500);
    let credit2 = issue_test_credit(&ctx, &issuer, 1000);

    // Retire both credits
    retire_test_credit(&ctx, credit1, &issuer, 500);
    retire_test_credit(&ctx, credit2, &issuer, 1000);

    // Check contract stats
    let (_, _, _, total_retired) = ctx.client.get_contract_stats();
    assert_eq!(total_retired, 1500, "Total retired should be 1500");
}

#[test]
fn test_retirement_history() {
    let (ctx, issuer) = setup_with_issuer();

    // Issue a credit
    let credit_id = issue_test_credit(&ctx, &issuer, 1000);

    // Retire the credit
    retire_test_credit(&ctx, credit_id.clone(), &issuer, 1000);

    // Get history (should have issuance + retirement)
    let history = ctx.client.get_credit_history(&credit_id);
    assert!(history.len() >= 2, "Should have at least 2 events (issuance + retirement)");
}

// ============ EDGE CASE TESTS ============

#[test]
#[should_panic]
fn test_retire_non_existent_credit() {
    let ctx = setup_test_env();

    let owner = Address::generate(&ctx.env);
    let non_existent_credit_id = soroban_sdk::BytesN::from_array(&ctx.env, &[99u8; 32]);

    // Attempt to retire non-existent credit - should panic
    retire_test_credit(&ctx, non_existent_credit_id, &owner, 1000);
}

#[test]
fn test_retire_multiple_credits() {
    let (ctx, issuer) = setup_with_issuer();

    // Issue multiple credits
    let credit1 = issue_test_credit(&ctx, &issuer, 500);
    let credit2 = issue_test_credit(&ctx, &issuer, 1000);
    let credit3 = issue_test_credit(&ctx, &issuer, 1500);

    // Retire all credits
    retire_test_credit(&ctx, credit1.clone(), &issuer, 500);
    retire_test_credit(&ctx, credit2.clone(), &issuer, 1000);
    retire_test_credit(&ctx, credit3.clone(), &issuer, 1500);

    // Verify all retired
    assert_credit_status(&ctx, &credit1, CreditStatus::Retired);
    assert_credit_status(&ctx, &credit2, CreditStatus::Retired);
    assert_credit_status(&ctx, &credit3, CreditStatus::Retired);

    // Check total retired
    let (_, _, _, total_retired) = ctx.client.get_contract_stats();
    assert_eq!(total_retired, 3000);
}

#[test]
fn test_trade_then_retire() {
    let (ctx, issuer) = setup_with_issuer();

    // Issue a credit
    let credit_id = issue_test_credit(&ctx, &issuer, 1000);

    // Trade chain: issuer -> buyer1 -> buyer2
    let buyer1 = Address::generate(&ctx.env);
    let buyer2 = Address::generate(&ctx.env);

    trade_test_credit(&ctx, credit_id.clone(), &issuer, &buyer1, 1000);
    trade_test_credit(&ctx, credit_id.clone(), &buyer1, &buyer2, 1000);

    // Final buyer retires the credit
    retire_test_credit(&ctx, credit_id.clone(), &buyer2, 1000);

    // Verify retired
    assert_credit_status(&ctx, &credit_id, CreditStatus::Retired);

    // Get full history (issuance + 2 trades + retirement = 4 events)
    let history = ctx.client.get_credit_history(&credit_id);
    assert!(history.len() >= 4, "Should have at least 4 events");
}

#[test]
fn test_retire_different_project_types() {
    let (ctx, issuer) = setup_with_issuer();

    // Issue credits for different project types
    let reforestation = issue_credit_with_params(&ctx, &issuer, "Reforestation", "Brazil", "Verra", 2024, 1000);
    let solar = issue_credit_with_params(&ctx, &issuer, "Solar Energy", "India", "Gold Standard", 2024, 2000);
    let wind = issue_credit_with_params(&ctx, &issuer, "Wind Energy", "Denmark", "Verra", 2024, 1500);

    // Retire all credits
    retire_test_credit(&ctx, reforestation.clone(), &issuer, 1000);
    retire_test_credit(&ctx, solar.clone(), &issuer, 2000);
    retire_test_credit(&ctx, wind.clone(), &issuer, 1500);

    // Verify all retired
    assert_credit_status(&ctx, &reforestation, CreditStatus::Retired);
    assert_credit_status(&ctx, &solar, CreditStatus::Retired);
    assert_credit_status(&ctx, &wind, CreditStatus::Retired);

    // Check total
    let (_, _, _, total_retired) = ctx.client.get_contract_stats();
    assert_eq!(total_retired, 4500);
}

#[test]
fn test_retire_different_vintages() {
    let (ctx, issuer) = setup_with_issuer();

    // Issue credits with different vintage years
    let vintage_2020 = issue_credit_with_params(&ctx, &issuer, "Reforestation", "Brazil", "Verra", 2020, 1000);
    let vintage_2022 = issue_credit_with_params(&ctx, &issuer, "Reforestation", "Brazil", "Verra", 2022, 1000);
    let vintage_2024 = issue_credit_with_params(&ctx, &issuer, "Reforestation", "Brazil", "Verra", 2024, 1000);

    // Retire all vintages
    retire_test_credit(&ctx, vintage_2020, &issuer, 1000);
    retire_test_credit(&ctx, vintage_2022, &issuer, 1000);
    retire_test_credit(&ctx, vintage_2024, &issuer, 1000);

    // Check total retired
    let (_, _, _, total_retired) = ctx.client.get_contract_stats();
    assert_eq!(total_retired, 3000);
}

#[test]
fn test_mixed_operations_comprehensive() {
    let (ctx, issuer) = setup_with_issuer();

    // Issue multiple credits
    let credit1 = issue_test_credit(&ctx, &issuer, 2000);
    let credit2 = issue_test_credit(&ctx, &issuer, 3000);

    // Trade credit1
    let buyer1 = Address::generate(&ctx.env);
    trade_test_credit(&ctx, credit1.clone(), &issuer, &buyer1, 2000);

    // Partial retirement of credit1 by buyer1
    retire_test_credit(&ctx, credit1.clone(), &buyer1, 500);
    assert_credit_quantity(&ctx, &credit1, 1500);

    // Trade remaining credit1
    let buyer2 = Address::generate(&ctx.env);
    trade_test_credit(&ctx, credit1.clone(), &buyer1, &buyer2, 1500);

    // Retire credit2 fully by issuer
    retire_test_credit(&ctx, credit2.clone(), &issuer, 3000);
    assert_credit_status(&ctx, &credit2, CreditStatus::Retired);

    // Retire remaining credit1 by buyer2
    retire_test_credit(&ctx, credit1.clone(), &buyer2, 1500);
    assert_credit_status(&ctx, &credit1, CreditStatus::Retired);

    // Check total retired
    let (_, _, _, total_retired) = ctx.client.get_contract_stats();
    assert_eq!(total_retired, 5000);
}
