#![no_std]

mod settlement;
mod trading;
mod utils;

#[cfg(test)]
mod test;

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, Map, String,
    Vec,
};

#[contracttype]
#[derive(Copy, Clone)]
#[repr(u32)]
pub enum DataKey {
    Initialized = 0,
    Admin = 1,
    GridOperators = 2,
    Producers = 3,
    Consumers = 4,
    Orders = 5,
    Trades = 6,
    NextOrderId = 7,
    NextTradeId = 8,
    MarketConfig = 9,
    PendingSettlements = 10,
    TradeHistory = 11,
    OrderBook = 12,
    UserOrders = 13,
    MarketStats = 14,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct EnergyOrder {
    pub order_id: u64,
    pub trader: Address,
    pub order_type: OrderType,
    pub quantity_kwh: u64,
    pub price_per_kwh: u64,
    pub timestamp: u64,
    pub order_expiry: u64, // When this order expires and gets cancelled
    pub status: OrderStatus,
    pub location: String,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct Trade {
    pub trade_id: u64,
    pub buy_order_id: u64,
    pub sell_order_id: u64,
    pub buyer: Address,
    pub seller: Address,
    pub quantity_kwh: u64,
    pub price_per_kwh: u64,
    pub total_amount: u64,
    pub timestamp: u64,
    pub settlement_status: SettlementStatus,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct TraderInfo {
    pub address: Address,
    pub role: TraderRole,
    pub registered_at: u64,
    pub verification_status: VerificationStatus,
    pub total_energy_traded: u64,
    pub reputation_score: u32,
    pub location: String,
    pub certificates: Vec<String>,
    pub active_orders: Vec<u64>,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct MarketConfig {
    pub trading_fee_rate: u32,   // basis points (100 = 1%)
    pub minimum_trade_size: u64, // kWh
    pub maximum_trade_size: u64, // kWh
    pub price_precision: u32,    // decimal places
    pub settlement_timeout: u64, // seconds
    pub dispute_period: u64,     // seconds
    pub max_order_duration: u64, // seconds
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct MarketStats {
    pub total_orders: u64,
    pub total_trades: u64,
    pub total_energy_traded: u64,
    pub average_price: u64,
    pub active_orders: u64,
    pub last_trade_price: u64,
    pub last_updated: u64,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
#[repr(u32)]
pub enum OrderType {
    Buy = 0,
    Sell = 1,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
#[repr(u32)]
pub enum OrderStatus {
    Active = 0,
    Filled = 1,
    Cancelled = 2,
    Expired = 3,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
#[repr(u32)]
pub enum SettlementStatus {
    Pending = 0,
    InProgress = 1,
    Completed = 2,
    Failed = 3,
    Disputed = 4,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
#[repr(u32)]
pub enum TraderRole {
    Producer = 0,
    Consumer = 1,
    GridOperator = 2,
    Trader = 3,
    MarketMaker = 4,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
#[repr(u32)]
pub enum VerificationStatus {
    Unverified = 0,
    Pending = 1,
    Verified = 2,
    Rejected = 3,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[repr(u32)]
pub enum MarketplaceError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    NotAuthorized = 3,
    InvalidInput = 4,
    OrderNotFound = 5,
    TradeNotFound = 6,
    InsufficientBalance = 7,
    OrderExpired = 8,
    InvalidOrderType = 9,
    PriceOutOfRange = 11,
    TraderNotRegistered = 13,
    TraderNotVerified = 14,
    OrderAlreadyFilled = 15,
    SettlementFailed = 16,

    InsufficientLiquidity = 18,

    DuplicateOrder = 20,
    QuantityOutOfRange = 21,
    GridOperatorRequired = 22,
    TradingFeeTooHigh = 23,
    DisputePeriodActive = 24,
    SettlementTimeout = 25,
    InvalidCertificate = 26,
    LocationMismatch = 27,
}

#[contract]
pub struct EnergyTradingMarketplace;

#[contractimpl]
impl EnergyTradingMarketplace {
    /// Initialize the marketplace with admin and market configuration
    pub fn initialize(
        env: Env,
        admin: Address,
        trading_fee_rate: u32,
        minimum_trade_size: u64,
        maximum_trade_size: u64,
    ) -> Result<(), MarketplaceError> {
        if env.storage().instance().has(&DataKey::Initialized) {
            return Err(MarketplaceError::AlreadyInitialized);
        }

        admin.require_auth();

        // Validate input parameters
        if trading_fee_rate > 1000 {
            // max 10%
            return Err(MarketplaceError::TradingFeeTooHigh);
        }
        if minimum_trade_size == 0 || maximum_trade_size <= minimum_trade_size {
            return Err(MarketplaceError::InvalidInput);
        }

        // Initialize storage
        env.storage().instance().set(&DataKey::Initialized, &true);
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::NextOrderId, &1u64);
        env.storage().instance().set(&DataKey::NextTradeId, &1u64);

        // Initialize maps
        let grid_operators: Map<Address, bool> = Map::new(&env);
        env.storage()
            .instance()
            .set(&DataKey::GridOperators, &grid_operators);

        let producers: Map<Address, TraderInfo> = Map::new(&env);
        env.storage()
            .instance()
            .set(&DataKey::Producers, &producers);

        let consumers: Map<Address, TraderInfo> = Map::new(&env);
        env.storage()
            .instance()
            .set(&DataKey::Consumers, &consumers);

        let orders: Map<u64, EnergyOrder> = Map::new(&env);
        env.storage().instance().set(&DataKey::Orders, &orders);

        let trades: Map<u64, Trade> = Map::new(&env);
        env.storage().instance().set(&DataKey::Trades, &trades);

        let pending_settlements: Map<u64, Trade> = Map::new(&env);
        env.storage()
            .instance()
            .set(&DataKey::PendingSettlements, &pending_settlements);

        let order_book: Map<String, Vec<u64>> = Map::new(&env); // key: energy_type_order_type
        env.storage()
            .instance()
            .set(&DataKey::OrderBook, &order_book);

        let user_orders: Map<Address, Vec<u64>> = Map::new(&env);
        env.storage()
            .instance()
            .set(&DataKey::UserOrders, &user_orders);

        // Market configuration
        let market_config = MarketConfig {
            trading_fee_rate,
            minimum_trade_size,
            maximum_trade_size,
            price_precision: 6,         // stroops precision
            settlement_timeout: 3600,   // 1 hour
            dispute_period: 86400,      // 24 hours
            max_order_duration: 604800, // 1 week
        };
        env.storage()
            .instance()
            .set(&DataKey::MarketConfig, &market_config);

        // Initialize market stats
        let market_stats = MarketStats {
            total_orders: 0,
            total_trades: 0,
            total_energy_traded: 0,
            average_price: 0,
            active_orders: 0,
            last_trade_price: 0,
            last_updated: env.ledger().timestamp(),
        };
        env.storage()
            .instance()
            .set(&DataKey::MarketStats, &market_stats);

        // Emit initialization event
        env.events().publish(
            (symbol_short!("init"), admin.clone()),
            (trading_fee_rate, minimum_trade_size, maximum_trade_size),
        );

        Ok(())
    }

    /// Register a new trader (producer or consumer)
    pub fn register_trader(
        env: Env,
        trader: Address,
        role: TraderRole,
        location: String,
        certificates: Vec<String>,
    ) -> Result<(), MarketplaceError> {
        Self::check_initialized(&env)?;
        trader.require_auth();

        let trader_info = TraderInfo {
            address: trader.clone(),
            role: role.clone(),
            registered_at: env.ledger().timestamp(),
            verification_status: VerificationStatus::Pending,
            total_energy_traded: 0,
            reputation_score: 100, // starting score
            location,
            certificates,
            active_orders: Vec::new(&env),
        };

        // Store in appropriate map based on role
        match role {
            TraderRole::Producer | TraderRole::MarketMaker => {
                let mut producers: Map<Address, TraderInfo> = env
                    .storage()
                    .instance()
                    .get(&DataKey::Producers)
                    .unwrap_or_else(|| Map::new(&env));
                producers.set(trader.clone(), trader_info);
                env.storage()
                    .instance()
                    .set(&DataKey::Producers, &producers);
            }
            TraderRole::Consumer | TraderRole::Trader => {
                let mut consumers: Map<Address, TraderInfo> = env
                    .storage()
                    .instance()
                    .get(&DataKey::Consumers)
                    .unwrap_or_else(|| Map::new(&env));
                consumers.set(trader.clone(), trader_info);
                env.storage()
                    .instance()
                    .set(&DataKey::Consumers, &consumers);
            }
            TraderRole::GridOperator => {
                let mut grid_operators: Map<Address, bool> = env
                    .storage()
                    .instance()
                    .get(&DataKey::GridOperators)
                    .unwrap_or_else(|| Map::new(&env));
                grid_operators.set(trader.clone(), true);
                env.storage()
                    .instance()
                    .set(&DataKey::GridOperators, &grid_operators);
            }
        }

        // Emit registration event
        env.events()
            .publish((symbol_short!("reg_trade"), trader), role);

        Ok(())
    }

    /// Place a new energy order
    pub fn place_order(
        env: Env,
        trader: Address,
        order_type: OrderType,
        quantity_kwh: u64,
        price_per_kwh: u64,
        location: String,
    ) -> Result<u64, MarketplaceError> {
        Self::check_initialized(&env)?;
        trader.require_auth();

        // Validate trader registration
        Self::validate_trader_registration(&env, &trader)?;

        // Validate order parameters
        Self::validate_order_params(&env, quantity_kwh, price_per_kwh)?;

        // Generate order ID
        let order_id = Self::get_next_order_id(&env);
        let timestamp = env.ledger().timestamp();
        let market_config: MarketConfig = env
            .storage()
            .instance()
            .get(&DataKey::MarketConfig)
            .unwrap();
        let order_expiry = timestamp + market_config.max_order_duration;

        // Create order
        let order = EnergyOrder {
            order_id,
            trader: trader.clone(),
            order_type: order_type.clone(),
            quantity_kwh,
            price_per_kwh,
            timestamp,
            order_expiry,
            status: OrderStatus::Active,
            location,
        };

        // Store order
        trading::store_order(&env, order)?;

        // Try to match order immediately
        let matched_trades = trading::match_order(&env, order_id)?;

        // Update market statistics
        utils::update_market_stats(&env, 1, 0, 0)?;

        // Emit order placement event
        env.events().publish(
            (symbol_short!("ord_place"), trader),
            (order_id, quantity_kwh, price_per_kwh),
        );

        // Emit trade events for any immediate matches
        for trade_id in matched_trades.iter() {
            env.events()
                .publish((symbol_short!("trd_match"), trade_id), order_id);
        }

        Ok(order_id)
    }

    /// Cancel an existing order
    pub fn cancel_order(env: Env, trader: Address, order_id: u64) -> Result<(), MarketplaceError> {
        Self::check_initialized(&env)?;
        trader.require_auth();

        trading::cancel_order(&env, trader, order_id)
    }

    /// Complete settlement for a trade
    pub fn settle_trade(
        env: Env,
        trade_id: u64,
        settler: Address,
        delivery_confirmation: Option<String>,
    ) -> Result<(), MarketplaceError> {
        Self::check_initialized(&env)?;
        settler.require_auth();

        settlement::settle_trade(&env, trade_id, settler, delivery_confirmation)
    }

    /// Dispute a trade settlement
    pub fn dispute_trade(
        env: Env,
        trade_id: u64,
        disputer: Address,
        reason: String,
        evidence: Option<String>,
    ) -> Result<(), MarketplaceError> {
        Self::check_initialized(&env)?;
        disputer.require_auth();

        settlement::dispute_trade(&env, trade_id, disputer, reason, evidence)
    }

    /// Verify a trader (admin or grid operator only)
    pub fn verify_trader(
        env: Env,
        verifier: Address,
        trader: Address,
        verification_status: VerificationStatus,
    ) -> Result<(), MarketplaceError> {
        Self::check_initialized(&env)?;
        verifier.require_auth();

        // Check if verifier is admin or grid operator
        Self::check_verification_authority(&env, &verifier)?;

        utils::update_trader_verification(&env, trader, verification_status)
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

    /// Get trader information
    pub fn get_trader_info(env: Env, trader: Address) -> Result<TraderInfo, MarketplaceError> {
        Self::check_initialized(&env)?;
        utils::get_trader_info(&env, trader)
    }

    /// Get active orders for a trader
    pub fn get_trader_orders(
        env: Env,
        trader: Address,
    ) -> Result<Vec<EnergyOrder>, MarketplaceError> {
        Self::check_initialized(&env)?;
        trading::get_trader_orders(&env, trader)
    }

    /// Get market statistics
    pub fn get_market_stats(env: Env) -> Result<MarketStats, MarketplaceError> {
        Self::check_initialized(&env)?;
        let stats: MarketStats = env.storage().instance().get(&DataKey::MarketStats).unwrap();
        Ok(stats)
    }

    /// Get order book for specific energy type
    pub fn get_order_book(
        env: Env,
        order_type: OrderType,
    ) -> Result<Vec<EnergyOrder>, MarketplaceError> {
        Self::check_initialized(&env)?;
        trading::get_order_book(&env, order_type)
    }

    /// Get recent trades
    pub fn get_recent_trades(env: Env, limit: u32) -> Result<Vec<Trade>, MarketplaceError> {
        Self::check_initialized(&env)?;
        settlement::get_recent_trades(&env, limit)
    }

    /// Get price history for energy type
    pub fn get_price_history(env: Env, hours: u32) -> Result<Vec<u64>, MarketplaceError> {
        Self::check_initialized(&env)?;
        utils::get_price_history(&env, hours)
    }

    /// Update market configuration (admin only)
    pub fn update_market_config(
        env: Env,
        admin: Address,
        new_config: MarketConfig,
    ) -> Result<(), MarketplaceError> {
        Self::check_initialized(&env)?;
        admin.require_auth();
        Self::check_admin(&env, &admin)?;

        env.storage()
            .instance()
            .set(&DataKey::MarketConfig, &new_config);

        env.events().publish(
            (symbol_short!("cfg_upd"), admin),
            new_config.trading_fee_rate,
        );

        Ok(())
    }

    /// Add grid operator (admin only)
    pub fn add_grid_operator(
        env: Env,
        admin: Address,
        operator: Address,
    ) -> Result<(), MarketplaceError> {
        Self::check_initialized(&env)?;
        admin.require_auth();
        Self::check_admin(&env, &admin)?;

        let mut grid_operators: Map<Address, bool> = env
            .storage()
            .instance()
            .get(&DataKey::GridOperators)
            .unwrap_or_else(|| Map::new(&env));

        grid_operators.set(operator.clone(), true);
        env.storage()
            .instance()
            .set(&DataKey::GridOperators, &grid_operators);

        env.events()
            .publish((symbol_short!("grid_add"), admin), operator);

        Ok(())
    }

    fn check_initialized(env: &Env) -> Result<(), MarketplaceError> {
        if !env.storage().instance().has(&DataKey::Initialized) {
            return Err(MarketplaceError::NotInitialized);
        }
        Ok(())
    }

    fn check_admin(env: &Env, caller: &Address) -> Result<(), MarketplaceError> {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if *caller != admin {
            return Err(MarketplaceError::NotAuthorized);
        }
        Ok(())
    }

    fn check_verification_authority(env: &Env, verifier: &Address) -> Result<(), MarketplaceError> {
        // Check if admin
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if *verifier == admin {
            return Ok(());
        }

        // Check if grid operator
        let grid_operators: Map<Address, bool> = env
            .storage()
            .instance()
            .get(&DataKey::GridOperators)
            .unwrap_or_else(|| Map::new(env));

        if grid_operators.get(verifier.clone()).unwrap_or(false) {
            return Ok(());
        }

        Err(MarketplaceError::NotAuthorized)
    }

    fn validate_trader_registration(env: &Env, trader: &Address) -> Result<(), MarketplaceError> {
        // Check if trader is registered in any category
        let producers: Map<Address, TraderInfo> = env
            .storage()
            .instance()
            .get(&DataKey::Producers)
            .unwrap_or_else(|| Map::new(env));

        let consumers: Map<Address, TraderInfo> = env
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
        env: &Env,
        quantity_kwh: u64,
        price_per_kwh: u64,
    ) -> Result<(), MarketplaceError> {
        let market_config: MarketConfig = env
            .storage()
            .instance()
            .get(&DataKey::MarketConfig)
            .unwrap();

        if quantity_kwh < market_config.minimum_trade_size {
            return Err(MarketplaceError::QuantityOutOfRange);
        }

        if quantity_kwh > market_config.maximum_trade_size {
            return Err(MarketplaceError::QuantityOutOfRange);
        }

        if price_per_kwh == 0 {
            return Err(MarketplaceError::PriceOutOfRange);
        }

        Ok(())
    }

    fn get_next_order_id(env: &Env) -> u64 {
        let current_id: u64 = env.storage().instance().get(&DataKey::NextOrderId).unwrap();
        env.storage()
            .instance()
            .set(&DataKey::NextOrderId, &(current_id + 1));
        current_id
    }

    pub fn get_next_trade_id(env: &Env) -> u64 {
        let current_id: u64 = env.storage().instance().get(&DataKey::NextTradeId).unwrap();
        env.storage()
            .instance()
            .set(&DataKey::NextTradeId, &(current_id + 1));
        current_id
    }
}
