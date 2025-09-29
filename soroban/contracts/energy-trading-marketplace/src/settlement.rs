use soroban_sdk::{Env, Map, Address, symbol_short, token};
use crate::utils::*;

/// Settle a trade with payment transfer
pub fn settle_trade(env: &Env, trade_id: u64, settler: Address) -> Result<(), MarketplaceError> {
    let trades: Map<u64, Trade> = env
        .storage()
        .instance()
        .get(&DataKey::Trades)
        .unwrap_or_else(|| Map::new(env));

    let trade = trades.get(trade_id).ok_or(MarketplaceError::TradeNotFound)?;

    // Verify authorization (only buyer or seller can settle)
    if settler != trade.buyer && settler != trade.seller {
        return Err(MarketplaceError::NotAuthorized);
    }

    // Execute payment transfer
    execute_payment(&env, &trade)?;

    // Emit settlement event
    env.events().publish(
        (symbol_short!("settled"), trade_id),
        (trade.quantity_kwh, trade.total_amount),
    );

    Ok(())
}

/// Execute payment transfer using Stellar tokens
fn execute_payment(env: &Env, trade: &Trade) -> Result<(), MarketplaceError> {
    // Get token contract
    let token_contract: Address = env
        .storage()
        .instance()
        .get(&DataKey::TokenContract)
        .ok_or(MarketplaceError::PaymentFailed)?;

    // Require buyer authorization
    trade.buyer.require_auth();

    // Execute transfer from buyer to seller
    let token_client = token::Client::new(env, &token_contract);
    token_client.transfer(&trade.buyer, &trade.seller, &(trade.total_amount as i128));

    // Emit payment event
    env.events().publish(
        (symbol_short!("payment"), trade.buyer.clone()),
        (trade.seller.clone(), trade.total_amount),
    );

    Ok(())
}

/// Get trade details
pub fn get_trade(env: &Env, trade_id: u64) -> Result<Trade, MarketplaceError> {
    let trades: Map<u64, Trade> = env
        .storage()
        .instance()
        .get(&DataKey::Trades)
        .unwrap_or_else(|| Map::new(env));

    trades.get(trade_id).ok_or(MarketplaceError::TradeNotFound)
}