#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Vec, BytesN};

mod math;
mod tick_bitmap;
mod pool;
mod liquidity;
mod swap;
mod fees;
mod position;

#[cfg(test)]
mod test;

pub use pool::{AmmError, create_pool, get_pool, get_pool_price, update_pool};
pub use liquidity::{add_liquidity, remove_liquidity};
pub use swap::{execute_swap, get_optimal_swap_path};
pub use fees::{collect_fees, calculate_fees_owed};
pub use position::{get_position, set_position_bounds, get_position_value};

// Core data structures
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Pool {
    pub token_a: Address,
    pub token_b: Address,
    pub reserve_a: i128,
    pub reserve_b: i128,
    pub fee_tier: u32, // in basis points (e.g., 30 = 0.3%)
    pub tick_spacing: u32,
    pub sqrt_price_x96: u128,
    pub liquidity: u128,
    pub fee_growth_global_0_x128: u128,
    pub fee_growth_global_1_x128: u128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Position {
    pub owner: Address,
    pub pool_id: BytesN<32>,
    pub liquidity: u128,
    pub price_lower: u128,
    pub price_upper: u128,
    pub fee_growth_inside_0_last_x128: u128,
    pub fee_growth_inside_1_last_x128: u128,
    pub tokens_owed_0: u128,
    pub tokens_owed_1: u128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PriceTick {
    pub price: u128,
    pub liquidity_net: i128,
    pub liquidity_gross: u128,
    pub fee_growth_outside_0_x128: u128,
    pub fee_growth_outside_1_x128: u128,
    pub initialized: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SwapPath {
    pub pools: Vec<BytesN<32>>,
    pub tokens: Vec<Address>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SwapParams {
    pub token_in: Address,
    pub token_out: Address,
    pub amount_in: i128,
    pub min_amount_out: i128,
    pub deadline: u64,
    pub recipient: Address,
}

// Storage keys
#[contracttype]
pub enum DataKey {
    Pool(BytesN<32>),
    Position(BytesN<32>),
    Tick(BytesN<32>, i32),
    TickBitmap(BytesN<32>, i32),
    PoolCount,
    PositionCount,
    Admin,
}

#[contract]
pub struct AmmContract;

#[contractimpl]
impl AmmContract {
    /// Initialize the contract with an admin
    pub fn initialize(env: Env, admin: Address) {
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::PoolCount, &0u32);
        env.storage().instance().set(&DataKey::PositionCount, &0u32);
    }

    /// Create a new liquidity pool
    pub fn create_pool(
        env: Env,
        token_a: Address,
        token_b: Address,
        fee_tier: u32,
        sqrt_price_x96: u128,
    ) -> Result<BytesN<32>, pool::AmmError> {
        pool::create_pool(env, token_a, token_b, fee_tier, sqrt_price_x96)
    }

    /// Add liquidity to a pool
    pub fn add_liquidity(
        env: Env,
        pool_id: BytesN<32>,
        amount_a_desired: i128,
        amount_b_desired: i128,
        amount_a_min: i128,
        amount_b_min: i128,
        price_lower: u128,
        price_upper: u128,
        recipient: Address,
        deadline: u64,
    ) -> Result<(BytesN<32>, u128, i128, i128), pool::AmmError> {
        liquidity::add_liquidity(
            env,
            pool_id,
            amount_a_desired,
            amount_b_desired,
            amount_a_min,
            amount_b_min,
            price_lower,
            price_upper,
            recipient,
            deadline,
        )
    }

    /// Remove liquidity from a pool
    pub fn remove_liquidity(
        env: Env,
        position_id: BytesN<32>,
        liquidity: u128,
        amount_0_min: i128,
        amount_1_min: i128,
        deadline: u64,
    ) -> Result<(i128, i128), pool::AmmError> {
        liquidity::remove_liquidity(env, position_id, liquidity, amount_0_min, amount_1_min, deadline)
    }

    /// Execute a token swap
    pub fn swap(
        env: Env,
        params: SwapParams,
    ) -> Result<i128, pool::AmmError> {
        swap::execute_swap(env, params)
    }

    /// Get optimal swap path between two tokens
    pub fn get_optimal_swap_path(
        env: Env,
        token_in: Address,
        token_out: Address,
        amount_in: i128,
    ) -> Result<SwapPath, pool::AmmError> {
        swap::get_optimal_swap_path(env, token_in, token_out, amount_in)
    }

    /// Collect fees from a position
    pub fn collect_fees(env: Env, position_id: BytesN<32>) -> Result<(u128, u128), pool::AmmError> {
        fees::collect_fees(env, position_id)
    }

    /// Get pool information
    pub fn get_pool(env: Env, pool_id: BytesN<32>) -> Result<Pool, pool::AmmError> {
        pool::get_pool(env, pool_id)
    }

    /// Get position information
    pub fn get_position(env: Env, position_id: BytesN<32>) -> Result<Position, pool::AmmError> {
        position::get_position(env, position_id)
    }

    /// Get current price of a pool
    pub fn get_pool_price(env: Env, pool_id: BytesN<32>) -> Result<u128, pool::AmmError> {
        let pool = pool::get_pool(env, pool_id)?;
        Ok(pool.sqrt_price_x96)
    }
}