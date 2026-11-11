use soroban_sdk::{symbol_short, testutils::Address as _, Address, Env, String, Vec};

use crate::{MedicalDonationEscrow, MedicalDonationEscrowClient};

pub const DONATION_ID: u64 = 1;
pub const AMOUNT: i128 = 1_000;

pub struct TestContext {
    pub env: Env,
    pub contract_id: Address,
    pub donor: Address,
    pub recipient: Address,
}

impl TestContext {
    pub fn new(milestone_count: u32) -> Self {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(MedicalDonationEscrow, ());
        let donor = Address::generate(&env);
        let recipient = Address::generate(&env);
        let client = MedicalDonationEscrowClient::new(&env, &contract_id);
        let mut milestones = Vec::new(&env);
        for index in 0..milestone_count {
            milestones.push_back(String::from_str(
                &env,
                if index == 0 { "ordered" } else { "delivered" },
            ));
        }
        client.initialize(
            &DONATION_ID,
            &donor,
            &recipient,
            &AMOUNT,
            &symbol_short!("USDC"),
            &String::from_str(&env, "medical equipment"),
            &milestones,
        );
        Self {
            env,
            contract_id,
            donor,
            recipient,
        }
    }

    pub fn client(&self) -> MedicalDonationEscrowClient<'_> {
        MedicalDonationEscrowClient::new(&self.env, &self.contract_id)
    }

    pub fn funded(&self) {
        let client = self.client();
        client.deposit(&DONATION_ID, &self.donor, &AMOUNT, &symbol_short!("USDC"));
    }

    pub fn verify(&self, verifier: &Address, index: u32) {
        let client = self.client();
        client.verify_milestone(
            &DONATION_ID,
            verifier,
            &index,
            &String::from_str(&self.env, "evidence"),
        );
    }
}
