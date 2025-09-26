use soroban_sdk::{Address, BytesN, Env, Vec, token, xdr::ToXdr};
use crate::{
    DataKey, Pool, SwapParams, SwapPath, PriceTick,
    pool::{AmmError, get_pool, update_pool, price_to_tick, tick_to_sqrt_price, get_next_tick},
    math::{mul_div, Q96},
};

/// Execute a token swap through optimal path
pub fn execute_swap(env: Env, params: SwapParams) -> Result<i128, AmmError> {
    // Check deadline
    if env.ledger().timestamp() > params.deadline {
        return Err(AmmError::DeadlineExceeded);
    }

    params.recipient.require_auth();

    // Get optimal swap path
    let swap_path = get_optimal_swap_path(
        env.clone(),
        params.token_in.clone(),
        params.token_out.clone(),
        params.amount_in,
    )?;

    // Execute multi-hop swap
    let min_amount_out = params.min_amount_out;
    let amount_out = execute_multi_hop_swap(env, swap_path, params)?;

    // Check slippage
    if amount_out < min_amount_out {
        return Err(AmmError::SlippageExceeded);
    }

    Ok(amount_out)
}

/// Get optimal swap path between two tokens
/// Implements sophisticated routing algorithm with multi-hop support
pub fn get_optimal_swap_path(
    env: Env,
    token_in: Address,
    token_out: Address,
    amount_in: i128,
) -> Result<SwapPath, AmmError> {
    // First try direct pools
    let mut best_path = SwapPath {
        pools: Vec::new(&env),
        tokens: Vec::new(&env),
    };
    let mut best_output = 0i128;

    // Check direct pools with different fee tiers
    for fee_tier in [5u32, 30u32, 100u32] {
        if let Some(pool_id) = find_pool(&env, &token_in, &token_out, fee_tier) {
            if let Ok(estimated_output) = estimate_swap_output(env.clone(), pool_id.clone(), token_in.clone(), amount_in) {
                if estimated_output > best_output {
                    best_output = estimated_output;
                    best_path.pools = Vec::from_array(&env, [pool_id]);
                    best_path.tokens = Vec::from_array(&env, [token_in.clone(), token_out.clone()]);
                }
            }
        }
    }

    // Try multi-hop paths through common intermediate tokens
    let intermediate_tokens = get_common_intermediate_tokens(&env);
    
    for intermediate in intermediate_tokens.iter() {
        if intermediate == token_in || intermediate == token_out {
            continue;
        }
        
        // Try path: token_in -> intermediate -> token_out
        if let Ok(path_output) = estimate_multi_hop_output(
            env.clone(),
            token_in.clone(),
            token_out.clone(),
            intermediate.clone(),
            amount_in,
        ) {
            if path_output > best_output {
                best_output = path_output;
                
                // Find the actual pools for this path
                if let (Some(pool_1), Some(pool_2)) = (
                    find_best_pool(&env, &token_in, &intermediate),
                    find_best_pool(&env, &intermediate, &token_out),
                ) {
                    best_path.pools = Vec::from_array(&env, [pool_1, pool_2]);
                    best_path.tokens = Vec::from_array(&env, [
                        token_in.clone(),
                        intermediate.clone(),
                        token_out.clone(),
                    ]);
                }
            }
        }
    }

    if best_path.pools.is_empty() {
        return Err(AmmError::PoolNotFound);
    }

    Ok(best_path)
}

/// Check if we should explore multi-hop paths
fn should_check_multi_hop(_env: &Env, _token_in: &Address, _token_out: &Address) -> bool {
    // Always check multi-hop for better price discovery
    // In production, you might want to limit this based on gas costs
    true
}

/// Get common intermediate tokens for multi-hop routing
/// Returns a curated list of high-liquidity tokens commonly used for routing
fn get_common_intermediate_tokens(env: &Env) -> Vec<Address> {
    let tokens = Vec::new(env);
    
    // Get stored intermediate tokens from contract storage
    let intermediate_tokens_key = DataKey::Admin; // Reuse admin key for simplicity in storage
    if let Some(stored_tokens) = env.storage().instance().get::<DataKey, Vec<Address>>(&intermediate_tokens_key) {
        return stored_tokens;
    }
    
    // Default intermediate tokens (would be set by admin in production)
    // For testing, we'll return an empty list but the infrastructure is there
    tokens
}

