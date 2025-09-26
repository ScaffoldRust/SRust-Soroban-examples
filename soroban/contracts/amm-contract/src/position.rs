use soroban_sdk::{Address, BytesN, Env, Vec, xdr::ToXdr};
use crate::{
    DataKey, Pool, Position,
    pool::{AmmError, get_pool, price_to_tick},
    fees::calculate_fees_owed,
    math::{mul_div, sqrt_u128, Q96},
};

/// Get position information
pub fn get_position(env: Env, position_id: BytesN<32>) -> Result<Position, AmmError> {
    env.storage()
        .persistent()
        .get(&DataKey::Position(position_id))
        .ok_or(AmmError::PoolNotFound)
}

/// Get all positions for a user using efficient indexing
pub fn get_user_positions(env: Env, user: Address) -> Vec<BytesN<32>> {
    let mut positions = Vec::new(&env);
    
    // Use user-specific index for efficient lookup
    let user_positions_key = DataKey::Position(generate_user_positions_key(&env, &user));
    if let Some(user_position_ids) = env.storage().persistent().get::<DataKey, Vec<BytesN<32>>>(&user_positions_key) {
        // Verify positions still exist and belong to user
        for position_id in user_position_ids.iter() {
            if let Some(position) = env.storage().persistent().get::<DataKey, Position>(&DataKey::Position(position_id.clone())) {
                if position.owner == user {
                    positions.push_back(position_id);
                }
            }
        }
    }

    positions
}

/// Generate a unique key for storing user's position list
fn generate_user_positions_key(env: &Env, user: &Address) -> BytesN<32> {
    let mut data = soroban_sdk::Bytes::new(env);
    data.append(&user.to_xdr(env));
    data.extend_from_array(b"user_positions");
    
    env.crypto().keccak256(&data).into()
}

/// Set position bounds (for concentrated liquidity)
pub fn set_position_bounds(
    env: Env,
    position_id: BytesN<32>,
    price_lower: u128,
    price_upper: u128,
) -> Result<Position, AmmError> {
    let mut position = get_position(env.clone(), position_id.clone())?;
    position.owner.require_auth();

    // Validate price bounds
    if price_lower >= price_upper {
        return Err(AmmError::InvalidPrice);
    }

    // Update position bounds
    position.price_lower = price_lower;
    position.price_upper = price_upper;

    // Reset fee tracking since bounds changed
    position.fee_growth_inside_0_last_x128 = 0;
    position.fee_growth_inside_1_last_x128 = 0;

    // Update storage
    env.storage().persistent().set(&DataKey::Position(position_id), &position);

    Ok(position)
}

/// Get position value in terms of underlying tokens
pub fn get_position_value(env: Env, position_id: BytesN<32>) -> Result<(i128, i128), AmmError> {
    let position = get_position(env.clone(), position_id)?;
    let pool = get_pool(env, position.pool_id)?;

    Ok(calculate_position_token_amounts(
        pool.sqrt_price_x96,
        position.price_lower,
        position.price_upper,
        position.liquidity,
    ))
}

/// Get position information with current fees
pub fn get_position_info(env: Env, position_id: BytesN<32>) -> Result<PositionInfo, AmmError> {
    let mut position = get_position(env.clone(), position_id)?;
    let pool = get_pool(env.clone(), position.pool_id.clone())?;

    let (amount_0, amount_1) = calculate_position_token_amounts(
        pool.sqrt_price_x96,
        position.price_lower,
        position.price_upper,
        position.liquidity,
    );

    let (fees_0, fees_1) = calculate_fees_owed(env, &pool, &mut position);

    Ok(PositionInfo {
        position: position.clone(),
        token_amount_0: amount_0,
        token_amount_1: amount_1,
        fees_owed_0: fees_0,
        fees_owed_1: fees_1,
        in_range: is_position_in_range(&pool, &position),
    })
}

/// Check if position is currently in range (earning fees)
pub fn is_position_in_range(pool: &Pool, position: &Position) -> bool {
    let current_tick = price_to_tick(pool.sqrt_price_x96);
    let tick_lower = price_to_tick(position.price_lower);
    let tick_upper = price_to_tick(position.price_upper);

    current_tick >= tick_lower && current_tick < tick_upper
}

/// Calculate position's share of pool liquidity
pub fn get_position_share(env: Env, position_id: BytesN<32>) -> Result<u128, AmmError> {
    let position = get_position(env.clone(), position_id)?;
    let pool = get_pool(env, position.pool_id)?;

    if pool.liquidity == 0 {
        return Ok(0);
    }

    // Calculate share as basis points (10000 = 100%)
    Ok((position.liquidity * 10000) / pool.liquidity)
}

/// Calculate position's impermanent loss with precise math
pub fn calculate_impermanent_loss(
    env: Env,
    position_id: BytesN<32>,
    initial_price: u128,
) -> Result<i128, AmmError> {
    let position = get_position(env.clone(), position_id)?;
    let pool = get_pool(env, position.pool_id)?;

    // Get current price from sqrt_price
    let current_price = mul_div(pool.sqrt_price_x96, pool.sqrt_price_x96, Q96);
    
    if initial_price == 0 {
        return Ok(0);
    }
    
    // Calculate price ratio with high precision
    let price_ratio = mul_div(current_price, Q96, initial_price);
    
    // Calculate impermanent loss using the exact formula:
    // IL = 2 * sqrt(price_ratio) / (1 + price_ratio) - 1
    
    // Calculate sqrt(price_ratio) using Newton's method
    let sqrt_price_ratio = sqrt_u128(price_ratio);
    
    // Calculate 2 * sqrt(price_ratio)
    let numerator = sqrt_price_ratio * 2;
    
    // Calculate (1 + price_ratio)
    let denominator = Q96 + price_ratio;
    
    // Calculate the ratio
    let il_ratio = mul_div(numerator, Q96, denominator);
    
    // Subtract 1 to get the impermanent loss
    let il = (il_ratio as i128) - (Q96 as i128);
    
    Ok(il)
}

