use crate::{
    pool::{get_pool, price_to_tick, AmmError},
    position::{get_position, calculate_position_token_amounts},
    DataKey, Pool, Position, PriceTick,
};
use soroban_sdk::{token, Address, BytesN, Env, Vec, xdr::ToXdr};

/// Collect fees from a position
pub fn collect_fees(env: Env, position_id: BytesN<32>) -> Result<(u128, u128), AmmError> {
    let mut position = get_position(env.clone(), position_id.clone())?;
    position.owner.require_auth();

    let pool = get_pool(env.clone(), position.pool_id.clone())?;

    // Calculate fees owed
    let (fees_0, fees_1) = calculate_fees_owed(env.clone(), &pool, &mut position);

    if fees_0 == 0 && fees_1 == 0 {
        return Ok((0, 0));
    }

    // Transfer fees to position owner
    if fees_0 > 0 {
        let token_0_client = token::Client::new(&env, &pool.token_a);
        token_0_client.transfer(
            &env.current_contract_address(),
            &position.owner,
            &(fees_0 as i128),
        );
    }

    if fees_1 > 0 {
        let token_1_client = token::Client::new(&env, &pool.token_b);
        token_1_client.transfer(
            &env.current_contract_address(),
            &position.owner,
            &(fees_1 as i128),
        );
    }

    // Record fee collection in history
    record_fee_collection(&env, &position_id, fees_0, fees_1);

    // Reset tokens owed
    position.tokens_owed_0 = 0;
    position.tokens_owed_1 = 0;

    // Update position
    env.storage()
        .persistent()
        .set(&DataKey::Position(position_id), &position);

    Ok((fees_0, fees_1))
}

/// Calculate fees owed to a position
pub fn calculate_fees_owed(env: Env, pool: &Pool, position: &mut Position) -> (u128, u128) {
    // Get fee growth inside the position's range
    let (fee_growth_inside_0_x128, fee_growth_inside_1_x128) = get_fee_growth_inside(
        env,
        &position.pool_id,
        position.price_lower,
        position.price_upper,
        pool,
    );

    // Calculate fees accrued since last collection
    let fees_0 = calculate_fee_amount(
        fee_growth_inside_0_x128,
        position.fee_growth_inside_0_last_x128,
        position.liquidity,
    ) + position.tokens_owed_0;

    let fees_1 = calculate_fee_amount(
        fee_growth_inside_1_x128,
        position.fee_growth_inside_1_last_x128,
        position.liquidity,
    ) + position.tokens_owed_1;

    // Update position's last fee growth
    position.fee_growth_inside_0_last_x128 = fee_growth_inside_0_x128;
    position.fee_growth_inside_1_last_x128 = fee_growth_inside_1_x128;

    (fees_0, fees_1)
}

/// Get fee growth inside a price range
fn get_fee_growth_inside(
    env: Env,
    pool_id: &BytesN<32>,
    price_lower: u128,
    price_upper: u128,
    pool: &Pool,
) -> (u128, u128) {
    let tick_lower = price_to_tick(price_lower);
    let tick_upper = price_to_tick(price_upper);
    let current_tick = price_to_tick(pool.sqrt_price_x96);

    // Get tick info for lower and upper bounds
    let tick_lower_info = get_tick_info(env.clone(), pool_id, tick_lower);
    let tick_upper_info = get_tick_info(env.clone(), pool_id, tick_upper);

    let (fee_growth_below_0_x128, fee_growth_below_1_x128) = if current_tick >= tick_lower {
        (
            tick_lower_info.fee_growth_outside_0_x128,
            tick_lower_info.fee_growth_outside_1_x128,
        )
    } else {
        (
            pool.fee_growth_global_0_x128
                .wrapping_sub(tick_lower_info.fee_growth_outside_0_x128),
            pool.fee_growth_global_1_x128
                .wrapping_sub(tick_lower_info.fee_growth_outside_1_x128),
        )
    };

    let (fee_growth_above_0_x128, fee_growth_above_1_x128) = if current_tick < tick_upper {
        (
            tick_upper_info.fee_growth_outside_0_x128,
            tick_upper_info.fee_growth_outside_1_x128,
        )
    } else {
        (
            pool.fee_growth_global_0_x128
                .wrapping_sub(tick_upper_info.fee_growth_outside_0_x128),
            pool.fee_growth_global_1_x128
                .wrapping_sub(tick_upper_info.fee_growth_outside_1_x128),
        )
    };

    let fee_growth_inside_0_x128 = pool
        .fee_growth_global_0_x128
        .wrapping_sub(fee_growth_below_0_x128)
        .wrapping_sub(fee_growth_above_0_x128);

    let fee_growth_inside_1_x128 = pool
        .fee_growth_global_1_x128
        .wrapping_sub(fee_growth_below_1_x128)
        .wrapping_sub(fee_growth_above_1_x128);

    (fee_growth_inside_0_x128, fee_growth_inside_1_x128)
}

