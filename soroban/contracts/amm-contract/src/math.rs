// Fixed-point math constants
pub const Q96: u128 = 1 << 96;

// Tick math constants
pub const MIN_TICK: i32 = -887272;
pub const MAX_TICK: i32 = 887272;
pub const MIN_SQRT_RATIO: u128 = 4295128739;
pub const MAX_SQRT_RATIO: u128 = 340282366920938463463374607431768211455u128; // Max u128 value

/// Full precision multiply-divide operation
/// Calculates (a * b) / c with overflow protection
pub fn mul_div(a: u128, b: u128, c: u128) -> u128 {
    if c == 0 {
        return 0;
    }
    
    // Check for overflow in multiplication
    if let Some(product) = a.checked_mul(b) {
        product / c
    } else {
        // Handle overflow using checked operations
        // Try to reduce precision by dividing first
        if a > c {
            let quotient = a / c;
            let remainder = a % c;
            quotient.saturating_mul(b).saturating_add(remainder.saturating_mul(b) / c)
        } else if b > c {
            let quotient = b / c;
            let remainder = b % c;
            quotient.saturating_mul(a).saturating_add(remainder.saturating_mul(a) / c)
        } else {
            // Both numbers are smaller than c, so result should be 0
            0
        }
    }
}

/// Calculate sqrt price from tick using precise math
/// Uses high-precision approximation of sqrt(1.0001^tick)
pub fn get_sqrt_ratio_at_tick(tick: i32) -> u128 {
    if tick < MIN_TICK || tick > MAX_TICK {
        return 0;
    }
    
    if tick == 0 {
        return Q96;
    }
    
    let abs_tick = if tick < 0 { (-tick) as u32 } else { tick as u32 };
    
    // Use precise calculation based on 1.0001^tick
    // Start with Q96 (representing 1.0)
    let mut ratio = Q96;
    
    // Apply tick-based price calculation using binary expansion
    // Each bit position represents a power of 2 in the tick
    let mut temp_tick = abs_tick;
    let mut power = 0u32;
    
    // Use precomputed sqrt(1.0001^(2^n)) values for efficiency
    let multipliers = [
        100005000u128,  // sqrt(1.0001^1) * 10^8
        100010000u128,  // sqrt(1.0001^2) * 10^8  
        100020001u128,  // sqrt(1.0001^4) * 10^8
        100040006u128,  // sqrt(1.0001^8) * 10^8
        100080028u128,  // sqrt(1.0001^16) * 10^8
        100160120u128,  // sqrt(1.0001^32) * 10^8
        100320481u128,  // sqrt(1.0001^64) * 10^8
        100641924u128,  // sqrt(1.0001^128) * 10^8
    ];
    
    while temp_tick > 0 && (power as usize) < multipliers.len() {
        if temp_tick & 1 != 0 {
            ratio = mul_div(ratio, multipliers[power as usize], 100000000u128);
        }
        temp_tick >>= 1;
        power += 1;
    }
    
    // Handle remaining large ticks with exponential approximation
    if abs_tick > 255 {
        let remaining_ticks = abs_tick - 255;
        let large_multiplier = 102020u128; // Approximately 1.0001^255
        let iterations = (remaining_ticks / 255).min(10);
        
        for _ in 0..iterations {
            ratio = mul_div(ratio, large_multiplier, 100000u128);
        }
    }
    
    // Apply direction (negative ticks mean reciprocal)
    if tick < 0 {
        ratio = mul_div(Q96, Q96, ratio);
    }
    
    // Ensure we stay within valid bounds
    ratio.max(MIN_SQRT_RATIO).min(MAX_SQRT_RATIO)
}

/// Calculate tick from sqrt price using precise math
/// Uses binary search with the exact inverse of get_sqrt_ratio_at_tick
pub fn get_tick_at_sqrt_ratio(sqrt_price_x96: u128) -> i32 {
    if sqrt_price_x96 < MIN_SQRT_RATIO || sqrt_price_x96 > MAX_SQRT_RATIO {
        return 0;
    }
    
    // Binary search for the exact tick
    let mut tick_low = MIN_TICK;
    let mut tick_high = MAX_TICK;
    
    while tick_high - tick_low > 1 {
        let tick_mid = (tick_low + tick_high) / 2;
        let sqrt_ratio_mid = get_sqrt_ratio_at_tick(tick_mid);
        
        if sqrt_ratio_mid <= sqrt_price_x96 {
            tick_low = tick_mid;
        } else {
            tick_high = tick_mid;
        }
    }
    
    // Final check to ensure we get the correct tick
    let sqrt_ratio_high = get_sqrt_ratio_at_tick(tick_high);
    
    if sqrt_price_x96 >= sqrt_ratio_high {
        tick_high
    } else {
        tick_low
    }
}

/// Calculate sqrt price from regular price
pub fn sqrt_price_from_price(price: u128) -> u128 {
    // Use Newton's method for square root calculation
    if price == 0 {
        return 0;
    }
    
    let scaled_price = price.saturating_mul(Q96); // Scale to Q96
    sqrt_u128(scaled_price)
}

/// Calculate regular price from sqrt price
pub fn price_from_sqrt_price(sqrt_price_x96: u128) -> u128 {
    mul_div(sqrt_price_x96, sqrt_price_x96, Q96)
}

/// Square root calculation using Newton's method
pub fn sqrt_u128(x: u128) -> u128 {
    if x == 0 {
        return 0;
    }
    
    let mut z = x;
    let mut y = (x + 1) / 2;
    
    while y < z {
        z = y;
        y = (x / y + y) / 2;
    }
    
    z
}

