// stablecoin-contract/src/burn.rs

use crate::collateral::deposit_collateral_internal; // Updated import
use crate::parameters::get_parameters;
use soroban_sdk::{contracttype, Address, Env, Symbol};

#[contracttype]
pub enum DataKey {
    StablecoinBalance(Address),
}

// Burn stablecoins and release collateral
pub fn burn(env: &Env, user: Address, amount: i128, asset: Symbol) {
    assert!(amount > 0, "Burn amount must be positive");
    user.require_auth();

    let key = DataKey::StablecoinBalance(user.clone());
    let current_balance = env.storage().instance().get::<_, i128>(&key).unwrap_or(0);
    assert!(current_balance >= amount, "Insufficient stablecoin balance");

    // Reduce user's stablecoin balance
    env.storage()
        .instance()
        .set(&key, &(current_balance - amount));

    // Calculate collateral to return based on min collateral ratio
    let params = get_parameters(env);
    let collateral_to_return = (amount as u128)
        .checked_mul(params.min_collateral_ratio as u128)
        .unwrap()
        .checked_div(10_000u128)
        .unwrap() as i128;

    // Return collateral to user without additional auth
    deposit_collateral_internal(env, user, asset, collateral_to_return);
}
