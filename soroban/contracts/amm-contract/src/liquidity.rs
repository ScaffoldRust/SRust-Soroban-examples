use soroban_sdk::{Address, BytesN, Env, token, xdr::ToXdr};
use crate::{
    DataKey, Position, PriceTick, math, tick_bitmap::TickBitmap,
    pool::{AmmError, get_pool, update_pool, price_to_tick, tick_to_sqrt_price}
};

/// Add liquidity to a pool within specified price bounds
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
) -> Result<(BytesN<32>, u128, i128, i128), AmmError> {
    // Check deadline
    if env.ledger().timestamp() > deadline {
        return Err(AmmError::DeadlineExceeded);
    }

    recipient.require_auth();

    let mut pool = get_pool(env.clone(), pool_id.clone())?;
    
    // Validate price bounds
    if price_lower >= price_upper {
        return Err(AmmError::InvalidPrice);
    }

    // Convert prices to ticks
    let tick_lower = price_to_tick(price_lower);
    let tick_upper = price_to_tick(price_upper);

    // Calculate liquidity amount
    let liquidity = math::get_liquidity_for_amounts(
        pool.sqrt_price_x96,
        price_lower,
        price_upper,
        amount_a_desired as u128,
        amount_b_desired as u128,
    );

    if liquidity == 0 {
        return Err(AmmError::InsufficientLiquidity);
    }

    // Calculate actual amounts needed
    let amount_0 = if pool.sqrt_price_x96 <= price_lower {
        math::get_amount_0_for_liquidity(price_lower, price_upper, liquidity)
    } else if pool.sqrt_price_x96 < price_upper {
        math::get_amount_0_for_liquidity(pool.sqrt_price_x96, price_upper, liquidity)
    } else {
        0
    };

    let amount_1 = if pool.sqrt_price_x96 <= price_lower {
        0
    } else if pool.sqrt_price_x96 < price_upper {
        math::get_amount_1_for_liquidity(price_lower, pool.sqrt_price_x96, liquidity)
    } else {
        math::get_amount_1_for_liquidity(price_lower, price_upper, liquidity)
    };

    let (amount_0, amount_1) = (amount_0 as i128, amount_1 as i128);

    // Check slippage
    if amount_0 < amount_a_min || amount_1 < amount_b_min {
        return Err(AmmError::SlippageExceeded);
    }

    // Transfer tokens from user
    let token_0_client = token::Client::new(&env, &pool.token_a);
    let token_1_client = token::Client::new(&env, &pool.token_b);
    
    token_0_client.transfer(&recipient, &env.current_contract_address(), &amount_0);
    token_1_client.transfer(&recipient, &env.current_contract_address(), &amount_1);

    // Update pool reserves
    pool.reserve_a += amount_0;
    pool.reserve_b += amount_1;
    pool.liquidity += liquidity;

    // Update ticks and bitmap
    update_tick(&env, &pool_id, tick_lower, liquidity as i128, pool.tick_spacing);
    update_tick(&env, &pool_id, tick_upper, -(liquidity as i128), pool.tick_spacing);

    // Create position
    let position_id = create_position(
        &env,
        recipient.clone(),
        pool_id.clone(),
        liquidity,
        price_lower,
        price_upper,
    );

    // Update pool
    update_pool(env, pool_id, &pool);

    Ok((position_id, liquidity, amount_0, amount_1))
}