/// Get tick information, returning default if not initialized
fn get_tick_info(env: Env, pool_id: &BytesN<32>, tick: i32) -> PriceTick {
    let tick_key = DataKey::Tick(pool_id.clone(), tick);
    env.storage()
        .persistent()
        .get(&tick_key)
        .unwrap_or(PriceTick {
            price: 0,
            liquidity_net: 0,
            liquidity_gross: 0,
            fee_growth_outside_0_x128: 0,
            fee_growth_outside_1_x128: 0,
            initialized: false,
        })
}

/// Calculate fee amount from fee growth
pub fn calculate_fee_amount(
    fee_growth_inside_x128: u128,
    fee_growth_inside_last_x128: u128,
    liquidity: u128,
) -> u128 {
    let fee_growth_delta = fee_growth_inside_x128.wrapping_sub(fee_growth_inside_last_x128);
    fee_growth_delta.saturating_mul(liquidity) >> 64
}

/// Update fee growth outside when crossing a tick
pub fn update_fee_growth_outside(
    env: &Env,
    pool_id: &BytesN<32>,
    tick: i32,
    fee_growth_global_0_x128: u128,
    fee_growth_global_1_x128: u128,
) {
    let tick_key = DataKey::Tick(pool_id.clone(), tick);

    if let Some(mut tick_info) = env
        .storage()
        .persistent()
        .get::<DataKey, PriceTick>(&tick_key)
    {
        tick_info.fee_growth_outside_0_x128 = fee_growth_global_0_x128;
        tick_info.fee_growth_outside_1_x128 = fee_growth_global_1_x128;
        env.storage().persistent().set(&tick_key, &tick_info);
    }
}

/// Calculate protocol fees (portion of trading fees that go to protocol)
pub fn calculate_protocol_fees(
    total_fees_0: u128,
    total_fees_1: u128,
    protocol_fee_rate: u32, // in basis points
) -> (u128, u128) {
    let protocol_fees_0 = (total_fees_0 * protocol_fee_rate as u128) / 10000;
    let protocol_fees_1 = (total_fees_1 * protocol_fee_rate as u128) / 10000;

    (protocol_fees_0, protocol_fees_1)
}

/// Collect protocol fees (admin only)
pub fn collect_protocol_fees(env: Env, pool_id: BytesN<32>) -> Result<(u128, u128), AmmError> {
    // Check admin authorization
    let admin: Address = env
        .storage()
        .instance()
        .get(&DataKey::Admin)
        .ok_or(AmmError::Unauthorized)?;
    admin.require_auth();

    let mut pool = get_pool(env.clone(), pool_id.clone())?;

    // Get protocol fee accumulation from pool state
    let protocol_fees_key_0 = DataKey::Tick(pool_id.clone(), -1); // Reuse tick storage for protocol fees
    let protocol_fees_key_1 = DataKey::Tick(pool_id.clone(), -2);
    
    let protocol_fees_0: u128 = env.storage().persistent().get(&protocol_fees_key_0).unwrap_or(0);
    let protocol_fees_1: u128 = env.storage().persistent().get(&protocol_fees_key_1).unwrap_or(0);

    if protocol_fees_0 > 0 {
        let token_0_client = token::Client::new(&env, &pool.token_a);
        token_0_client.transfer(
            &env.current_contract_address(),
            &admin,
            &(protocol_fees_0 as i128),
        );
        
        // Reset protocol fees
        env.storage().persistent().remove(&protocol_fees_key_0);
        
        // Update pool reserves
        pool.reserve_a -= protocol_fees_0 as i128;
    }

    if protocol_fees_1 > 0 {
        let token_1_client = token::Client::new(&env, &pool.token_b);
        token_1_client.transfer(
            &env.current_contract_address(),
            &admin,
            &(protocol_fees_1 as i128),
        );
        
        // Reset protocol fees
        env.storage().persistent().remove(&protocol_fees_key_1);
        
        // Update pool reserves
        pool.reserve_b -= protocol_fees_1 as i128;
    }

    // Update pool state
    env.storage().persistent().set(&DataKey::Pool(pool_id), &pool);

    Ok((protocol_fees_0, protocol_fees_1))
}

