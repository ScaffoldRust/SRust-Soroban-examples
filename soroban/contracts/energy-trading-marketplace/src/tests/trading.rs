#![cfg(test)]

use soroban_sdk::testutils::Address as _;
use crate::tests::utils::{setup_test_environment};
use crate::utils::{OrderType, OrderStatus, MarketplaceError};

#[test]
fn test_place_sell_order() {
    let (_env, client, _admin, _token, producer, _consumer) = setup_test_environment();

    let order_id = client.place_order(&producer, &OrderType::Sell, &100u64, &50u64);

    let order = client.get_order(&order_id);
    assert_eq!(order.trader, producer);
    assert_eq!(order.order_type, OrderType::Sell);
    assert_eq!(order.quantity_kwh, 100u64);
    assert_eq!(order.price_per_kwh, 50u64);
    assert_eq!(order.status, OrderStatus::Active);
}

#[test]
fn test_place_buy_order() {
    let (_env, client, _admin, _token, _producer, consumer) = setup_test_environment();

    let order_id = client.place_order(&consumer, &OrderType::Buy, &100u64, &60u64);

    let order = client.get_order(&order_id);
    assert_eq!(order.trader, consumer);
    assert_eq!(order.order_type, OrderType::Buy);
    assert_eq!(order.quantity_kwh, 100u64);
    assert_eq!(order.price_per_kwh, 60u64);
    assert_eq!(order.status, OrderStatus::Active);
}

#[test]
fn test_place_order_unregistered_trader() {
    let (env, client, _admin, _token, _producer, _consumer) = setup_test_environment();

    let unregistered = soroban_sdk::Address::generate(&env);
    let result = client.try_place_order(&unregistered, &OrderType::Sell, &100u64, &50u64);
    assert_eq!(result, Err(Ok(MarketplaceError::TraderNotRegistered)));
}

#[test]
fn test_place_order_zero_quantity() {
    let (_env, client, _admin, _token, producer, _consumer) = setup_test_environment();

    let result = client.try_place_order(&producer, &OrderType::Sell, &0u64, &50u64);
    assert_eq!(result, Err(Ok(MarketplaceError::QuantityOutOfRange)));
}

#[test]
fn test_place_order_zero_price() {
    let (_env, client, _admin, _token, producer, _consumer) = setup_test_environment();

    let result = client.try_place_order(&producer, &OrderType::Sell, &100u64, &0u64);
    assert_eq!(result, Err(Ok(MarketplaceError::PriceOutOfRange)));
}

#[test]
fn test_cancel_order() {
    let (_env, client, _admin, _token, producer, _consumer) = setup_test_environment();

    let order_id = client.place_order(&producer, &OrderType::Sell, &100u64, &50u64);
    client.cancel_order(&producer, &order_id);

    let order = client.get_order(&order_id);
    assert_eq!(order.status, OrderStatus::Cancelled);
}

#[test]
fn test_cancel_order_unauthorized() {
    let (env, client, _admin, _token, producer, _consumer) = setup_test_environment();

    let order_id = client.place_order(&producer, &OrderType::Sell, &100u64, &50u64);

    let unauthorized = soroban_sdk::Address::generate(&env);
    let result = client.try_cancel_order(&unauthorized, &order_id);
    assert_eq!(result, Err(Ok(MarketplaceError::NotAuthorized)));
}

#[test]
fn test_cancel_nonexistent_order() {
    let (_env, client, _admin, _token, producer, _consumer) = setup_test_environment();

    let result = client.try_cancel_order(&producer, &999u64);
    assert_eq!(result, Err(Ok(MarketplaceError::OrderNotFound)));
}