/// Estimate output for a multi-hop path
fn estimate_multi_hop_output(
    env: Env,
    token_in: Address,
    _token_out: Address,
    intermediate: Address,
    amount_in: i128,
) -> Result<i128, AmmError> {
    // Estimate first hop: token_in -> intermediate
    let pool_1 = find_best_pool(&env, &token_in, &intermediate)
        .ok_or(AmmError::PoolNotFound)?;
    let intermediate_amount = estimate_swap_output(env.clone(), pool_1, token_in, amount_in)?;
    
    // Estimate second hop: intermediate -> _token_out
    let pool_2 = find_best_pool(&env, &intermediate, &_token_out)
        .ok_or(AmmError::PoolNotFound)?;
    let final_amount = estimate_swap_output(env, pool_2, intermediate, intermediate_amount)?;
    
    Ok(final_amount)
}

/// Find the best pool between two tokens (highest liquidity)
fn find_best_pool(env: &Env, token_a: &Address, token_b: &Address) -> Option<BytesN<32>> {
    let mut best_pool = None;
    let mut best_liquidity = 0u128;
    
    for fee_tier in [5u32, 30u32, 100u32] {
        if let Some(pool_id) = find_pool(env, token_a, token_b, fee_tier) {
            if let Ok(pool) = get_pool(env.clone(), pool_id.clone()) {
                if pool.liquidity > best_liquidity {
                    best_liquidity = pool.liquidity;
                    best_pool = Some(pool_id);
                }
            }
        }
    }
    
    best_pool
}

/// Execute multi-hop swap through multiple pools
fn execute_multi_hop_swap(env: Env, swap_path: SwapPath, params: SwapParams) -> Result<i128, AmmError> {
    if swap_path.pools.is_empty() {
        return Err(AmmError::PoolNotFound);
    }

    let mut current_amount = params.amount_in;
    let mut current_token = params.token_in.clone();

    // Transfer initial tokens from user
    let token_in_client = token::Client::new(&env, &current_token);
    token_in_client.transfer(&params.recipient, &env.current_contract_address(), &current_amount);

    // Execute swaps through each pool in the path
    for i in 0..swap_path.pools.len() {
        let pool_id = swap_path.pools.get(i).unwrap();
        let next_token = swap_path.tokens.get(i + 1).unwrap();
        
        current_amount = execute_single_swap(
            env.clone(),
            pool_id,
            current_token.clone(),
            next_token.clone(),
            current_amount,
        )?;
        
        current_token = next_token;
    }

    // Transfer final tokens to recipient
    let _token_out_client = token::Client::new(&env, &current_token);
    _token_out_client.transfer(&env.current_contract_address(), &params.recipient, &current_amount);

    Ok(current_amount)
}

/// Execute a single swap within a pool
fn execute_single_swap(
    env: Env,
    pool_id: BytesN<32>,
    token_in: Address,
    _token_out: Address,
    amount_in: i128,
) -> Result<i128, AmmError> {
    let mut pool = get_pool(env.clone(), pool_id.clone())?;
    
    // Determine swap direction
    let zero_for_one = token_in == pool.token_a;
    
    // Calculate swap result
    let swap_result = compute_swap_step(
        env.clone(),
        &mut pool,
        pool_id.clone(),
        amount_in,
        zero_for_one,
    );

    // Update pool state
    if zero_for_one {
        pool.reserve_a += amount_in;
        pool.reserve_b -= swap_result.amount_out;
    } else {
        pool.reserve_b += amount_in;
        pool.reserve_a -= swap_result.amount_out;
    }
    
    pool.sqrt_price_x96 = swap_result.sqrt_price_x96;
    pool.liquidity = swap_result.liquidity;

    // Update fee growth
    if zero_for_one {
        if pool.liquidity > 0 {
            pool.fee_growth_global_0_x128 = pool.fee_growth_global_0_x128
                .saturating_add(swap_result.fee_amount.saturating_mul(1u128 << 64) / pool.liquidity);
        }
    } else {
        if pool.liquidity > 0 {
            pool.fee_growth_global_1_x128 = pool.fee_growth_global_1_x128
                .saturating_add(swap_result.fee_amount.saturating_mul(1u128 << 64) / pool.liquidity);
        }
    }

    update_pool(env, pool_id, &pool);

    Ok(swap_result.amount_out)
}

