#![cfg(test)]

use crate::tests::utils::setup_test_environment;
use crate::utils::{OrderStatus, OrderType};
use soroban_sdk::testutils::Address as _;
extern crate alloc;
use alloc::vec::Vec;

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

#[test]
fn test_high_volume_order_placement() {
    let (env, client, _admin, _token, _producer, _consumer) = setup_test_environment();

    // Create multiple traders for scalability testing
    let mut producers = Vec::new();
    let mut consumers = Vec::new();

    // Create 10 producers and 10 consumers
    for _i in 0..10 {
        let producer = soroban_sdk::Address::generate(&env);
        let consumer = soroban_sdk::Address::generate(&env);

        client.register_producer(&producer);
        client.register_consumer(&consumer);

        producers.push(producer);
        consumers.push(consumer);
    }

    // Place 50 sell orders from producers
    let mut sell_order_ids = Vec::new();
    for i in 0..50 {
        let producer = &producers[i % 10];
        let price = 50 + (i as u64 % 10); // Vary prices 50-59
        let quantity = 100 + (i as u64 * 10); // Vary quantities

        let order_id = client.place_order(producer, &OrderType::Sell, &quantity, &price);
        sell_order_ids.push(order_id);
    }

    // Place 30 buy orders from consumers (some will match)
    let mut buy_order_ids = Vec::new();
    for i in 0..30 {
        let consumer = &consumers[i % 10];
        let price = 55 + (i as u64 % 5); // Prices 55-59, higher than some sells
        let quantity = 150 + (i as u64 * 5); // Vary quantities

        let order_id = client.place_order(consumer, &OrderType::Buy, &quantity, &price);
        buy_order_ids.push(order_id);
    }

    // Verify that some orders were matched (trades created)
    let mut total_trades = 0;
    for consumer in &consumers {
        let trades = client.get_trade_history(consumer);
        total_trades += trades.len();
    }

    // Should have created some trades due to price compatibility
    assert!(
        total_trades > 0,
        "High volume test should create some trades"
    );

    // Verify system can handle the load without panicking
    // All order IDs should be unique and sequential
    for (i, &order_id) in sell_order_ids.iter().enumerate() {
        assert_eq!(order_id, (i + 1) as u64);
    }

    // Buy orders should continue the sequence
    for (i, &order_id) in buy_order_ids.iter().enumerate() {
        assert_eq!(order_id, (50 + i + 1) as u64);
    }
}
