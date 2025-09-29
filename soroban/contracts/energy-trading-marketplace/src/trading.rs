use crate::utils::*;
use soroban_sdk::{symbol_short, Address, Env, Map, Vec};

/// Place an order
pub fn place_order(
    env: &Env,
    trader: Address,
    order_type: OrderType,
    quantity_kwh: u64,
    price_per_kwh: u64,
) -> Result<u64, MarketplaceError> {
    let order_id = get_next_order_id(env);

    let order = EnergyOrder {
        order_id,
        trader: trader.clone(),
        order_type: order_type.clone(),
        quantity_kwh,
        price_per_kwh,
        timestamp: env.ledger().timestamp(),
        status: OrderStatus::Active,
    };

    // Store order
    let mut orders: Map<u64, EnergyOrder> = env
        .storage()
        .instance()
        .get(&DataKey::Orders)
        .unwrap_or_else(|| Map::new(env));

    orders.set(order_id, order.clone());
    env.storage().instance().set(&DataKey::Orders, &orders);

    // Try to match immediately
    match_order(env, order_id)?;

    Ok(order_id)
}

/// Match an order with existing orders
pub fn match_order(env: &Env, order_id: u64) -> Result<Vec<u64>, MarketplaceError> {
    let orders: Map<u64, EnergyOrder> = env.storage().instance().get(&DataKey::Orders).unwrap();
    let new_order = orders
        .get(order_id)
        .ok_or(MarketplaceError::OrderNotFound)?;

    if new_order.status != OrderStatus::Active {
        return Ok(Vec::new(env));
    }

    let mut matched_trades = Vec::new(env);

    // Find matching orders of opposite type
    let opposite_type = match new_order.order_type {
        OrderType::Buy => OrderType::Sell,
        OrderType::Sell => OrderType::Buy,
    };

    for (match_order_id, match_order) in orders.iter() {
        if match_order_id == order_id || match_order.status != OrderStatus::Active {
            continue;
        }

        if match_order.order_type == opposite_type && can_orders_match(&new_order, &match_order) {
            let trade_id = execute_trade(env, &new_order, &match_order)?;
            matched_trades.push_back(trade_id);
            break;
        }
    }

    Ok(matched_trades)
}

/// Check if two orders can match
fn can_orders_match(buy_order: &EnergyOrder, sell_order: &EnergyOrder) -> bool {
    let (buyer, seller) = if buy_order.order_type == OrderType::Buy {
        (buy_order, sell_order)
    } else {
        (sell_order, buy_order)
    };

    // Price matching: buy price >= sell price
    buyer.price_per_kwh >= seller.price_per_kwh && buyer.trader != seller.trader
}

/// Execute a trade between matching orders
fn execute_trade(
    env: &Env,
    order1: &EnergyOrder,
    order2: &EnergyOrder,
) -> Result<u64, MarketplaceError> {
    let trade_id = get_next_trade_id(env);

    let (buyer, seller) = if order1.order_type == OrderType::Buy {
        (order1, order2)
    } else {
        (order2, order1)
    };

    let quantity = buyer.quantity_kwh.min(seller.quantity_kwh);
    let price = seller.price_per_kwh; // Use seller's price
    let total_amount = quantity * price;

    let trade = Trade {
        trade_id,
        buyer: buyer.trader.clone(),
        seller: seller.trader.clone(),
        quantity_kwh: quantity,
        price_per_kwh: price,
        total_amount,
        timestamp: env.ledger().timestamp(),
    };

    // Store trade
    let mut trades: Map<u64, Trade> = env
        .storage()
        .instance()
        .get(&DataKey::Trades)
        .unwrap_or_else(|| Map::new(env));

    trades.set(trade_id, trade);
    env.storage().instance().set(&DataKey::Trades, &trades);

    // Mark orders as filled
    mark_orders_filled(env, order1.order_id, order2.order_id)?;

    // Emit trade event
    env.events().publish(
        (symbol_short!("trade"), trade_id),
        (quantity, price, total_amount),
    );

    Ok(trade_id)
}

/// Mark orders as filled
fn mark_orders_filled(env: &Env, order_id1: u64, order_id2: u64) -> Result<(), MarketplaceError> {
    let mut orders: Map<u64, EnergyOrder> = env.storage().instance().get(&DataKey::Orders).unwrap();

    if let Some(mut order1) = orders.get(order_id1) {
        order1.status = OrderStatus::Filled;
        orders.set(order_id1, order1);
    }

    if let Some(mut order2) = orders.get(order_id2) {
        order2.status = OrderStatus::Filled;
        orders.set(order_id2, order2);
    }

    env.storage().instance().set(&DataKey::Orders, &orders);
    Ok(())
}

/// Cancel an order
pub fn cancel_order(env: &Env, trader: Address, order_id: u64) -> Result<(), MarketplaceError> {
    let mut orders: Map<u64, EnergyOrder> = env.storage().instance().get(&DataKey::Orders).unwrap();
    let mut order = orders
        .get(order_id)
        .ok_or(MarketplaceError::OrderNotFound)?;

    if order.trader != trader {
        return Err(MarketplaceError::NotAuthorized);
    }

    if order.status != OrderStatus::Active {
        return Err(MarketplaceError::InvalidInput);
    }

    order.status = OrderStatus::Cancelled;
    orders.set(order_id, order);
    env.storage().instance().set(&DataKey::Orders, &orders);

    env.events()
        .publish((symbol_short!("cancelled"), trader), order_id);

    Ok(())
}

/// Get order details
pub fn get_order(env: &Env, order_id: u64) -> Result<EnergyOrder, MarketplaceError> {
    let orders: Map<u64, EnergyOrder> = env
        .storage()
        .instance()
        .get(&DataKey::Orders)
        .unwrap_or_else(|| Map::new(env));

    orders.get(order_id).ok_or(MarketplaceError::OrderNotFound)
}

fn get_next_order_id(env: &Env) -> u64 {
    let current_id: u64 = env
        .storage()
        .instance()
        .get(&DataKey::NextOrderId)
        .unwrap_or(1);
    env.storage()
        .instance()
        .set(&DataKey::NextOrderId, &(current_id + 1));
    current_id
}

fn get_next_trade_id(env: &Env) -> u64 {
    let current_id: u64 = env
        .storage()
        .instance()
        .get(&DataKey::NextTradeId)
        .unwrap_or(1);
    env.storage()
        .instance()
        .set(&DataKey::NextTradeId, &(current_id + 1));
    current_id
}
