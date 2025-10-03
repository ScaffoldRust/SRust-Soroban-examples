#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _},
    Address,
};

use crate::{CreditStatus};

use super::utils::*;

// ============ CREDIT TRADING TESTS ============

#[test]
fn test_trade_credit() {
    let (ctx, issuer) = setup_with_issuer();

    // Issue a credit
    let credit_id = issue_test_credit(&ctx, &issuer, 1000);

    // Trade the credit
    let buyer = Address::generate(&ctx.env);
    trade_test_credit(&ctx, credit_id.clone(), &issuer, &buyer, 1000);

    // Verify ownership changed
    assert_credit_owner(&ctx, &credit_id, &buyer);

    // Verify status changed to Traded
    assert_credit_status(&ctx, &credit_id, CreditStatus::Traded);
}

#[test]
fn test_multiple_trades() {
    let (ctx, issuer) = setup_with_issuer();

    // Issue a credit
    let credit_id = issue_test_credit(&ctx, &issuer, 1000);

    // First trade: issuer -> buyer1
    let buyer1 = Address::generate(&ctx.env);
    trade_test_credit(&ctx, credit_id.clone(), &issuer, &buyer1, 1000);
    assert_credit_owner(&ctx, &credit_id, &buyer1);

    // Second trade: buyer1 -> buyer2
    let buyer2 = Address::generate(&ctx.env);
    trade_test_credit(&ctx, credit_id.clone(), &buyer1, &buyer2, 1000);
    assert_credit_owner(&ctx, &credit_id, &buyer2);

    // Third trade: buyer2 -> buyer3
    let buyer3 = Address::generate(&ctx.env);
    trade_test_credit(&ctx, credit_id.clone(), &buyer2, &buyer3, 1000);
    assert_credit_owner(&ctx, &credit_id, &buyer3);
}

#[test]
fn test_trade_partial_credit() {
    let (ctx, issuer) = setup_with_issuer();

    // Issue a credit with 1000 quantity
    let credit_id = issue_test_credit(&ctx, &issuer, 1000);

    // Trade partial quantity
    let buyer = Address::generate(&ctx.env);
    trade_test_credit(&ctx, credit_id.clone(), &issuer, &buyer, 500);

    // Verify ownership changed
    assert_credit_owner(&ctx, &credit_id, &buyer);
}

#[test]
fn test_trading_history() {
    let (ctx, issuer) = setup_with_issuer();

    // Issue a credit
    let credit_id = issue_test_credit(&ctx, &issuer, 1000);

    // Make multiple trades
    let buyer1 = Address::generate(&ctx.env);
    let buyer2 = Address::generate(&ctx.env);

    trade_test_credit(&ctx, credit_id.clone(), &issuer, &buyer1, 1000);
    trade_test_credit(&ctx, credit_id.clone(), &buyer1, &buyer2, 1000);

    // Get credit history (should include issuance + 2 trades = 3 events)
    let history = ctx.client.get_credit_history(&credit_id);
    assert!(history.len() >= 3, "Should have at least 3 events (issuance + 2 trades)");
}

// ============ EDGE CASE TESTS ============

#[test]
#[should_panic]
fn test_trade_non_existent_credit() {
    let ctx = setup_test_env();

    let seller = Address::generate(&ctx.env);
    let buyer = Address::generate(&ctx.env);
    let non_existent_credit_id = soroban_sdk::BytesN::from_array(&ctx.env, &[99u8; 32]);

    // Attempt to trade non-existent credit - should panic
    trade_test_credit(&ctx, non_existent_credit_id, &seller, &buyer, 1000);
}

#[test]
fn test_trade_multiple_credits() {
    let (ctx, issuer) = setup_with_issuer();

    // Issue multiple credits
    let credit1 = issue_test_credit(&ctx, &issuer, 500);
    let credit2 = issue_test_credit(&ctx, &issuer, 1000);
    let credit3 = issue_test_credit(&ctx, &issuer, 1500);

    // Trade each credit to different buyers
    let buyer1 = Address::generate(&ctx.env);
    let buyer2 = Address::generate(&ctx.env);
    let buyer3 = Address::generate(&ctx.env);

    trade_test_credit(&ctx, credit1.clone(), &issuer, &buyer1, 500);
    trade_test_credit(&ctx, credit2.clone(), &issuer, &buyer2, 1000);
    trade_test_credit(&ctx, credit3.clone(), &issuer, &buyer3, 1500);

    // Verify each credit has correct owner
    assert_credit_owner(&ctx, &credit1, &buyer1);
    assert_credit_owner(&ctx, &credit2, &buyer2);
    assert_credit_owner(&ctx, &credit3, &buyer3);
}

#[test]
fn test_trade_credit_same_buyer_multiple_times() {
    let (ctx, issuer) = setup_with_issuer();

    // Issue credits
    let credit1 = issue_test_credit(&ctx, &issuer, 1000);
    let credit2 = issue_test_credit(&ctx, &issuer, 2000);

    // Same buyer purchases multiple credits
    let buyer = Address::generate(&ctx.env);

    trade_test_credit(&ctx, credit1.clone(), &issuer, &buyer, 1000);
    trade_test_credit(&ctx, credit2.clone(), &issuer, &buyer, 2000);

    // Verify both credits owned by same buyer
    assert_credit_owner(&ctx, &credit1, &buyer);
    assert_credit_owner(&ctx, &credit2, &buyer);
}

#[test]
fn test_trade_chain() {
    let (ctx, issuer) = setup_with_issuer();

    // Issue a credit
    let credit_id = issue_test_credit(&ctx, &issuer, 1000);

    // Create a chain of trades
    let buyer1 = Address::generate(&ctx.env);
    let buyer2 = Address::generate(&ctx.env);
    let buyer3 = Address::generate(&ctx.env);
    let buyer4 = Address::generate(&ctx.env);
    let buyer5 = Address::generate(&ctx.env);

    trade_test_credit(&ctx, credit_id.clone(), &issuer, &buyer1, 1000);
    trade_test_credit(&ctx, credit_id.clone(), &buyer1, &buyer2, 1000);
    trade_test_credit(&ctx, credit_id.clone(), &buyer2, &buyer3, 1000);
    trade_test_credit(&ctx, credit_id.clone(), &buyer3, &buyer4, 1000);
    trade_test_credit(&ctx, credit_id.clone(), &buyer4, &buyer5, 1000);

    // Verify final owner
    assert_credit_owner(&ctx, &credit_id, &buyer5);

    // Verify history has all trades
    let history = ctx.client.get_credit_history(&credit_id);
    assert!(history.len() >= 6, "Should have issuance + 5 trades");
}

#[test]
fn test_trade_different_verification_standards() {
    let (ctx, issuer) = setup_with_issuer();

    // Issue credits with different standards
    let verra_credit = issue_credit_with_params(&ctx, &issuer, "Reforestation", "Brazil", "Verra", 2024, 1000);
    let gold_credit = issue_credit_with_params(&ctx, &issuer, "Solar", "India", "Gold Standard", 2024, 1000);

    let buyer = Address::generate(&ctx.env);

    // Trade both credits
    trade_test_credit(&ctx, verra_credit.clone(), &issuer, &buyer, 1000);
    trade_test_credit(&ctx, gold_credit.clone(), &issuer, &buyer, 1000);

    // Verify both traded successfully
    assert_credit_owner(&ctx, &verra_credit, &buyer);
    assert_credit_owner(&ctx, &gold_credit, &buyer);
}
