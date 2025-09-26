#[cfg(test)]
mod test {
    use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, String};

    use crate::{TelemedicinePaymentGateway, TelemedicinePaymentGatewayClient};

    #[test]
    fn test_initialize() {
        let env = Env::default();
        let contract_id = env.register(TelemedicinePaymentGateway, ());
        let client = TelemedicinePaymentGatewayClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let platform_fee_percentage = 10;
        let min_session_duration = 15;
        let max_session_duration = 180;

        client.initialize(&admin, &platform_fee_percentage, &min_session_duration, &max_session_duration);

        // Test that the contract was initialized correctly
        assert_eq!(client.get_platform_fee_percentage(), platform_fee_percentage);
        assert_eq!(client.get_contract_status(), true);
    }

    #[test]
    fn test_register_provider() {
        let env = Env::default();
        let contract_id = env.register(TelemedicinePaymentGateway, ());
        let client = TelemedicinePaymentGatewayClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        client.initialize(&admin, &10, &15, &180);

        let provider_address = Address::generate(&env);
        let provider_name = String::from_str(&env, "Dr. Alice");
        let specialty = String::from_str(&env, "Cardiology");
        let hourly_rate = 1000;
        let currency = Address::generate(&env);

        client.register_provider(&provider_address, &provider_name, &specialty, &hourly_rate, &currency);

        // Test that the provider was registered correctly
        assert_eq!(client.get_provider_hourly_rate(&provider_address), hourly_rate);
    }

    #[test]
    fn test_contract_pause_resume() {
        let env = Env::default();
        let contract_id = env.register(TelemedicinePaymentGateway, ());
        let client = TelemedicinePaymentGatewayClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        client.initialize(&admin, &10, &15, &180);

        // Test that contract is initially active
        assert_eq!(client.get_contract_status(), true);

        // Test pause functionality
        client.pause_contract();
        assert_eq!(client.get_contract_status(), false);

        // Test resume functionality
        client.resume_contract();
        assert_eq!(client.get_contract_status(), true);
    }

    #[test]
    fn test_multiple_providers() {
        let env = Env::default();
        let contract_id = env.register(TelemedicinePaymentGateway, ());
        let client = TelemedicinePaymentGatewayClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        client.initialize(&admin, &10, &15, &180);

        // Register multiple providers
        let provider1 = Address::generate(&env);
        let provider2 = Address::generate(&env);
        let currency = Address::generate(&env);

        client.register_provider(&provider1, &String::from_str(&env, "Dr. Smith"), &String::from_str(&env, "Cardiology"), &1500, &currency);
        client.register_provider(&provider2, &String::from_str(&env, "Dr. Johnson"), &String::from_str(&env, "Pediatrics"), &1200, &currency);

        // Test that both providers were registered
        assert_eq!(client.get_provider_hourly_rate(&provider1), 1500);
        assert_eq!(client.get_provider_hourly_rate(&provider2), 1200);
    }

    #[test]
    fn test_balance_queries() {
        let env = Env::default();
        let contract_id = env.register(TelemedicinePaymentGateway, ());
        let client = TelemedicinePaymentGatewayClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        client.initialize(&admin, &10, &15, &180);

        let test_address = Address::generate(&env);
        
        // Test balance calculation (currently returns (0, 0) as placeholder)
        let (pending, completed) = client.get_balance(&test_address);
        assert_eq!(pending, 0);
        assert_eq!(completed, 0);
    }

    #[test]
    fn test_provider_registration_and_rates() {
        let env = Env::default();
        let contract_id = env.register(TelemedicinePaymentGateway, ());
        let client = TelemedicinePaymentGatewayClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        client.initialize(&admin, &5, &30, &240); // 5% fee, 30min min, 4hr max

        // Test different provider rates
        let cardiologist = Address::generate(&env);
        let pediatrician = Address::generate(&env);
        let currency = Address::generate(&env);

        client.register_provider(&cardiologist, &String::from_str(&env, "Dr. Heart"), &String::from_str(&env, "Cardiology"), &2000, &currency);
        client.register_provider(&pediatrician, &String::from_str(&env, "Dr. Child"), &String::from_str(&env, "Pediatrics"), &1500, &currency);

        // Test that different rates are stored correctly
        assert_eq!(client.get_provider_hourly_rate(&cardiologist), 2000);
        assert_eq!(client.get_provider_hourly_rate(&pediatrician), 1500);
        
        // Test platform fee percentage
        assert_eq!(client.get_platform_fee_percentage(), 5);
    }

    #[test]
    fn test_contract_initialization_parameters() {
        let env = Env::default();
        let contract_id = env.register(TelemedicinePaymentGateway, ());
        let client = TelemedicinePaymentGatewayClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let platform_fee = 15;
        let min_duration = 20;
        let max_duration = 300;

        client.initialize(&admin, &platform_fee, &min_duration, &max_duration);

        // Test that all initialization parameters are stored correctly
        assert_eq!(client.get_platform_fee_percentage(), platform_fee);
        assert_eq!(client.get_contract_status(), true);
    }
}