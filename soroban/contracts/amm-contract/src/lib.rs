#![no_std]

use soroban_sdk::{contract, contractimpl, Address, Env, Symbol, Vec, i128};

mod liquidity;
mod swap;
mod fees;
mod positions;

#[contract]
pub struct AdvancedAmmContract;

#[contractimpl]
impl AdvancedAmmContract {
    // Funciones de liquidez
    pub fn create_pool(
        env: Env,
        token_a: Symbol,
        token_b: Symbol,
        fee_tier: i128,
        tick_spacing: i128,
    ) {
        liquidity::create_pool(&env, token_a, token_b, fee_tier, tick_spacing);
    }

    pub fn add_liquidity(
        env: Env,
        user: Address,
        token_a: Symbol,
        token_b: Symbol,
        amount_a: i128,
        amount_b: i128,
        tick_lower: i128,
        tick_upper: i128,
    ) {
        liquidity::add_liquidity(&env, user, token_a, token_b, amount_a, amount_b, tick_lower, tick_upper);
    }

    pub fn remove_liquidity(
        env: Env,
        user: Address,
        token_a: Symbol,
        token_b: Symbol,
        liquidity: i128,
    ) {
        liquidity::remove_liquidity(&env, user, token_a, token_b, liquidity);
    }

    // Funciones de swap
    pub fn swap(
        env: Env,
        user: Address,
        token_in: Symbol,
        token_out: Symbol,
        amount_in: i128,
        min_amount_out: i128,
    ) {
        swap::execute_swap(&env, user, token_in, token_out, amount_in, min_amount_out);
    }

    pub fn get_optimal_path(
        env: Env,
        token_in: Symbol,
        token_out: Symbol,
        amount_in: i128,
    ) -> Vec<Symbol> {
        swap::get_optimal_path(&env, token_in, token_out, amount_in)
    }

    // Funciones de fees
    pub fn collect_fees(
        env: Env,
        user: Address,
        token_a: Symbol,
        token_b: Symbol,
    ) {
        fees::collect_fees(&env, user, token_a, token_b);
    }

    pub fn get_accumulated_fees(
        env: Env,
        user: Address,
        token_a: Symbol,
        token_b: Symbol,
    ) -> (i128, i128) {
        fees::get_accumulated_fees(&env, user, token_a, token_b)
    }
} 