/// Update position after liquidity change
pub fn update_position_liquidity(
    env: Env,
    position_id: BytesN<32>,
    liquidity_delta: i128,
) -> Result<Position, AmmError> {
    let mut position = get_position(env.clone(), position_id.clone())?;
    
    if liquidity_delta < 0 && position.liquidity < (-liquidity_delta) as u128 {
        return Err(AmmError::InsufficientLiquidity);
    }

    if liquidity_delta > 0 {
        position.liquidity += liquidity_delta as u128;
    } else {
        position.liquidity -= (-liquidity_delta) as u128;
    }

    env.storage().persistent().set(&DataKey::Position(position_id), &position);
    Ok(position)
}

/// Calculate token amounts for a position at current price
pub fn calculate_position_token_amounts(
    sqrt_price_x96: u128,
    sqrt_price_a_x96: u128,
    sqrt_price_b_x96: u128,
    liquidity: u128,
) -> (i128, i128) {
    if sqrt_price_a_x96 > sqrt_price_b_x96 {
        let (amount_1, amount_0) = calculate_position_token_amounts(
            sqrt_price_x96, sqrt_price_b_x96, sqrt_price_a_x96, liquidity
        );
        return (amount_0, amount_1);
    }

    let amount_0 = if sqrt_price_x96 <= sqrt_price_a_x96 {
        calculate_amount_0(sqrt_price_a_x96, sqrt_price_b_x96, liquidity)
    } else if sqrt_price_x96 < sqrt_price_b_x96 {
        calculate_amount_0(sqrt_price_x96, sqrt_price_b_x96, liquidity)
    } else {
        0
    };

    let amount_1 = if sqrt_price_x96 <= sqrt_price_a_x96 {
        0
    } else if sqrt_price_x96 < sqrt_price_b_x96 {
        calculate_amount_1(sqrt_price_a_x96, sqrt_price_x96, liquidity)
    } else {
        calculate_amount_1(sqrt_price_a_x96, sqrt_price_b_x96, liquidity)
    };

    (amount_0 as i128, amount_1 as i128)
}

/// Calculate amount_0 from liquidity and price range
fn calculate_amount_0(sqrt_price_a_x96: u128, sqrt_price_b_x96: u128, liquidity: u128) -> u128 {
    if sqrt_price_a_x96 > sqrt_price_b_x96 {
        return calculate_amount_0(sqrt_price_b_x96, sqrt_price_a_x96, liquidity);
    }
    
    let numerator = mul_div(liquidity, Q96, 1);
    let denominator_factor = sqrt_price_b_x96.saturating_sub(sqrt_price_a_x96);
    let denominator = mul_div(sqrt_price_a_x96, sqrt_price_b_x96, Q96);
    
    if denominator == 0 {
        return 0;
    }
    
    mul_div(numerator, denominator_factor, denominator)
}

/// Calculate amount_1 from liquidity and price range
fn calculate_amount_1(sqrt_price_a_x96: u128, sqrt_price_b_x96: u128, liquidity: u128) -> u128 {
    if sqrt_price_a_x96 > sqrt_price_b_x96 {
        return calculate_amount_1(sqrt_price_b_x96, sqrt_price_a_x96, liquidity);
    }
    
    let price_diff = sqrt_price_b_x96.saturating_sub(sqrt_price_a_x96);
    mul_div(liquidity, price_diff, Q96)
}

/// Generate position ID from index (for scanning)
fn generate_position_id_from_index(env: &Env, index: u32) -> BytesN<32> {
    let mut data = soroban_sdk::Bytes::new(env);
    data.extend_from_array(&index.to_be_bytes());
    data.extend_from_array(b"position");
    
    env.crypto().keccak256(&data).into()
}

/// Extended position information
#[derive(Clone, Debug)]
pub struct PositionInfo {
    pub position: Position,
    pub token_amount_0: i128,
    pub token_amount_1: i128,
    pub fees_owed_0: u128,
    pub fees_owed_1: u128,
    pub in_range: bool,
}

/// Position statistics for analytics
#[derive(Clone, Debug)]
pub struct PositionStats {
    pub total_fees_earned_0: u128,
    pub total_fees_earned_1: u128,
    pub days_active: u64,
    pub average_liquidity: u128,
    pub impermanent_loss: i128,
}

/// Get comprehensive position statistics
pub fn get_position_stats(
    env: Env,
    position_id: BytesN<32>,
    initial_price: u128,
    creation_timestamp: u64,
) -> Result<PositionStats, AmmError> {
    let position_info = get_position_info(env.clone(), position_id.clone())?;
    let current_timestamp = env.ledger().timestamp();
    
    let days_active = (current_timestamp - creation_timestamp) / (24 * 3600);
    let impermanent_loss = calculate_impermanent_loss(env, position_id, initial_price)?;

    Ok(PositionStats {
        total_fees_earned_0: position_info.fees_owed_0,
        total_fees_earned_1: position_info.fees_owed_1,
        days_active,
        average_liquidity: position_info.position.liquidity, // Simplified
        impermanent_loss,
    })
}