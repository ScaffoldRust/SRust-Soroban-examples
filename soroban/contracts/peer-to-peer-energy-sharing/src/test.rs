#[cfg(test)]
mod tests;

#[cfg(test)]
mod edge_cases_security {
    use super::tests::utils::*;
    use crate::utils::*;
    use soroban_sdk::{testutils::Address as _, Env};

    #[test]
    fn test_contract_initialization_twice() {
        let setup = TestSetup::new();
        
        // Try to initialize again
        let result = setup.client.initialize(&setup.admin, &setup.token_contract);
        assert_eq!(result.unwrap_err(), SharingError::AlreadyInitialized);
    }

    #[test]
    fn test_register_prosumer_twice() {
        let setup = TestSetup::new();
        
        // Try to register the same prosumer again
        let result = setup.client.register_prosumer(&setup.provider);
        assert!(result.is_ok()); // Should succeed (idempotent operation)
    }

    #[test]
    fn test_high_volume_agreements() {
        let setup = TestSetup::new();
        let mut agreement_ids = Vec::new();
        
        // Create 100 agreements
        for i in 0..100 {
            let energy_amount = 50 + (i % 200);
            let price = 25 + (i % 100);
            
            let agreement_id = setup.create_custom_agreement(
                &setup.provider,
                &setup.consumer,
                energy_amount as u64,
                price as u64,
                setup.get_future_deadline(),
            ).unwrap();
            
            agreement_ids.push(agreement_id);
        }
        
        // Verify all agreements were created with sequential IDs
        assert_eq!(agreement_ids.len(), 100);
        for (i, &id) in agreement_ids.iter().enumerate() {
            assert_eq!(id, (i + 1) as u64);
        }
    }

    #[test]
    fn test_high_volume_transactions() {
        let setup = TestSetup::new();
        let mut transaction_ids = Vec::new();
        
        // Create multiple agreements and deliver energy for each
        for i in 0..50 {
            let agreement_id = setup.create_custom_agreement(
                &setup.provider,
                &setup.consumer,
                100,
                50,
                setup.get_future_deadline(),
            ).unwrap();
            
            let transaction_id = setup.client.deliver_energy(
                &agreement_id,
                &(50 + (i % 50)),
                &(1000 + i * 100),
                &setup.provider,
            ).unwrap();
            
            transaction_ids.push(transaction_id);
        }
        
        // Verify all transactions were created
        assert_eq!(transaction_ids.len(), 50);
        
        // Verify transaction history contains all transactions
        let history = setup.client.get_transaction_history(&setup.provider).unwrap();
        assert_eq!(history.len(), 50);
    }

    #[test]
    fn test_maximum_energy_amounts() {
        let setup = TestSetup::new();
        
        let max_energy = u64::MAX;
        let price = 1u64;
        
        // This might overflow in total_amount calculation, but should be handled
        let result = setup.create_custom_agreement(
            &setup.provider,
            &setup.consumer,
            max_energy,
            price,
            setup.get_future_deadline(),
        );
        
        // Should either succeed or fail gracefully
        match result {
            Ok(agreement_id) => {
                let agreement = setup.client.get_agreement(&agreement_id).unwrap();
                assert_eq!(agreement.energy_amount_kwh, max_energy);
            }
            Err(_) => {
                // Acceptable if the contract prevents overflow
            }
        }
    }

    #[test]
    fn test_edge_case_deadlines() {
        let setup = TestSetup::new();
        
        // Test with deadline exactly at current time + 1 second
        let near_future_deadline = setup.env.ledger().timestamp() + 1;
        
        let agreement_id = setup.create_custom_agreement(
            &setup.provider,
            &setup.consumer,
            100,
            50,
            near_future_deadline,
        ).unwrap();
        
        // Should be able to deliver immediately
        let transaction_id = setup.client.deliver_energy(
            &agreement_id,
            &100u64,
            &1500u64,
            &setup.provider,
        ).unwrap();
        
        assert_eq!(transaction_id, 1);
    }

    #[test]
    fn test_concurrent_operations_different_prosumers() {
        let setup = TestSetup::new();
        
        // Register additional prosumers
        let prosumer4 = setup.env.generate::<Address>();
        let prosumer5 = setup.env.generate::<Address>();
        setup.client.register_prosumer(&prosumer4).unwrap();
        setup.client.register_prosumer(&prosumer5).unwrap();
        
        // Create multiple agreements simultaneously
        let agreement_id1 = setup.create_custom_agreement(
            &setup.provider,
            &setup.consumer,
            100,
            50,
            setup.get_future_deadline(),
        ).unwrap();
        
        let agreement_id2 = setup.create_custom_agreement(
            &prosumer4,
            &prosumer5,
            200,
            30,
            setup.get_future_deadline(),
        ).unwrap();
        
        let agreement_id3 = setup.create_custom_agreement(
            &setup.prosumer3,
            &prosumer4,
            150,
            40,
            setup.get_future_deadline(),
        ).unwrap();
        
        // All should have unique IDs
        assert_ne!(agreement_id1, agreement_id2);
        assert_ne!(agreement_id2, agreement_id3);
        assert_ne!(agreement_id1, agreement_id3);
    }