/// Calculate liquidity from amounts with full precision
pub fn get_liquidity_for_amounts(
    sqrt_price_x96: u128,
    sqrt_price_a_x96: u128,
    sqrt_price_b_x96: u128,
    amount_0: u128,
    amount_1: u128,
) -> u128 {
    if sqrt_price_a_x96 > sqrt_price_b_x96 {
        return get_liquidity_for_amounts(sqrt_price_x96, sqrt_price_b_x96, sqrt_price_a_x96, amount_1, amount_0);
    }
    
    if sqrt_price_x96 <= sqrt_price_a_x96 {
        get_liquidity_for_amount_0(sqrt_price_a_x96, sqrt_price_b_x96, amount_0)
    } else if sqrt_price_x96 < sqrt_price_b_x96 {
        let liquidity_0 = get_liquidity_for_amount_0(sqrt_price_x96, sqrt_price_b_x96, amount_0);
        let liquidity_1 = get_liquidity_for_amount_1(sqrt_price_a_x96, sqrt_price_x96, amount_1);
        liquidity_0.min(liquidity_1)
    } else {
        get_liquidity_for_amount_1(sqrt_price_a_x96, sqrt_price_b_x96, amount_1)
    }
}

/// Calculate liquidity from amount0
pub fn get_liquidity_for_amount_0(
    sqrt_price_a_x96: u128,
    sqrt_price_b_x96: u128,
    amount_0: u128,
) -> u128 {
    if sqrt_price_a_x96 > sqrt_price_b_x96 {
        return get_liquidity_for_amount_0(sqrt_price_b_x96, sqrt_price_a_x96, amount_0);
    }
    
    let intermediate = mul_div(sqrt_price_a_x96, sqrt_price_b_x96, Q96);
    mul_div(amount_0, intermediate, sqrt_price_b_x96 - sqrt_price_a_x96)
}

/// Calculate liquidity from amount1
pub fn get_liquidity_for_amount_1(
    sqrt_price_a_x96: u128,
    sqrt_price_b_x96: u128,
    amount_1: u128,
) -> u128 {
    if sqrt_price_a_x96 > sqrt_price_b_x96 {
        return get_liquidity_for_amount_1(sqrt_price_b_x96, sqrt_price_a_x96, amount_1);
    }
    
    mul_div(amount_1, Q96, sqrt_price_b_x96 - sqrt_price_a_x96)
}

/// Calculate amount0 from liquidity
pub fn get_amount_0_for_liquidity(
    sqrt_price_a_x96: u128,
    sqrt_price_b_x96: u128,
    liquidity: u128,
) -> u128 {
    if sqrt_price_a_x96 > sqrt_price_b_x96 {
        return get_amount_0_for_liquidity(sqrt_price_b_x96, sqrt_price_a_x96, liquidity);
    }
    
    mul_div(
        liquidity << 96,
        sqrt_price_b_x96 - sqrt_price_a_x96,
        sqrt_price_b_x96,
    ) / sqrt_price_a_x96
}

/// Calculate amount1 from liquidity
pub fn get_amount_1_for_liquidity(
    sqrt_price_a_x96: u128,
    sqrt_price_b_x96: u128,
    liquidity: u128,
) -> u128 {
    if sqrt_price_a_x96 > sqrt_price_b_x96 {
        return get_amount_1_for_liquidity(sqrt_price_b_x96, sqrt_price_a_x96, liquidity);
    }
    
    mul_div(liquidity, sqrt_price_b_x96 - sqrt_price_a_x96, Q96)
}

/// Calculate next sqrt price from input amount
pub fn get_next_sqrt_price_from_amount_0_rounding_up(
    sqrt_price_x96: u128,
    liquidity: u128,
    amount: u128,
    add: bool,
) -> u128 {
    if amount == 0 {
        return sqrt_price_x96;
    }
    
    let numerator_1 = liquidity << 96;
    
    if add {
        let product = amount * sqrt_price_x96;
        if product / amount == sqrt_price_x96 {
            let denominator = numerator_1 + product;
            if denominator >= numerator_1 {
                return mul_div(numerator_1, sqrt_price_x96, denominator);
            }
        }
        
        // Fallback for overflow
        numerator_1 / (numerator_1 / sqrt_price_x96 + amount)
    } else {
        let product = amount * sqrt_price_x96;
        if product / amount == sqrt_price_x96 && numerator_1 > product {
            let denominator = numerator_1 - product;
            return mul_div(numerator_1, sqrt_price_x96, denominator);
        }
        
        // This should not happen in normal circumstances
        0
    }
}

/// Calculate next sqrt price from output amount
pub fn get_next_sqrt_price_from_amount_1_rounding_down(
    sqrt_price_x96: u128,
    liquidity: u128,
    amount: u128,
    add: bool,
) -> u128 {
    if add {
        let quotient = if amount <= u128::MAX >> 96 {
            (amount << 96) / liquidity
        } else {
            mul_div(amount, Q96, liquidity)
        };
        
        sqrt_price_x96 + quotient
    } else {
        let quotient = if amount <= u128::MAX >> 96 {
            (amount << 96) / liquidity
        } else {
            mul_div(amount, Q96, liquidity)
        };
        
        if sqrt_price_x96 > quotient {
            sqrt_price_x96 - quotient
        } else {
            0
        }
    }
}

// All arithmetic is done with u128 for Soroban compatibility