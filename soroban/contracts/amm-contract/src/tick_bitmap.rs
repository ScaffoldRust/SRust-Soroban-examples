use soroban_sdk::{BytesN, Env};
use crate::DataKey;

/// Tick bitmap for efficient tick traversal
/// Each bit represents whether a tick is initialized
pub struct TickBitmap;

impl TickBitmap {
    /// Flip the initialized state of a tick
    pub fn flip_tick(env: &Env, pool_id: &BytesN<32>, tick: i32, tick_spacing: i32) {
        let (word_pos, bit_pos) = Self::position(tick / tick_spacing);
        let key = DataKey::TickBitmap(pool_id.clone(), word_pos);
        
        let mut word: u128 = env.storage().persistent().get(&key).unwrap_or(0);
        word ^= 1u128 << bit_pos;
        
        if word == 0 {
            env.storage().persistent().remove(&key);
        } else {
            env.storage().persistent().set(&key, &word);
        }
    }
    
    /// Find the next initialized tick to the left (lte = true) or right (lte = false)
    pub fn next_initialized_tick_within_one_word(
        env: &Env,
        pool_id: &BytesN<32>,
        tick: i32,
        tick_spacing: i32,
        lte: bool,
    ) -> (i32, bool) {
        let compressed = tick / tick_spacing;
        
        if lte {
            let (word_pos, bit_pos) = Self::position(compressed);
            let key = DataKey::TickBitmap(pool_id.clone(), word_pos);
            let word: u128 = env.storage().persistent().get(&key).unwrap_or(0);
            
            // Create mask for bits at or to the right of the current bit
            let mask = (1u128 << bit_pos) - 1 + (1u128 << bit_pos);
            let masked = word & mask;
            
            let initialized = masked != 0;
            let next = if initialized {
                (compressed - (bit_pos - Self::most_significant_bit(masked)) as i32) * tick_spacing
            } else {
                (compressed - bit_pos as i32) * tick_spacing
            };
            
            (next, initialized)
        } else {
            let (word_pos, bit_pos) = Self::position(compressed + 1);
            let key = DataKey::TickBitmap(pool_id.clone(), word_pos);
            let word: u128 = env.storage().persistent().get(&key).unwrap_or(0);
            
            // Create mask for bits to the left of the current bit
            let mask = !((1u128 << bit_pos) - 1);
            let masked = word & mask;
            
            let initialized = masked != 0;
            let next = if initialized {
                (compressed + 1 + (Self::least_significant_bit(masked) - bit_pos) as i32) * tick_spacing
            } else {
                (compressed + 1 + (255 - bit_pos) as i32) * tick_spacing
            };
            
            (next, initialized)
        }
    }
    
    /// Get the position in the bitmap
    fn position(tick: i32) -> (i32, u8) {
        let word_pos = tick >> 7; // Use 128-bit words instead of 256-bit
        let bit_pos = (tick % 128) as u8;
        (word_pos, bit_pos)
    }
    
    /// Find the most significant bit
    fn most_significant_bit(x: u128) -> u8 {
        if x == 0 {
            return 0;
        }
        
        let mut msb = 0u8;
        let mut value = x;
        
        if value >= 1u128 << 64 {
            value >>= 64;
            msb += 64;
        }
        if value >= 1u128 << 32 {
            value >>= 32;
            msb += 32;
        }
        if value >= 1u128 << 16 {
            value >>= 16;
            msb += 16;
        }
        if value >= 1u128 << 8 {
            value >>= 8;
            msb += 8;
        }
        if value >= 1u128 << 4 {
            value >>= 4;
            msb += 4;
        }
        if value >= 1u128 << 2 {
            value >>= 2;
            msb += 2;
        }
        if value >= 1u128 << 1 {
            msb += 1;
        }
        
        msb
    }
    
    /// Find the least significant bit
    fn least_significant_bit(x: u128) -> u8 {
        if x == 0 {
            return 0;
        }
        
        let mut lsb = 0u8;
        let mut value = x;
        
        if value & ((1u128 << 64) - 1) == 0 {
            value >>= 64;
            lsb += 64;
        }
        if value & ((1u128 << 32) - 1) == 0 {
            value >>= 32;
            lsb += 32;
        }
        if value & ((1u128 << 16) - 1) == 0 {
            value >>= 16;
            lsb += 16;
        }
        if value & ((1u128 << 8) - 1) == 0 {
            value >>= 8;
            lsb += 8;
        }
        if value & ((1u128 << 4) - 1) == 0 {
            value >>= 4;
            lsb += 4;
        }
        if value & ((1u128 << 2) - 1) == 0 {
            value >>= 2;
            lsb += 2;
        }
        if value & 1 == 0 {
            lsb += 1;
        }
        
        lsb
    }
}

// Using u128 for bitmap operations in Soroban