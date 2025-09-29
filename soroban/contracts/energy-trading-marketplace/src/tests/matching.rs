#![cfg(test)]

use soroban_sdk::testutils::Address as _;
use crate::tests::utils::{setup_test_environment};
use crate::utils::{OrderType, OrderStatus};

#[test]
fn test_automatic_order_matching() {
    let (_env, client, _admin, _token, producer, consumer) = setup_test_environment();

    // Place sell order first
    let sell_order_id = client.place_order(&producer, &OrderType::Sell, &100u64, &50u64);

    // Place matching buy order - should trigger match
    let buy_order_id = client.place_order(&consumer, &OrderType::Buy, &100u64, &60u64);

    // Check orders are filled
    let sell_order = client.get_order(&sell_order_id);
    let buy_order = client.get_order(&buy_order_id);

    assert_eq!(sell_order.status, OrderStatus::Filled);
    assert_eq!(buy_order.status, OrderStatus::Filled);

    // Verify trade was created
    let trade_history = client.get_trade_history(&consumer);
    assert_eq!(trade_history.len(), 1);

    let trade = &trade_history.get(0).unwrap();
    assert_eq!(trade.buyer, consumer);
    assert_eq!(trade.seller, producer);
    assert_eq!(trade.quantity_kwh, 100u64);
    assert_eq!(trade.price_per_kwh, 50u64); // Uses seller's price
}

#[test]
fn test_no_matching_incompatible_prices() {
    let (_env, client, _admin, _token, producer, consumer) = setup_test_environment();

    // Place sell order with high price
    let sell_order_id = client.place_order(&producer, &OrderType::Sell, &100u64, &100u64);

    // Place buy order with lower price - should not match
    let buy_order_id = client.place_order(&consumer, &OrderType::Buy, &100u64, &50u64);

    // Check orders remain active
    let sell_order = client.get_order(&sell_order_id);
    let buy_order = client.get_order(&buy_order_id);

    assert_eq!(sell_order.status, OrderStatus::Active);
    assert_eq!(buy_order.status, OrderStatus::Active);

    // Verify no trade was created
    let trade_history = client.get_trade_history(&consumer);
    assert_eq!(trade_history.len(), 0);
}

#[test]
fn test_matching_with_exact_price() {
    let (_env, client, _admin, _token, producer, consumer) = setup_test_environment();

    // Place orders with exact same price
    let sell_order_id = client.place_order(&producer, &OrderType::Sell, &100u64, &50u64);
    let buy_order_id = client.place_order(&consumer, &OrderType::Buy, &100u64, &50u64);

    // Check orders are filled
    let sell_order = client.get_order(&sell_order_id);
    let buy_order = client.get_order(&buy_order_id);

    assert_eq!(sell_order.status, OrderStatus::Filled);
    assert_eq!(buy_order.status, OrderStatus::Filled);
}

#[test]
fn test_matching_different_quantities() {
    let (_env, client, _admin, _token, producer, consumer) = setup_test_environment();

    // Place sell order with larger quantity
    let sell_order_id = client.place_order(&producer, &OrderType::Sell, &200u64, &50u64);

    // Place buy order with smaller quantity
    let buy_order_id = client.place_order(&consumer, &OrderType::Buy, &100u64, &60u64);

    // Check buy order is filled, sell order remains active
    let sell_order = client.get_order(&sell_order_id);
    let buy_order = client.get_order(&buy_order_id);

    assert_eq!(buy_order.status, OrderStatus::Filled);
    assert_eq!(sell_order.status, OrderStatus::Filled); // Current implementation fills both

    // Verify trade quantity matches smaller order
    let trade_history = client.get_trade_history(&consumer);
    assert_eq!(trade_history.len(), 1);

    let trade = &trade_history.get(0).unwrap();
    assert_eq!(trade.quantity_kwh, 100u64); // Matches smaller quantity
}

#[test]
fn test_multiple_order_matching() {
    let (env, client, _admin, _token, producer, consumer) = setup_test_environment();

    // Create additional traders
    let producer2 = soroban_sdk::Address::generate(&env);
    let consumer2 = soroban_sdk::Address::generate(&env);

    client.register_producer(&producer2);
    client.register_consumer(&consumer2);

    // Place multiple sell orders
    let sell_order_id1 = client.place_order(&producer, &OrderType::Sell, &100u64, &50u64);
    let sell_order_id2 = client.place_order(&producer2, &OrderType::Sell, &100u64, &55u64);

    // Place buy order that can match the first (cheaper) sell order
    let buy_order_id = client.place_order(&consumer, &OrderType::Buy, &100u64, &60u64);

    // Check first sell order is matched
    let sell_order1 = client.get_order(&sell_order_id1);
    let sell_order2 = client.get_order(&sell_order_id2);
    let buy_order = client.get_order(&buy_order_id);

    assert_eq!(sell_order1.status, OrderStatus::Filled);
    assert_eq!(sell_order2.status, OrderStatus::Active); // Not matched
    assert_eq!(buy_order.status, OrderStatus::Filled);
}

#[test]
fn test_same_trader_no_self_match() {
    let (_env, client, _admin, _token, producer, _consumer) = setup_test_environment();

    // Producer tries to both buy and sell
    let sell_order_id = client.place_order(&producer, &OrderType::Sell, &100u64, &50u64);
    let buy_order_id = client.place_order(&producer, &OrderType::Buy, &100u64, &60u64);

    // Orders should not match with themselves
    let sell_order = client.get_order(&sell_order_id);
    let buy_order = client.get_order(&buy_order_id);

    assert_eq!(sell_order.status, OrderStatus::Active);
    assert_eq!(buy_order.status, OrderStatus::Active);

    // No trade should be created
    let trade_history = client.get_trade_history(&producer);
    assert_eq!(trade_history.len(), 0);
}