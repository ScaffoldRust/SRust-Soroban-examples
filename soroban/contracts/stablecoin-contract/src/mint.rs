// stablecoin-contract/src/mint.rs

use crate::collateral::get_user_collateral;
use crate::parameters::get_parameters;
use soroban_sdk::{contracttype, Address, Env, Symbol};

#[contracttype]
pub enum DataKey {
    StablecoinBalance(Address),
}

// Mint stablecoins against collateral
pub fn mint(env: &Env, user: Address, amount: i128, asset: Symbol) {
    assert!(amount > 0, "Mint amount must be positive");
    user.require_auth();

    // Check user collateral for the asset
    let user_collateral = get_user_collateral(env, user.clone(), asset.clone());
    let params = get_parameters(env);

    // Calculate max mintable based on collateral ratio
    // max_mint = collateral * (1 / min_collateral_ratio)
    let max_mint = (user_collateral as u128)
        .checked_mul(10_000u128)
        .unwrap()
        .checked_div(params.min_collateral_ratio as u128)
        .unwrap() as i128;

    assert!(
        amount <= max_mint,
        "Mint amount exceeds collateral-backed limit"
    );
    assert!(
        amount <= params.max_mint_amount,
        "Mint amount exceeds protocol max"
    );

    // Increase user's stablecoin balance
    let key = DataKey::StablecoinBalance(user.clone());
    let current_balance = env.storage().instance().get::<_, i128>(&key).unwrap_or(0);
    env.storage()
        .instance()
        .set(&key, &(current_balance + amount));
}