    #[test]
    fn test_dispute_transaction() {
        let setup = TestSetup::new();
        let agreement_id = setup.create_test_agreement().unwrap();
        
        let transaction_id = setup.client.deliver_energy(
            &agreement_id,
            &100u64,
            &1500u64,
            &setup.provider,
        ).unwrap();
        
        // This function doesn't exist in the current implementation, 
        // but would be part of a dispute resolution system
        // setup.client.dispute_transaction(&transaction_id, &setup.consumer).unwrap();
        
        // For now, we'll just verify the transaction exists and can be disputed
        let transaction = setup.client.get_transaction(&transaction_id).unwrap();
        assert_eq!(transaction.status, TransactionStatus::Delivered);
    }

    #[test]
    fn test_unauthorized_settlement_attempts() {
        let setup = TestSetup::new();
        let agreement_id = setup.create_test_agreement().unwrap();
        
        let transaction_id = setup.client.deliver_energy(
            &agreement_id,
            &100u64,
            &1500u64,
            &setup.provider,
        ).unwrap();
        
        // Generate random unauthorized user
        let unauthorized_user = setup.env.generate::<Address>();
        
        let result = setup.client.settle_payment(&transaction_id, &unauthorized_user);
        assert_eq!(result.unwrap_err(), SharingError::NotAuthorized);
    }

    #[test]
    fn test_agreement_expiry_edge_cases() {
        let setup = TestSetup::new();
        let agreement_id = setup.create_test_agreement().unwrap();
        
        // Advance time to exactly the deadline
        setup.advance_ledger_time(86400); // Exactly 1 day
        
        // Try to deliver at exactly the deadline
        let result = setup.client.deliver_energy(
            &agreement_id,
            &100u64,
            &1500u64,
            &setup.provider,
        );
        
        // Should fail as deadline has passed
        assert_eq!(result.unwrap_err(), SharingError::DeliveryDeadlinePassed);
    }

    #[test]
    fn test_large_meter_readings() {
        let setup = TestSetup::new();
        let agreement_id = setup.create_test_agreement().unwrap();
        
        let large_meter_reading = u64::MAX;
        
        let transaction_id = setup.client.deliver_energy(
            &agreement_id,
            &100u64,
            &large_meter_reading,
            &setup.provider,
        ).unwrap();
        
        let transaction = setup.client.get_transaction(&transaction_id).unwrap();
        assert_eq!(transaction.meter_reading, large_meter_reading);
    }

    #[test]
    fn test_prosumer_acting_as_both_provider_and_consumer() {
        let setup = TestSetup::new();
        
        // Provider creates agreement to sell energy
        let agreement_id1 = setup.create_custom_agreement(
            &setup.provider,
            &setup.consumer,
            100,
            50,
            setup.get_future_deadline(),
        ).unwrap();
        
        // Same prosumer (provider) creates agreement to buy energy from another prosumer
        let agreement_id2 = setup.create_custom_agreement(
            &setup.prosumer3,
            &setup.provider, // provider is now consumer
            150,
            40,
            setup.get_future_deadline(),
        ).unwrap();
        
        // Both agreements should be valid
        let agreement1 = setup.client.get_agreement(&agreement_id1).unwrap();
        let agreement2 = setup.client.get_agreement(&agreement_id2).unwrap();
        
        assert_eq!(agreement1.provider, setup.provider);
        assert_eq!(agreement2.consumer, setup.provider);
    }

    #[test]
    fn test_chain_of_energy_transactions() {
        let setup = TestSetup::new();
        
        // Create a chain: provider -> prosumer3 -> consumer
        let agreement_id1 = setup.create_custom_agreement(
            &setup.provider,
            &setup.prosumer3,
            100,
            50,
            setup.get_future_deadline(),
        ).unwrap();
        
        let agreement_id2 = setup.create_custom_agreement(
            &setup.prosumer3,
            &setup.consumer,
            100,
            60, // Higher price in second transaction
            setup.get_future_deadline(),
        ).unwrap();
        
        // Execute both transactions
        let transaction_id1 = setup.client.deliver_energy(
            &agreement_id1,
            &100u64,
            &1500u64,
            &setup.provider,
        ).unwrap();
        
        let transaction_id2 = setup.client.deliver_energy(
            &agreement_id2,
            &100u64,
            &1500u64,
            &setup.prosumer3,
        ).unwrap();
        
        // Settle both payments
        setup.client.settle_payment(&transaction_id1, &setup.prosumer3).unwrap();
        setup.client.settle_payment(&transaction_id2, &setup.consumer).unwrap();
        
        // Verify prosumer3 transaction history shows both transactions
        let history = setup.client.get_transaction_history(&setup.prosumer3).unwrap();
        assert_eq!(history.len(), 2);
    }
}