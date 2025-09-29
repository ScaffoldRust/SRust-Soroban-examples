#![cfg(test)]

use soroban_sdk::testutils::Address as _;
use crate::tests::utils::{setup_test_environment, mint_tokens};
use crate::utils::{OrderType, MarketplaceError};

#[test]
fn test_settle_trade_by_buyer() {
    let (env, client, _admin, token_contract, producer, consumer) = setup_test_environment();

    // Mint tokens for the consumer to pay with
    mint_tokens(&env, &token_contract, &_admin, &consumer, 10000);

    // Create a trade
    client.place_order(&producer, &OrderType::Sell, &100u64, &50u64);
    client.place_order(&consumer, &OrderType::Buy, &100u64, &60u64);

    // Get trade ID
    let trade_history = client.get_trade_history(&consumer);
    let trade = &trade_history.get(0).unwrap();
    let trade_id = trade.trade_id;

    // Settle the trade as buyer
    client.settle_trade(&trade_id, &consumer);

    // Verify trade still exists (settlement processes payment)
    let settled_trade = client.get_trade(&trade_id);
    assert_eq!(settled_trade.trade_id, trade_id);
}

#[test]
fn test_settle_trade_by_seller() {
    let (env, client, _admin, token_contract, producer, consumer) = setup_test_environment();

    // Mint tokens for the consumer to pay with
    mint_tokens(&env, &token_contract, &_admin, &consumer, 10000);

    // Create a trade
    client.place_order(&producer, &OrderType::Sell, &100u64, &50u64);
    client.place_order(&consumer, &OrderType::Buy, &100u64, &60u64);

    // Get trade ID
    let trade_history = client.get_trade_history(&producer);
    let trade = &trade_history.get(0).unwrap();
    let trade_id = trade.trade_id;

    // Settle the trade as seller
    client.settle_trade(&trade_id, &producer);

    // Verify trade still exists
    let settled_trade = client.get_trade(&trade_id);
    assert_eq!(settled_trade.trade_id, trade_id);
}

#[test]
fn test_settle_trade_unauthorized() {
    let (env, client, _admin, token_contract, producer, consumer) = setup_test_environment();

    // Mint tokens for the consumer
    mint_tokens(&env, &token_contract, &_admin, &consumer, 10000);

    // Create a trade
    client.place_order(&producer, &OrderType::Sell, &100u64, &50u64);
    client.place_order(&consumer, &OrderType::Buy, &100u64, &60u64);

    // Get trade ID
    let trade_history = client.get_trade_history(&consumer);
    let trade = &trade_history.get(0).unwrap();
    let trade_id = trade.trade_id;

    // Try to settle as unauthorized party
    let unauthorized = soroban_sdk::Address::generate(&env);
    let result = client.try_settle_trade(&trade_id, &unauthorized);
    assert_eq!(result, Err(Ok(MarketplaceError::NotAuthorized)));
}

#[test]
fn test_settle_nonexistent_trade() {
    let (_env, client, _admin, _token, _producer, consumer) = setup_test_environment();

    let result = client.try_settle_trade(&999u64, &consumer);
    assert_eq!(result, Err(Ok(MarketplaceError::TradeNotFound)));
}

#[test]
fn test_settlement_with_insufficient_balance() {
    let (_env, client, _admin, _token, producer, consumer) = setup_test_environment();

    // Don't mint tokens for consumer - they will have zero balance

    // Create a trade
    client.place_order(&producer, &OrderType::Sell, &100u64, &50u64);
    client.place_order(&consumer, &OrderType::Buy, &100u64, &60u64);

    // Get trade ID
    let trade_history = client.get_trade_history(&consumer);
    let trade = &trade_history.get(0).unwrap();
    let trade_id = trade.trade_id;

    // Try to settle - should fail due to insufficient balance
    let result = client.try_settle_trade(&trade_id, &consumer);
    assert_eq!(result, Err(Ok(MarketplaceError::PaymentFailed)));
}

#[test]
fn test_get_trade_details() {
    let (env, client, _admin, token_contract, producer, consumer) = setup_test_environment();

    // Mint tokens for the consumer
    mint_tokens(&env, &token_contract, &_admin, &consumer, 10000);

    // Create a trade
    client.place_order(&producer, &OrderType::Sell, &100u64, &50u64);
    client.place_order(&consumer, &OrderType::Buy, &100u64, &60u64);

    // Get trade details
    let trade_history = client.get_trade_history(&consumer);
    assert_eq!(trade_history.len(), 1);

    let trade = &trade_history.get(0).unwrap();
    assert_eq!(trade.buyer, consumer);
    assert_eq!(trade.seller, producer);
    assert_eq!(trade.quantity_kwh, 100u64);
    assert_eq!(trade.price_per_kwh, 50u64); // Seller's price
    assert_eq!(trade.total_amount, 5000u64); // 100 * 50

    // Verify trade can be retrieved by ID
    let retrieved_trade = client.get_trade(&trade.trade_id);
    assert_eq!(retrieved_trade.trade_id, trade.trade_id);
    assert_eq!(retrieved_trade.buyer, trade.buyer);
    assert_eq!(retrieved_trade.seller, trade.seller);
}

#[test]
fn test_get_trade_history_multiple_trades() {
    let (env, client, _admin, token_contract, producer, consumer) = setup_test_environment();

    // Create additional producer
    let producer2 = soroban_sdk::Address::generate(&env);
    client.register_producer(&producer2);

    // Mint tokens for the consumer
    mint_tokens(&env, &token_contract, &_admin, &consumer, 20000);

    // Create multiple trades with same consumer
    client.place_order(&producer, &OrderType::Sell, &100u64, &50u64);
    client.place_order(&consumer, &OrderType::Buy, &100u64, &60u64);

    client.place_order(&producer2, &OrderType::Sell, &200u64, &45u64);
    client.place_order(&consumer, &OrderType::Buy, &200u64, &55u64);

    // Verify trade history
    let trade_history = client.get_trade_history(&consumer);
    assert_eq!(trade_history.len(), 2);

    // Verify all trades involve the consumer as buyer
    for trade in trade_history.iter() {
        assert_eq!(trade.buyer, consumer);
    }
}

#[test]
fn test_get_empty_trade_history() {
    let (env, client, _admin, _token, _producer, _consumer) = setup_test_environment();

    // Create a trader who hasn't made any trades
    let new_trader = soroban_sdk::Address::generate(&env);
    client.register_consumer(&new_trader);

    let trade_history = client.get_trade_history(&new_trader);
    assert_eq!(trade_history.len(), 0);
}