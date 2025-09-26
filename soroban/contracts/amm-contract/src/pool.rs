use soroban_sdk::{Address, BytesN, Env, contracterror, xdr::ToXdr};
use crate::{DataKey, Pool, math};

// Import constants from math module
use crate::math::{MIN_SQRT_RATIO, MAX_SQRT_RATIO};

// Fee tier constants (in basis points)
pub const FEE_TIER_LOW: u32 = 5;      // 0.05%
pub const FEE_TIER_MEDIUM: u32 = 30;   // 0.30%
pub const FEE_TIER_HIGH: u32 = 100;    // 1.00%

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum AmmError {
    InvalidFee = 1,
    InvalidPrice = 2,
    PoolExists = 3,
    PoolNotFound = 4,
    InvalidTokens = 5,
    Unauthorized = 6,
    DeadlineExceeded = 7,
    InsufficientLiquidity = 8,
    SlippageExceeded = 9,
}

/// Create a new liquidity pool
pub fn create_pool(
    env: Env,
    token_a: Address,
    token_b: Address,
    fee_tier: u32,
    sqrt_price_x96: u128,
) -> Result<BytesN<32>, AmmError> {
    // Validate fee tier
    if !is_valid_fee_tier(fee_tier) {
        return Err(AmmError::InvalidFee);
    }

    // Validate price range
    if sqrt_price_x96 < MIN_SQRT_RATIO || sqrt_price_x96 > MAX_SQRT_RATIO {
        return Err(AmmError::InvalidPrice);
    }

    // Ensure tokens are different
    if token_a == token_b {
        return Err(AmmError::InvalidTokens);
    }

    // Order tokens (token_a should be < token_b)
    let (token_0, token_1) = if token_a < token_b {
        (token_a, token_b)
    } else {
        (token_b, token_a)
    };

    // Generate pool ID
    let pool_id = generate_pool_id(&env, &token_0, &token_1, fee_tier);

    // Check if pool already exists
    if env.storage().persistent().has(&DataKey::Pool(pool_id.clone())) {
        return Err(AmmError::PoolExists);
    }

    // Create pool
    let pool = Pool {
        token_a: token_0,
        token_b: token_1,
        reserve_a: 0,
        reserve_b: 0,
        fee_tier,
        tick_spacing: get_tick_spacing(fee_tier),
        sqrt_price_x96,
        liquidity: 0,
        fee_growth_global_0_x128: 0,
        fee_growth_global_1_x128: 0,
    };

    // Store pool
    env.storage().persistent().set(&DataKey::Pool(pool_id.clone()), &pool);

    // Increment pool count
    let pool_count: u32 = env.storage().instance().get(&DataKey::PoolCount).unwrap_or(0);
    env.storage().instance().set(&DataKey::PoolCount, &(pool_count + 1));

    Ok(pool_id)
}

/// Get pool information
pub fn get_pool(env: Env, pool_id: BytesN<32>) -> Result<Pool, AmmError> {
    env.storage()
        .persistent()
        .get(&DataKey::Pool(pool_id))
        .ok_or(AmmError::PoolNotFound)
}

/// Get current price of a pool
pub fn get_pool_price(env: Env, pool_id: BytesN<32>) -> Result<u128, AmmError> {
    let pool = get_pool(env, pool_id)?;
    Ok(pool.sqrt_price_x96)
}

/// Update pool reserves and price
pub fn update_pool(env: Env, pool_id: BytesN<32>, pool: &Pool) {
    env.storage().persistent().set(&DataKey::Pool(pool_id), pool);
}

/// Calculate the price from sqrt price
pub fn sqrt_price_to_price(sqrt_price_x96: u128) -> u128 {
    math::price_from_sqrt_price(sqrt_price_x96)
}

/// Calculate sqrt price from price
pub fn price_to_sqrt_price(price: u128) -> u128 {
    math::sqrt_price_from_price(price)
}

/// Generate a unique pool ID
fn generate_pool_id(env: &Env, token_a: &Address, token_b: &Address, fee_tier: u32) -> BytesN<32> {
    let mut data = soroban_sdk::Bytes::new(env);
    data.append(&token_a.to_xdr(env));
    data.append(&token_b.to_xdr(env));
    data.extend_from_array(&fee_tier.to_be_bytes());
    
    env.crypto().keccak256(&data).into()
}

/// Check if fee tier is valid
fn is_valid_fee_tier(fee_tier: u32) -> bool {
    matches!(fee_tier, FEE_TIER_LOW | FEE_TIER_MEDIUM | FEE_TIER_HIGH)
}

/// Get tick spacing for fee tier
fn get_tick_spacing(fee_tier: u32) -> u32 {
    match fee_tier {
        FEE_TIER_LOW => 1,
        FEE_TIER_MEDIUM => 60,
        FEE_TIER_HIGH => 200,
        _ => 60, // default
    }
}

/// Calculate the tick from price
pub fn price_to_tick(sqrt_price_x96: u128) -> i32 {
    math::get_tick_at_sqrt_ratio(sqrt_price_x96)
}

/// Calculate the price from tick
pub fn tick_to_sqrt_price(tick: i32) -> u128 {
    math::get_sqrt_ratio_at_tick(tick)
}

/// Get the next initialized tick
pub fn get_next_tick(env: &Env, pool_id: &BytesN<32>, tick: i32, tick_spacing: i32, lte: bool) -> Option<i32> {
    use crate::tick_bitmap::TickBitmap;
    
    let (next_tick, initialized) = TickBitmap::next_initialized_tick_within_one_word(
        env, pool_id, tick, tick_spacing, lte
    );
    
    if initialized {
        Some(next_tick)
    } else {
        None
    }
}