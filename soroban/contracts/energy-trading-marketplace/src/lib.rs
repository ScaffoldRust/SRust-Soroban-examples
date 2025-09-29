#![no_std]

mod settlement;
mod trading;
mod utils;

use soroban_sdk::{contract, contractimpl, Address, Env, Map, Vec};

use crate::utils::*;

#[contract]
pub struct EnergyTradingMarketplace;

#[contractimpl]
impl EnergyTradingMarketplace {
    /// Initialize the marketplace
    pub fn initialize(
        env: Env,
        admin: Address,
        token_contract: Address,
        min_trade_size: u64,
        max_trade_size: u64,
    ) -> Result<(), MarketplaceError> {
        if env.storage().instance().has(&DataKey::Initialized) {
            return Err(MarketplaceError::AlreadyInitialized);
        }

        admin.require_auth();

        if min_trade_size == 0 || max_trade_size <= min_trade_size {
            return Err(MarketplaceError::InvalidInput);
        }

        env.storage().instance().set(&DataKey::Initialized, &true);
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::TokenContract, &token_contract);
        env.storage().instance().set(&DataKey::NextOrderId, &1u64);
        env.storage().instance().set(&DataKey::NextTradeId, &1u64);

        // Initialize empty maps
        let producers: Map<Address, bool> = Map::new(&env);
        let consumers: Map<Address, bool> = Map::new(&env);
        let grid_operators: Map<Address, bool> = Map::new(&env);
        let orders: Map<u64, EnergyOrder> = Map::new(&env);
        let trades: Map<u64, Trade> = Map::new(&env);

        env.storage()
            .instance()
            .set(&DataKey::Producers, &producers);
        env.storage()
            .instance()
            .set(&DataKey::Consumers, &consumers);
        env.storage()
            .instance()
            .set(&DataKey::GridOperators, &grid_operators);
        env.storage().instance().set(&DataKey::Orders, &orders);
        env.storage().instance().set(&DataKey::Trades, &trades);

        Ok(())
    }

    /// Register a producer
    pub fn register_producer(env: Env, producer: Address) -> Result<(), MarketplaceError> {
        Self::check_initialized(&env)?;
        producer.require_auth();

        let mut producers: Map<Address, bool> = env
            .storage()
            .instance()
            .get(&DataKey::Producers)
            .unwrap_or_else(|| Map::new(&env));

        producers.set(producer, true);
        env.storage()
            .instance()
            .set(&DataKey::Producers, &producers);

        Ok(())
    }

    /// Register a consumer
    pub fn register_consumer(env: Env, consumer: Address) -> Result<(), MarketplaceError> {
        Self::check_initialized(&env)?;
        consumer.require_auth();

        let mut consumers: Map<Address, bool> = env
            .storage()
            .instance()
            .get(&DataKey::Consumers)
            .unwrap_or_else(|| Map::new(&env));

        consumers.set(consumer, true);
        env.storage()
            .instance()
            .set(&DataKey::Consumers, &consumers);

        Ok(())
    }

    /// Place an energy order
    pub fn place_order(
        env: Env,
        trader: Address,
        order_type: OrderType,
        quantity_kwh: u64,
        price_per_kwh: u64,
    ) -> Result<u64, MarketplaceError> {
        Self::check_initialized(&env)?;
        trader.require_auth();

        Self::validate_trader_registration(&env, &trader)?;
        Self::validate_order_params(quantity_kwh, price_per_kwh)?;

        trading::place_order(&env, trader, order_type, quantity_kwh, price_per_kwh)
    }

    /// Cancel an order
    pub fn cancel_order(env: Env, trader: Address, order_id: u64) -> Result<(), MarketplaceError> {
        Self::check_initialized(&env)?;
        trader.require_auth();

        trading::cancel_order(&env, trader, order_id)
    }

    /// Settle a trade
    pub fn settle_trade(env: Env, trade_id: u64, settler: Address) -> Result<(), MarketplaceError> {
        Self::check_initialized(&env)?;
        settler.require_auth();

        settlement::settle_trade(&env, trade_id, settler)
    }

    /// Get order details
    pub fn get_order(env: Env, order_id: u64) -> Result<EnergyOrder, MarketplaceError> {
        Self::check_initialized(&env)?;
        trading::get_order(&env, order_id)
    }

    /// Get trade details
    pub fn get_trade(env: Env, trade_id: u64) -> Result<Trade, MarketplaceError> {
        Self::check_initialized(&env)?;
        settlement::get_trade(&env, trade_id)
    }

    /// Get trade history for a trader
    pub fn get_trade_history(env: Env, trader: Address) -> Result<Vec<Trade>, MarketplaceError> {
        Self::check_initialized(&env)?;
        get_trades_by_trader(&env, trader)
    }

    fn check_initialized(env: &Env) -> Result<(), MarketplaceError> {
        if !env.storage().instance().has(&DataKey::Initialized) {
            return Err(MarketplaceError::NotInitialized);
        }
        Ok(())
    }

    fn validate_trader_registration(env: &Env, trader: &Address) -> Result<(), MarketplaceError> {
        let producers: Map<Address, bool> = env
            .storage()
            .instance()
            .get(&DataKey::Producers)
            .unwrap_or_else(|| Map::new(env));

        let consumers: Map<Address, bool> = env
            .storage()
            .instance()
            .get(&DataKey::Consumers)
            .unwrap_or_else(|| Map::new(env));

        if producers.contains_key(trader.clone()) || consumers.contains_key(trader.clone()) {
            Ok(())
        } else {
            Err(MarketplaceError::TraderNotRegistered)
        }
    }

    fn validate_order_params(
        quantity_kwh: u64,
        price_per_kwh: u64,
    ) -> Result<(), MarketplaceError> {
        if quantity_kwh == 0 {
            return Err(MarketplaceError::QuantityOutOfRange);
        }
        if price_per_kwh == 0 {
            return Err(MarketplaceError::PriceOutOfRange);
        }
        Ok(())
    }
}
