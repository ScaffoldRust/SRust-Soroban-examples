use soroban_sdk::{Env, Symbol, Address, i128, Map};

pub fn collect_fees(
    env: &Env,
    user: Address,
    token_a: Symbol,
    token_b: Symbol,
) {
    // Obtener los swaps del usuario
    let swaps: Map<Address, Vec<(Symbol, Symbol, i128, i128)>> = env.storage()
        .get(&Symbol::new(env, "swaps"))
        .unwrap();
    
    let user_swaps = swaps.get(user.clone()).unwrap();
    
    // Calcular fees acumulados
    let mut fees_a = 0;
    let mut fees_b = 0;
    
    for (token_in, token_out, amount_in, _) in user_swaps.iter() {
        if *token_in == token_a {
            fees_a += calculate_fee(*amount_in);
        } else if *token_in == token_b {
            fees_b += calculate_fee(*amount_in);
        }
    }
    
    // Actualizar el balance de fees del usuario
    let mut user_fees: Map<Address, (i128, i128)> = env.storage()
        .get(&Symbol::new(env, "user_fees"))
        .unwrap_or(Map::new(env));
    
    user_fees.set(user, (fees_a, fees_b));
    env.storage().set(&Symbol::new(env, "user_fees"), &user_fees);
    
    // Limpiar los swaps del usuario
    let mut swaps = swaps;
    swaps.set(user, Vec::new(env));
    env.storage().set(&Symbol::new(env, "swaps"), &swaps);
}

pub fn get_accumulated_fees(
    env: &Env,
    user: Address,
    token_a: Symbol,
    token_b: Symbol,
) -> (i128, i128) {
    let user_fees: Map<Address, (i128, i128)> = env.storage()
        .get(&Symbol::new(env, "user_fees"))
        .unwrap();
    
    user_fees.get(user).unwrap_or((0, 0))
}

fn calculate_fee(amount: i128) -> i128 {
    // Calcular fee del 0.3%
    (amount * 3) / 1000
} 