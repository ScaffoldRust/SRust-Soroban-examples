use soroban_sdk::{Env, Address, BytesN, String, testutils::Address as _};
use crate::{TelemedicinePaymentGateway, TelemedicinePaymentGatewayClient};

pub struct TestContext {
    pub env: Env,
    pub client: TelemedicinePaymentGatewayClient<'static>,
}

pub fn setup_default(fee: u32, min_dur: u64, max_dur: u64) -> TestContext {
    let env = Env::default();
    let id = env.register(TelemedicinePaymentGateway, ());
    let client = TelemedicinePaymentGatewayClient::new(&env, &id);
    let admin = Address::generate(&env);
    client.initialize(&admin, &fee, &min_dur, &max_dur);
    TestContext { env, client }
}

pub fn register_provider(ctx: &TestContext, rate: i128) -> Address {
    let provider = Address::generate(&ctx.env);
    let currency = Address::generate(&ctx.env);
    let name = String::from_str(&ctx.env, "Dr Test");
    let spec = String::from_str(&ctx.env, "General");
    ctx.client.register_provider(&provider, &name, &spec, &rate, &currency);
    provider
}

pub fn new_session_id(env: &Env, seed: u64) -> BytesN<32> {
    let mut data = [0u8;32];
    data[0..8].copy_from_slice(&seed.to_le_bytes());
    BytesN::from_array(env, &data)
}

pub fn consent_hash(env: &Env, seed: u64) -> BytesN<32> { new_session_id(env, seed + 10) }
