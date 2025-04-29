use soroban_sdk::{Env, Symbol, Address, i128, Map};

pub struct Pool {
    pub token_a: Symbol,
    pub token_b: Symbol,
    pub fee_tier: i128,
    pub tick_spacing: i128,
    pub liquidity: i128,
    pub sqrt_price: i128,
    pub tick: i128,
}

pub fn create_pool(
    env: &Env,
    token_a: Symbol,
    token_b: Symbol,
    fee_tier: i128,
    tick_spacing: i128,
) {
    let pool = Pool {
        token_a,
        token_b,
        fee_tier,
        tick_spacing,
        liquidity: 0,
        sqrt_price: 0,
        tick: 0,
    };
    
    // Almacenar el pool en el storage del contrato
    let mut pools: Map<Symbol, Pool> = env.storage().get(&Symbol::new(env, "pools")).unwrap_or(Map::new(env));
    pools.set(token_a, pool);
    env.storage().set(&Symbol::new(env, "pools"), &pools);
}

pub fn add_liquidity(
    env: &Env,
    user: Address,
    token_a: Symbol,
    token_b: Symbol,
    amount_a: i128,
    amount_b: i128,
    tick_lower: i128,
    tick_upper: i128,
) {
    // Obtener el pool existente
    let mut pools: Map<Symbol, Pool> = env.storage().get(&Symbol::new(env, "pools")).unwrap();
    let mut pool = pools.get(token_a).unwrap();
    
    // Actualizar la liquidez del pool
    pool.liquidity += amount_a + amount_b;
    
    // Actualizar el pool en el storage
    pools.set(token_a, pool);
    env.storage().set(&Symbol::new(env, "pools"), &pools);
    
    // Registrar la posición del usuario
    let mut positions: Map<Address, (i128, i128, i128, i128)> = env.storage()
        .get(&Symbol::new(env, "positions"))
        .unwrap_or(Map::new(env));
    
    positions.set(user, (amount_a, amount_b, tick_lower, tick_upper));
    env.storage().set(&Symbol::new(env, "positions"), &positions);
}

pub fn remove_liquidity(
    env: &Env,
    user: Address,
    token_a: Symbol,
    token_b: Symbol,
    liquidity: i128,
) {
    // Obtener la posición del usuario
    let positions: Map<Address, (i128, i128, i128, i128)> = env.storage()
        .get(&Symbol::new(env, "positions"))
        .unwrap();
    
    let (amount_a, amount_b, _, _) = positions.get(user).unwrap();
    
    // Verificar que el usuario tiene suficiente liquidez
    assert!(amount_a + amount_b >= liquidity, "Insufficient liquidity");
    
    // Actualizar el pool
    let mut pools: Map<Symbol, Pool> = env.storage().get(&Symbol::new(env, "pools")).unwrap();
    let mut pool = pools.get(token_a).unwrap();
    pool.liquidity -= liquidity;
    pools.set(token_a, pool);
    env.storage().set(&Symbol::new(env, "pools"), &pools);
    
    // Actualizar la posición del usuario
    let mut positions = positions;
    positions.set(user, (amount_a - liquidity, amount_b - liquidity, 0, 0));
    env.storage().set(&Symbol::new(env, "positions"), &positions);
} 