/// Compute swap step result
fn compute_swap_step(
    env: Env,
    pool: &mut Pool,
    pool_id: BytesN<32>,
    amount_specified: i128,
    zero_for_one: bool,
) -> SwapStepResult {
    let mut sqrt_price_x96 = pool.sqrt_price_x96;
    let mut liquidity = pool.liquidity;
    let mut amount_remaining = amount_specified;
    let mut amount_out = 0i128;
    let mut fee_amount = 0u128;

    // Get current tick
    let mut tick = price_to_tick(sqrt_price_x96);

    while amount_remaining > 0 {
        // Find next tick
        let next_tick = get_next_tick(&env, &pool_id, tick, pool.tick_spacing as i32, !zero_for_one)
            .unwrap_or(if zero_for_one { tick - pool.tick_spacing as i32 } else { tick + pool.tick_spacing as i32 });
        
        let sqrt_price_next_x96 = tick_to_sqrt_price(next_tick);

        // Calculate swap within current tick range
        let step_result = compute_swap_within_tick(
            sqrt_price_x96,
            sqrt_price_next_x96,
            liquidity,
            amount_remaining,
            pool.fee_tier,
            zero_for_one,
        );

        sqrt_price_x96 = step_result.sqrt_price_next_x96;
        amount_remaining -= step_result.amount_in;
        amount_out += step_result.amount_out;
        fee_amount += step_result.fee_amount;

        // Cross tick if we reached it
        if sqrt_price_x96 == sqrt_price_next_x96 {
            let tick_key = DataKey::Tick(pool_id.clone(), next_tick);
            if let Some(tick_info) = env.storage().persistent().get::<DataKey, PriceTick>(&tick_key) {
                if zero_for_one {
                    liquidity = (liquidity as i128 - tick_info.liquidity_net) as u128;
                } else {
                    liquidity = (liquidity as i128 + tick_info.liquidity_net) as u128;
                }
            }
            tick = next_tick;
        }

        // Break if we can't continue
        if amount_remaining <= 0 || liquidity == 0 {
            break;
        }
    }

    SwapStepResult {
        sqrt_price_x96,
        liquidity,
        amount_out,
        fee_amount,
    }
}

/// Compute swap within a single tick range
fn compute_swap_within_tick(
    sqrt_price_current_x96: u128,
    sqrt_price_target_x96: u128,
    liquidity: u128,
    amount_remaining: i128,
    fee_tier: u32,
    zero_for_one: bool,
) -> SwapWithinTickResult {
    // Calculate the maximum amount we can swap in this tick range
    let amount_max = if zero_for_one {
        calculate_amount_0_delta(sqrt_price_target_x96, sqrt_price_current_x96, liquidity)
    } else {
        calculate_amount_1_delta(sqrt_price_current_x96, sqrt_price_target_x96, liquidity)
    };

    let amount_in = if amount_remaining >= amount_max as i128 {
        amount_max as i128
    } else {
        amount_remaining
    };

    // Calculate fee
    let fee_amount = (amount_in as u128 * fee_tier as u128) / 10000;
    let amount_in_after_fee = amount_in - fee_amount as i128;

    // Calculate new price after swap
    let sqrt_price_next_x96 = if amount_in == amount_max as i128 {
        sqrt_price_target_x96
    } else {
        calculate_new_price_after_swap(
            sqrt_price_current_x96,
            liquidity,
            amount_in_after_fee,
            zero_for_one,
        )
    };

    // Calculate amount out
    let amount_out = if zero_for_one {
        calculate_amount_1_delta(sqrt_price_next_x96, sqrt_price_current_x96, liquidity)
    } else {
        calculate_amount_0_delta(sqrt_price_current_x96, sqrt_price_next_x96, liquidity)
    };

    SwapWithinTickResult {
        sqrt_price_next_x96,
        amount_in,
        amount_out: amount_out as i128,
        fee_amount,
    }
}

/// Calculate new price after swap
fn calculate_new_price_after_swap(
    sqrt_price_x96: u128,
    liquidity: u128,
    amount: i128,
    zero_for_one: bool,
) -> u128 {
    if zero_for_one {
        // Selling token0 for token1
        let numerator = (liquidity as u128) << 96;
        let denominator = liquidity as u128 + (amount as u128 * sqrt_price_x96 as u128 >> 96);
        (numerator / denominator) as u128
    } else {
        // Selling token1 for token0
        sqrt_price_x96 + ((amount as u128) << 96) / liquidity
    }
}

/// Calculate amount0 delta
fn calculate_amount_0_delta(sqrt_price_a_x96: u128, sqrt_price_b_x96: u128, liquidity: u128) -> u128 {
    if sqrt_price_a_x96 > sqrt_price_b_x96 {
        return calculate_amount_0_delta(sqrt_price_b_x96, sqrt_price_a_x96, liquidity);
    }
    
    let numerator = (liquidity << 96) * (sqrt_price_b_x96 - sqrt_price_a_x96);
    let denominator = sqrt_price_b_x96 * sqrt_price_a_x96;
    numerator / denominator
}

