use crate::types::*;
use soroban_sdk::Env;

pub fn calculate_fees(env: Env, amount: i128, is_urgent: bool) -> i128 {
    let fee_structure: FeeStructure =
        env.storage()
            .instance()
            .get(&DataKey::Fees)
            .unwrap_or(FeeStructure {
                base_fee: 100,
                percentage: 100,         // 1%
                urgency_multiplier: 150, // 1.5x for urgent
            });

    let mut fee = fee_structure.base_fee;
    let percentage_fee = (amount * fee_structure.percentage as i128) / 10_000; // Convert basis points
    fee += percentage_fee;

    if is_urgent {
        fee = (fee * fee_structure.urgency_multiplier as i128) / 100;
    }

    fee
}
