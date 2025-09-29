use soroban_sdk::{contracterror, contracttype, Address, Env, Map, Vec};

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
    TokenContract = 9,
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
    pub status: OrderStatus,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct Trade {
    pub trade_id: u64,
    pub buyer: Address,
    pub seller: Address,
    pub quantity_kwh: u64,
    pub price_per_kwh: u64,
    pub total_amount: u64,
    pub timestamp: u64,
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
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
#[repr(u32)]
pub enum TraderRole {
    Producer = 0,
    Consumer = 1,
    GridOperator = 2,
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
    PriceOutOfRange = 7,
    TraderNotRegistered = 8,
    QuantityOutOfRange = 9,
    PaymentFailed = 10,
}

pub fn get_trades_by_trader(env: &Env, trader: Address) -> Result<Vec<Trade>, MarketplaceError> {
    let trades: Map<u64, Trade> = env
        .storage()
        .instance()
        .get(&DataKey::Trades)
        .unwrap_or_else(|| Map::new(env));

    let mut trader_trades = Vec::new(env);
    for (_, trade) in trades.iter() {
        if trade.buyer == trader || trade.seller == trader {
            trader_trades.push_back(trade);
        }
    }
    Ok(trader_trades)
}
