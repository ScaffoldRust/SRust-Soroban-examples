// stablecoin-contract/src/collateral.rs
use soroban_sdk::{contracttype, Address, Env, Symbol};

#[derive(Clone, Debug)]
#[contracttype]
pub struct CollateralAsset {
    pub asset_type: Symbol, // E.g., "USDC", "XLM"
    pub ratio: u32,         // In basis points, e.g., 15000 = 150%
    pub reserve: i128,      // Total collateral reserve
}

#[contracttype]
pub enum DataKey {
    CollateralReserve(Symbol),
    UserCollateral(Address, Symbol),
}

// Internal function to deposit collateral without auth
pub fn deposit_collateral_internal(env: &Env, user: Address, asset: Symbol, amount: i128) {
    assert!(amount > 0, "Amount must be positive");

    // Increase user's collateral
    let key = DataKey::UserCollateral(user.clone(), asset.clone());
    let current = env.storage().instance().get::<_, i128>(&key).unwrap_or(0);
    env.storage().instance().set(&key, &(current + amount));

    // Increase total reserve
    let reserve_key = DataKey::CollateralReserve(asset.clone());
    let reserve = env
        .storage()
        .instance()
        .get::<_, i128>(&reserve_key)
        .unwrap_or(0);
    env.storage()
        .instance()
        .set(&reserve_key, &(reserve + amount));
}

// Public function to deposit collateral with auth
pub fn deposit_collateral(env: &Env, user: Address, asset: Symbol, amount: i128) {
    user.require_auth();
    deposit_collateral_internal(env, user, asset, amount);
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
    let reserve = env
        .storage()
        .instance()
        .get::<_, i128>(&reserve_key)
        .unwrap_or(0);
    env.storage()
        .instance()
        .set(&reserve_key, &(reserve - amount));
}

// Returns a user’s collateral balance
pub fn get_user_collateral(env: &Env, user: Address, asset: Symbol) -> i128 {
    env.storage()
        .instance()
        .get::<_, i128>(&DataKey::UserCollateral(user, asset))
        .unwrap_or(0)
}

// Returns total collateral held by the protocol for an asset
pub fn get_total_reserve(env: &Env, asset: Symbol) -> i128 {
    env.storage()
        .instance()
        .get::<_, i128>(&DataKey::CollateralReserve(asset))
        .unwrap_or(0)
}