/// Get total fees earned by a position (including uncollected)
pub fn get_position_fees(env: Env, position_id: BytesN<32>) -> Result<(u128, u128), AmmError> {
    let mut position = get_position(env.clone(), position_id.clone())?;
    let pool = get_pool(env.clone(), position.pool_id.clone())?;

    Ok(calculate_fees_owed(env, &pool, &mut position))
}

/// Calculate APR for a position based on historical fee collection
pub fn calculate_position_apr(
    env: Env,
    position_id: BytesN<32>,
    time_period_seconds: u64,
) -> Result<(u128, u128), AmmError> {
    let position = get_position(env.clone(), position_id.clone())?;
    let pool = get_pool(env.clone(), position.pool_id)?;

    // Get historical fee data
    let fee_history_key = DataKey::Position(generate_fee_history_key(&env, &position_id));
    let fee_history: Vec<(u64, u128, u128)> = env.storage().persistent()
        .get(&fee_history_key)
        .unwrap_or(Vec::new(&env));

    let current_timestamp = env.ledger().timestamp();
    let cutoff_time = current_timestamp.saturating_sub(time_period_seconds);

    // Calculate fees earned in the time period
    let mut total_fees_0 = 0u128;
    let mut total_fees_1 = 0u128;
    
    for entry in fee_history.iter() {
        let (timestamp, fees_0, fees_1) = (entry.0, entry.1, entry.2);
        if timestamp >= cutoff_time {
            total_fees_0 += fees_0;
            total_fees_1 += fees_1;
        }
    }

    // Calculate position value for APR calculation
    let (amount_0, amount_1) = calculate_position_token_amounts(
        pool.sqrt_price_x96,
        position.price_lower,
        position.price_upper,
        position.liquidity,
    );

    // Calculate APR as (fees / position_value) * (seconds_per_year / time_period)
    let seconds_per_year = 365 * 24 * 3600;
    let time_multiplier = seconds_per_year / time_period_seconds.max(1);

    let apr_0 = if amount_0 > 0 {
        (total_fees_0 * time_multiplier as u128 * 10000) / (amount_0 as u128) // Basis points
    } else {
        0
    };

    let apr_1 = if amount_1 > 0 {
        (total_fees_1 * time_multiplier as u128 * 10000) / (amount_1 as u128) // Basis points
    } else {
        0
    };

    Ok((apr_0, apr_1))
}

/// Generate key for fee history storage
fn generate_fee_history_key(env: &Env, position_id: &BytesN<32>) -> BytesN<32> {
    let mut data = soroban_sdk::Bytes::new(env);
    data.append(&position_id.clone().to_xdr(env));
    data.extend_from_array(b"fee_history");
    
    env.crypto().keccak256(&data).into()
}

/// Record fee collection in history for APR calculation
pub fn record_fee_collection(
    env: &Env,
    position_id: &BytesN<32>,
    fees_0: u128,
    fees_1: u128,
) {
    let fee_history_key = DataKey::Position(generate_fee_history_key(env, position_id));
    let mut fee_history: Vec<(u64, u128, u128)> = env.storage().persistent()
        .get(&fee_history_key)
        .unwrap_or(Vec::new(env));

    let current_timestamp = env.ledger().timestamp();
    fee_history.push_back((current_timestamp, fees_0, fees_1));

    // Keep only last 100 entries to prevent unbounded growth
    while fee_history.len() > 100 {
        fee_history.pop_front();
    }

    env.storage().persistent().set(&fee_history_key, &fee_history);
}