/// Calculate amount1 delta
fn calculate_amount_1_delta(sqrt_price_a_x96: u128, sqrt_price_b_x96: u128, liquidity: u128) -> u128 {
    if sqrt_price_a_x96 > sqrt_price_b_x96 {
        return calculate_amount_1_delta(sqrt_price_b_x96, sqrt_price_a_x96, liquidity);
    }
    
    (liquidity * (sqrt_price_b_x96 - sqrt_price_a_x96)) >> 96
}

/// Estimate swap output for pathfinding using concentrated liquidity math
fn estimate_swap_output(env: Env, pool_id: BytesN<32>, token_in: Address, amount_in: i128) -> Result<i128, AmmError> {
    let pool = get_pool(env.clone(), pool_id.clone())?;
    
    if pool.liquidity == 0 {
        return Ok(0);
    }
    
    let zero_for_one = token_in == pool.token_a;
    let sqrt_price_x96 = pool.sqrt_price_x96;
    
    // Calculate fee
    let fee_amount = (amount_in as u128 * pool.fee_tier as u128) / 10000;
    let amount_in_after_fee = amount_in - fee_amount as i128;
    
    // Use concentrated liquidity formula
    let amount_out = if zero_for_one {
        // Selling token0 for token1
        let sqrt_price_next = calculate_sqrt_price_from_amount_0(
            sqrt_price_x96,
            pool.liquidity,
            amount_in_after_fee,
            true
        );
        calculate_amount_1_delta(sqrt_price_next, sqrt_price_x96, pool.liquidity) as i128
    } else {
        // Selling token1 for token0
        let sqrt_price_next = calculate_sqrt_price_from_amount_1(
            sqrt_price_x96,
            pool.liquidity,
            amount_in_after_fee,
            true
        );
        calculate_amount_0_delta(sqrt_price_x96, sqrt_price_next, pool.liquidity) as i128
    };
    
    Ok(amount_out)
}

/// Calculate new sqrt price when adding amount0
fn calculate_sqrt_price_from_amount_0(
    sqrt_price_x96: u128,
    liquidity: u128,
    amount: i128,
    add: bool,
) -> u128 {
    if amount == 0 {
        return sqrt_price_x96;
    }
    
    let numerator_1 = liquidity << 96;
    
    if add {
        let product = (amount as u128).saturating_mul(sqrt_price_x96);
        let denominator = numerator_1.saturating_add(product);
        if denominator >= numerator_1 {
            return mul_div(numerator_1, sqrt_price_x96, denominator);
        }
        
        // Fallback calculation
        numerator_1 / (numerator_1 / sqrt_price_x96 + amount as u128)
    } else {
        let product = (amount as u128).saturating_mul(sqrt_price_x96);
        if numerator_1 > product {
            let denominator = numerator_1 - product;
            return mul_div(numerator_1, sqrt_price_x96, denominator);
        }
        0
    }
}

/// Calculate new sqrt price when adding amount1
fn calculate_sqrt_price_from_amount_1(
    sqrt_price_x96: u128,
    liquidity: u128,
    amount: i128,
    add: bool,
) -> u128 {
    if add {
        let quotient = if amount <= (u128::MAX >> 96) as i128 {
            ((amount as u128) << 96) / liquidity
        } else {
            mul_div(amount as u128, Q96, liquidity)
        };
        
        sqrt_price_x96.saturating_add(quotient)
    } else {
        let quotient = if amount <= (u128::MAX >> 96) as i128 {
            ((amount as u128) << 96) / liquidity
        } else {
            mul_div(amount as u128, Q96, liquidity)
        };
        
        sqrt_price_x96.saturating_sub(quotient)
    }
}

/// Find a pool by tokens and fee tier
fn find_pool(env: &Env, token_a: &Address, token_b: &Address, fee_tier: u32) -> Option<BytesN<32>> {
    // Order tokens
    let (token_0, token_1) = if token_a < token_b {
        (token_a.clone(), token_b.clone())
    } else {
        (token_b.clone(), token_a.clone())
    };

    // Generate pool ID
    let mut data = soroban_sdk::Bytes::new(env);
    data.append(&token_0.to_xdr(env));
    data.append(&token_1.to_xdr(env));
    data.extend_from_array(&fee_tier.to_be_bytes());
    
    let pool_id: BytesN<32> = env.crypto().keccak256(&data).into();

    if env.storage().persistent().has(&DataKey::Pool(pool_id.clone())) {
        Some(pool_id)
    } else {
        None
    }
}

// Helper structs for swap calculations
struct SwapStepResult {
    sqrt_price_x96: u128,
    liquidity: u128,
    amount_out: i128,
    fee_amount: u128,
}

struct SwapWithinTickResult {
    sqrt_price_next_x96: u128,
    amount_in: i128,
    amount_out: i128,
    fee_amount: u128,
}
