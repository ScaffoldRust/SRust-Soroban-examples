use crate::{GridLoadBalancingIncentives, GridLoadBalancingIncentivesClient};
use soroban_sdk::{testutils::Address as _, token, Address, Env};

pub struct TestContext {
    pub env: Env,
    pub contract_id: Address,
    pub client: GridLoadBalancingIncentivesClient<'static>,
    pub admin: Address,
    pub operator: Address,
    pub consumer: Address,
    pub token: Address,
}

pub fn setup() -> TestContext {
    let env = Env::default();
    env.mock_all_auths_allowing_non_root_auth();
    let admin = Address::generate(&env);
    let operator = Address::generate(&env);
    let consumer = Address::generate(&env);
    let token = env
        .register_stellar_asset_contract_v2(admin.clone())
        .address();
    let contract_id = env.register(GridLoadBalancingIncentives, ());
    let client = GridLoadBalancingIncentivesClient::new(&env, &contract_id);
    client.initialize(&admin, &token);
    TestContext {
        env,
        contract_id,
        client,
        admin,
        operator,
        consumer,
        token,
    }
}

pub fn mint(env: &Env, token_address: &Address, recipient: &Address, amount: i128) {
    token::StellarAssetClient::new(env, token_address).mint(recipient, &amount);
}

pub fn setup_event(context: &TestContext) -> u64 {
    context.client.register_grid_operator(&context.operator);
    context.client.start_event(&context.operator, &100, &5, &60)
}