/// Remove liquidity from a position
pub fn remove_liquidity(
    env: Env,
    position_id: BytesN<32>,
    liquidity: u128,
    amount_0_min: i128,
    amount_1_min: i128,
    deadline: u64,
) -> Result<(i128, i128), AmmError> {
    // Check deadline
    if env.ledger().timestamp() > deadline {
        return Err(AmmError::DeadlineExceeded);
    }

    let mut position = get_position(env.clone(), position_id.clone())?;
    position.owner.require_auth();

    if position.liquidity < liquidity {
        return Err(AmmError::InsufficientLiquidity);
    }

    let mut pool = get_pool(env.clone(), position.pool_id.clone())?;

    // Calculate amounts to return
    let amount_0 = if pool.sqrt_price_x96 <= position.price_lower {
        math::get_amount_0_for_liquidity(position.price_lower, position.price_upper, liquidity)
    } else if pool.sqrt_price_x96 < position.price_upper {
        math::get_amount_0_for_liquidity(pool.sqrt_price_x96, position.price_upper, liquidity)
    } else {
        0
    };

    let amount_1 = if pool.sqrt_price_x96 <= position.price_lower {
        0
    } else if pool.sqrt_price_x96 < position.price_upper {
        math::get_amount_1_for_liquidity(position.price_lower, pool.sqrt_price_x96, liquidity)
    } else {
        math::get_amount_1_for_liquidity(position.price_lower, position.price_upper, liquidity)
    };

    let (amount_0, amount_1) = (amount_0 as i128, amount_1 as i128);

    // Check slippage
    if amount_0 < amount_0_min || amount_1 < amount_1_min {
        return Err(AmmError::SlippageExceeded);
    }

    // Update position
    position.liquidity -= liquidity;
    
    // Update pool
    pool.reserve_a -= amount_0;
    pool.reserve_b -= amount_1;
    pool.liquidity -= liquidity;

    // Update ticks and bitmap
    let tick_lower = price_to_tick(position.price_lower);
    let tick_upper = price_to_tick(position.price_upper);
    update_tick(&env, &position.pool_id, tick_lower, -(liquidity as i128), pool.tick_spacing);
    update_tick(&env, &position.pool_id, tick_upper, liquidity as i128, pool.tick_spacing);

    // Transfer tokens back to user
    let token_0_client = token::Client::new(&env, &pool.token_a);
    let token_1_client = token::Client::new(&env, &pool.token_b);
    
    token_0_client.transfer(&env.current_contract_address(), &position.owner, &amount_0);
    token_1_client.transfer(&env.current_contract_address(), &position.owner, &amount_1);

    // Update storage
    if position.liquidity == 0 {
        env.storage().persistent().remove(&DataKey::Position(position_id));
    } else {
        env.storage().persistent().set(&DataKey::Position(position_id), &position);
    }
    
    update_pool(env, position.pool_id, &pool);

    Ok((amount_0, amount_1))
}

/// Create a new position
fn create_position(
    env: &Env,
    owner: Address,
    pool_id: BytesN<32>,
    liquidity: u128,
    price_lower: u128,
    price_upper: u128,
) -> BytesN<32> {
    let position_count: u32 = env.storage().instance().get(&DataKey::PositionCount).unwrap_or(0);
    let position_id = generate_position_id(env, &owner, position_count);

    let position = Position {
        owner,
        pool_id,
        liquidity,
        price_lower,
        price_upper,
        fee_growth_inside_0_last_x128: 0,
        fee_growth_inside_1_last_x128: 0,
        tokens_owed_0: 0,
        tokens_owed_1: 0,
    };

    env.storage().persistent().set(&DataKey::Position(position_id.clone()), &position);
    env.storage().instance().set(&DataKey::PositionCount, &(position_count + 1));

    position_id
}

/// Get position information
pub fn get_position(env: Env, position_id: BytesN<32>) -> Result<Position, AmmError> {
    env.storage()
        .persistent()
        .get(&DataKey::Position(position_id))
        .ok_or(AmmError::PoolNotFound)
}

/// Update a tick's liquidity and bitmap
fn update_tick(env: &Env, pool_id: &BytesN<32>, tick: i32, liquidity_delta: i128, tick_spacing: u32) {
    let tick_key = DataKey::Tick(pool_id.clone(), tick);
    
    let mut tick_info = env.storage()
        .persistent()
        .get(&tick_key)
        .unwrap_or(PriceTick {
            price: tick_to_sqrt_price(tick),
            liquidity_net: 0,
            liquidity_gross: 0,
            fee_growth_outside_0_x128: 0,
            fee_growth_outside_1_x128: 0,
            initialized: false,
        });

    let liquidity_gross_before = tick_info.liquidity_gross;
    
    tick_info.liquidity_net += liquidity_delta;
    tick_info.liquidity_gross = if liquidity_delta > 0 {
        tick_info.liquidity_gross + (liquidity_delta as u128)
    } else {
        tick_info.liquidity_gross - ((-liquidity_delta) as u128)
    };
    
    let liquidity_gross_after = tick_info.liquidity_gross;
    tick_info.initialized = liquidity_gross_after > 0;

    // Update bitmap if initialization state changed
    if (liquidity_gross_before == 0) != (liquidity_gross_after == 0) {
        TickBitmap::flip_tick(env, pool_id, tick, tick_spacing as i32);
    }

    if tick_info.liquidity_gross == 0 {
        env.storage().persistent().remove(&tick_key);
    } else {
        env.storage().persistent().set(&tick_key, &tick_info);
    }
}

// All math functions are now in the math module

/// Generate a unique position ID
fn generate_position_id(env: &Env, owner: &Address, nonce: u32) -> BytesN<32> {
    let mut data = soroban_sdk::Bytes::new(env);
    data.append(&owner.to_xdr(env));
    data.extend_from_array(&nonce.to_be_bytes());
    data.extend_from_array(&env.ledger().timestamp().to_be_bytes());
    
    env.crypto().keccak256(&data).into()
}