// stablecoin-contract/src/collateral.rs
use soroban_sdk::{contracttype, Address, Env, Symbol};


#[derive(Clone, Debug)]
#[contracttype]
pub struct CollateralAsset {
    pub asset_type: Symbol, // E.g., "USDC", "XLM"
    pub ratio: u32,         // In basis points, e.g., 15000 = 150%
    pub reserve: i128,      // Total collateral reserve
}

// Key used to store user collateral mappings
#[contracttype]
pub enum DataKey {
    CollateralReserve(Symbol),              // Total reserves per asset
    UserCollateral(Address, Symbol),        // Collateral per user per asset
}

// Adds collateral to the protocol and user balance
pub fn deposit_collateral(env: &Env, user: Address, asset: Symbol, amount: i128) {
    assert!(amount > 0, "Amount must be positive");
    user.require_auth();

    // Increase user's collateral
    let key = DataKey::UserCollateral(user.clone(), asset.clone());
    let current = env.storage().instance().get::<_, i128>(&key).unwrap_or(0);
    env.storage().instance().set(&key, &(current + amount));

    // Increase total reserve
    let reserve_key = DataKey::CollateralReserve(asset.clone());
    let reserve = env.storage().instance().get::<_, i128>(&reserve_key).unwrap_or(0);
    env.storage().instance().set(&reserve_key, &(reserve + amount));
}

// Withdraws collateral from user balance and total reserve
pub fn withdraw_collateral(env: &Env, user: Address, asset: Symbol, amount: i128) {
    assert!(amount > 0, "Amount must be positive");
    user.require_auth();

    let key = DataKey::UserCollateral(user.clone(), asset.clone());
    let current = env.storage().instance().get::<_, i128>(&key).unwrap_or(0);
    assert!(current >= amount, "Insufficient collateral");

    env.storage().instance().set(&key, &(current - amount));

    let reserve_key = DataKey::CollateralReserve(asset.clone());
    let reserve = env.storage().instance().get::<_, i128>(&reserve_key).unwrap_or(0);
    env.storage().instance().set(&reserve_key, &(reserve - amount));
}

// Returns a user’s collateral balance
pub fn get_user_collateral(env: &Env, user: Address, asset: Symbol) -> i128 {
    env.storage().instance()
        .get::<_, i128>(&DataKey::UserCollateral(user, asset))
        .unwrap_or(0)
}

// Returns total collateral held by the protocol for an asset
pub fn get_total_reserve(env: &Env, asset: Symbol) -> i128 {
    env.storage().instance()
        .get::<_, i128>(&DataKey::CollateralReserve(asset))
        .unwrap_or(0)
}
