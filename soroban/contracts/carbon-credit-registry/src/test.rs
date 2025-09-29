#[cfg(test)]
mod test {
    use soroban_sdk::{testutils::Address as _, Address, BytesN, String, Vec, Map};
    use super::{CarbonCreditRegistry, CarbonCreditRegistryClient};

    #[test]
    fn test_initialization() {
        let env = soroban_sdk::Env::default();
        let contract_id = env.register_contract(None, CarbonCreditRegistry);
        let client = CarbonCreditRegistryClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let trading_fee_rate = 25u32; // 0.25%
        let retirement_fee_rate = 10u32; // 0.1%

        env.mock_auths(&[crate::CarbonCreditRegistryClient::initialize(
            &client,
            &admin,
            &trading_fee_rate,
            &retirement_fee_rate,
        )]);

        client.initialize(&admin, &trading_fee_rate, &retirement_fee_rate);

        // Verify stats are initialized correctly
        let stats = client.get_contract_stats();
        assert_eq!(stats.0, 0u32); // issuer_count
        assert_eq!(stats.1, 0u32); // credit_count
        assert_eq!(stats.2, 0i128); // total_issued
        assert_eq!(stats.3, 0i128); // total_retired
    }

    #[test]
    fn test_register_issuer() {
        let env = soroban_sdk::Env::default();
        let contract_id = env.register_contract(None, CarbonCreditRegistry);
        let client = CarbonCreditRegistryClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        
        // Initialize contract first
        env.mock_auths(&[crate::CarbonCreditRegistryClient::initialize(
            &client,
            &admin,
            &0u32,
            &0u32,
        )]);
        client.initialize(&admin, &0u32, &0u32);

        let issuer = Address::generate(&env);
        let name = String::from_str(&env, "Test Issuer");
        let mut standards = Vec::new(&env);
        standards.push_back(String::from_str(&env, "VERRA"));
        standards.push_back(String::from_str(&env, "GOLD_STANDARD"));

        env.mock_auths(&[crate::CarbonCreditRegistryClient::register_issuer(
            &client,
            &issuer,
            &name,
            &standards,
        )]);
        
        client.register_issuer(&issuer, &name, &standards);

        // Verify issuer was registered
        let issuer_profile = client.get_issuer_profile(&issuer);
        assert!(issuer_profile.is_some());
        let profile = issuer_profile.unwrap();
        assert_eq!(profile.address, issuer);
        assert_eq!(profile.name, name);
        assert_eq!(profile.is_active, true);
        assert_eq!(profile.total_issued, 0i128);
        assert_eq!(profile.total_retired, 0i128);
    }

    #[test]
    fn test_issue_credit() {
        let env = soroban_sdk::Env::default();
        let contract_id = env.register_contract(None, CarbonCreditRegistry);
        let client = CarbonCreditRegistryClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        
        // Initialize contract first
        env.mock_auths(&[crate::CarbonCreditRegistryClient::initialize(
            &client,
            &admin,
            &0u32,
            &0u32,
        )]);
        client.initialize(&admin, &0u32, &0u32);

        let issuer = Address::generate(&env);
        let name = String::from_str(&env, "Test Issuer");
        let mut standards = Vec::new(&env);
        standards.push_back(String::from_str(&env, "VERRA"));

        env.mock_auths(&[crate::CarbonCreditRegistryClient::register_issuer(
            &client,
            &issuer,
            &name,
            &standards,
        )]);
        client.register_issuer(&issuer, &name, &standards);

        // Issue a credit
        let project_type = String::from_str(&env, "RENEWABLE_ENERGY");
        let project_location = String::from_str(&env, "Brazil");
        let verification_standard = String::from_str(&env, "VERRA");
        let vintage_year = 2024u32;
        let quantity = 1000i128;
        let verification_hash = BytesN::from_array(&env, &[0u8; 32]);
        let mut metadata = Map::new(&env);
        metadata.set(String::from_str(&env, "project_id"), String::from_str(&env, "PROJ-001"));

        env.mock_auths(&[crate::CarbonCreditRegistryClient::issue_credit(
            &client,
            &issuer,
            &project_type,
            &project_location,
            &verification_standard,
            &vintage_year,
            &quantity,
            &verification_hash,
            &metadata,
        )]);

        let credit_id = client.issue_credit(
            &issuer,
            &project_type,
            &project_location,
            &verification_standard,
            &vintage_year,
            &quantity,
            &verification_hash,
            &metadata,
        );

        // Verify credit was issued
        let credit = client.get_credit_status(&credit_id);
        assert!(credit.is_some());
        let credit = credit.unwrap();
        assert_eq!(credit.issuer, issuer);
        assert_eq!(credit.project_type, project_type);
        assert_eq!(credit.quantity, quantity);
        assert_eq!(credit.current_owner, issuer);

        // Verify stats were updated
        let stats = client.get_contract_stats();
        assert_eq!(stats.1, 1u32); // credit_count
        assert_eq!(stats.2, quantity); // total_issued
    }
}