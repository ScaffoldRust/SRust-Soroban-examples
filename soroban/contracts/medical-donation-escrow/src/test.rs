use soroban_sdk::{symbol_short, testutils::Address as _, Address, Env, String, Vec};

use crate::{MedicalDonationEscrow, MedicalDonationEscrowClient};

#[test]
fn test_initialize_donation() {
    let env = Env::default();
    let contract_id = env.register(MedicalDonationEscrow, ());
    let client = MedicalDonationEscrowClient::new(&env, &contract_id);

    let donor = Address::generate(&env);
    let recipient = Address::generate(&env);
    let donation_id = 1u64;
    let amount = 1000i128;
    let token = symbol_short!("USDC");
    let description = String::from_str(&env, "Medical equipment donation");
    let mut milestones = Vec::new(&env);
    milestones.push_back(String::from_str(&env, "Equipment ordered"));
    milestones.push_back(String::from_str(&env, "Equipment delivered"));
    milestones.push_back(String::from_str(&env, "Equipment installed"));

    client.initialize(
        &donation_id,
        &donor,
        &recipient,
        &amount,
        &token,
        &description,
        &milestones,
    );

    let donation = client.get_donation(&donation_id).unwrap();
    assert_eq!(donation.id, donation_id);
    assert_eq!(donation.donor, donor);
    assert_eq!(donation.recipient, recipient);
    assert_eq!(donation.amount, amount);
    assert_eq!(donation.token, token);
    assert_eq!(donation.description, description);
    assert_eq!(donation.milestones.len(), 3);
    assert_eq!(donation.status, crate::escrow::DonationStatus::Pending);
}

#[test]
fn test_deposit_funds() {
    let env = Env::default();
    let contract_id = env.register(MedicalDonationEscrow, ());
    let client = MedicalDonationEscrowClient::new(&env, &contract_id);

    let donor = Address::generate(&env);
    let recipient = Address::generate(&env);
    let donation_id = 1u64;
    let amount = 1000i128;
    let token = symbol_short!("USDC");
    let description = String::from_str(&env, "Medical equipment donation");
    let mut milestones = Vec::new(&env);
    milestones.push_back(String::from_str(&env, "Equipment ordered"));

    // Initialize donation
    client.initialize(
        &donation_id,
        &donor,
        &recipient,
        &amount,
        &token,
        &description,
        &milestones,
    );

    // Deposit funds
    client.deposit(&donation_id, &donor, &amount, &token);

    let donation = client.get_donation(&donation_id).unwrap();
    assert_eq!(donation.status, crate::escrow::DonationStatus::Funded);
    assert!(donation.funded_at.is_some());
}

#[test]
fn test_verify_milestone() {
    let env = Env::default();
    let contract_id = env.register(MedicalDonationEscrow, ());
    let client = MedicalDonationEscrowClient::new(&env, &contract_id);

    let donor = Address::generate(&env);
    let recipient = Address::generate(&env);
    let donation_id = 1u64;
    let amount = 1000i128;
    let token = symbol_short!("USDC");
    let description = String::from_str(&env, "Medical equipment donation");
    let mut milestones = Vec::new(&env);
    milestones.push_back(String::from_str(&env, "Equipment ordered"));
    milestones.push_back(String::from_str(&env, "Equipment delivered"));

    // Initialize and fund donation
    client.initialize(
        &donation_id,
        &donor,
        &recipient,
        &amount,
        &token,
        &description,
        &milestones,
    );
    client.deposit(&donation_id, &donor, &amount, &token);

    // Verify first milestone
    let verification_data = String::from_str(&env, "Order confirmation #12345");
    client.verify_milestone(&donation_id, &recipient, &0u32, &verification_data);

    // Check milestone status
    assert!(client.get_milestone_status(&donation_id, &0u32));
    assert!(!client.get_milestone_status(&donation_id, &1u32));

    let donation = client.get_donation(&donation_id).unwrap();
    assert_eq!(donation.status, crate::escrow::DonationStatus::InProgress);
}

#[test]
fn test_complete_all_milestones_and_release() {
    let env = Env::default();
    let contract_id = env.register(MedicalDonationEscrow, ());
    let client = MedicalDonationEscrowClient::new(&env, &contract_id);

    let donor = Address::generate(&env);
    let recipient = Address::generate(&env);
    let donation_id = 1u64;
    let amount = 1000i128;
    let token = symbol_short!("USDC");
    let description = String::from_str(&env, "Medical equipment donation");
    let mut milestones = Vec::new(&env);
    milestones.push_back(String::from_str(&env, "Equipment ordered"));
    milestones.push_back(String::from_str(&env, "Equipment delivered"));

    // Initialize and fund donation
    client.initialize(
        &donation_id,
        &donor,
        &recipient,
        &amount,
        &token,
        &description,
        &milestones,
    );
    client.deposit(&donation_id, &donor, &amount, &token);

    // Verify all milestones
    client.verify_milestone(
        &donation_id,
        &recipient,
        &0u32,
        &String::from_str(&env, "Order confirmation #12345"),
    );
    client.verify_milestone(
        &donation_id,
        &recipient,
        &1u32,
        &String::from_str(&env, "Delivery confirmation #67890"),
    );

    // Check all milestones are verified
    assert!(client.get_milestone_status(&donation_id, &0u32));
    assert!(client.get_milestone_status(&donation_id, &1u32));

    let donation = client.get_donation(&donation_id).unwrap();
    assert_eq!(donation.status, crate::escrow::DonationStatus::Completed);

    // Release funds
    client.release_funds(&donation_id, &recipient);
}

#[test]
fn test_refund() {
    let env = Env::default();
    let contract_id = env.register(MedicalDonationEscrow, ());
    let client = MedicalDonationEscrowClient::new(&env, &contract_id);

    let donor = Address::generate(&env);
    let recipient = Address::generate(&env);
    let donation_id = 1u64;
    let amount = 1000i128;
    let token = symbol_short!("USDC");
    let description = String::from_str(&env, "Medical equipment donation");
    let mut milestones = Vec::new(&env);
    milestones.push_back(String::from_str(&env, "Equipment ordered"));

    // Initialize and fund donation
    client.initialize(
        &donation_id,
        &donor,
        &recipient,
        &amount,
        &token,
        &description,
        &milestones,
    );
    client.deposit(&donation_id, &donor, &amount, &token);

    // Process refund
    client.refund(&donation_id, &donor);

    let donation = client.get_donation(&donation_id).unwrap();
    assert_eq!(donation.status, crate::escrow::DonationStatus::Refunded);
    assert!(donation.refunded_at.is_some());
}

#[test]
fn test_pause_and_resume_donation() {
    let env = Env::default();
    let contract_id = env.register(MedicalDonationEscrow, ());
    let client = MedicalDonationEscrowClient::new(&env, &contract_id);

    let donor = Address::generate(&env);
    let recipient = Address::generate(&env);
    let donation_id = 1u64;
    let amount = 1000i128;
    let token = symbol_short!("USDC");
    let description = String::from_str(&env, "Medical equipment donation");
    let mut milestones = Vec::new(&env);
    milestones.push_back(String::from_str(&env, "Equipment ordered"));

    // Initialize and fund donation
    client.initialize(
        &donation_id,
        &donor,
        &recipient,
        &amount,
        &token,
        &description,
        &milestones,
    );
    client.deposit(&donation_id, &donor, &amount, &token);

    // Pause donation (donor is admin by default)
    client.pause_donation(&donation_id, &donor);

    let donation = client.get_donation(&donation_id).unwrap();
    assert_eq!(donation.status, crate::escrow::DonationStatus::Paused);

    // Resume donation
    client.resume_donation(&donation_id, &donor);

    let donation = client.get_donation(&donation_id).unwrap();
    assert_eq!(donation.status, crate::escrow::DonationStatus::Funded);
}

#[test]
fn test_get_user_donations() {
    let env = Env::default();
    let contract_id = env.register(MedicalDonationEscrow, ());
    let client = MedicalDonationEscrowClient::new(&env, &contract_id);

    let donor = Address::generate(&env);
    let recipient = Address::generate(&env);
    let amount = 1000i128;
    let token = symbol_short!("USDC");
    let description = String::from_str(&env, "Medical equipment donation");
    let mut milestones = Vec::new(&env);
    milestones.push_back(String::from_str(&env, "Equipment ordered"));

    // Create multiple donations
    for i in 1..=3 {
        client.initialize(
            &i,
            &donor,
            &recipient,
            &amount,
            &token,
            &description,
            &milestones,
        );
    }

    let user_donations = client.get_user_donations(&donor);
    assert_eq!(user_donations.len(), 3);
    assert!(user_donations.contains(&1));
    assert!(user_donations.contains(&2));
    assert!(user_donations.contains(&3));
}

#[test]
#[should_panic(expected = "Invalid amount")]
fn test_initialize_invalid_amount() {
    let env = Env::default();
    let contract_id = env.register(MedicalDonationEscrow, ());
    let client = MedicalDonationEscrowClient::new(&env, &contract_id);

    let donor = Address::generate(&env);
    let recipient = Address::generate(&env);
    let donation_id = 1u64;
    let amount = 0i128; // Invalid amount
    let token = symbol_short!("USDC");
    let description = String::from_str(&env, "Medical equipment donation");
    let mut milestones = Vec::new(&env);
    milestones.push_back(String::from_str(&env, "Equipment ordered"));

    client.initialize(
        &donation_id,
        &donor,
        &recipient,
        &amount,
        &token,
        &description,
        &milestones,
    );
}

#[test]
#[should_panic(expected = "No milestones provided")]
fn test_initialize_no_milestones() {
    let env = Env::default();
    let contract_id = env.register(MedicalDonationEscrow, ());
    let client = MedicalDonationEscrowClient::new(&env, &contract_id);

    let donor = Address::generate(&env);
    let recipient = Address::generate(&env);
    let donation_id = 1u64;
    let amount = 1000i128;
    let token = symbol_short!("USDC");
    let description = String::from_str(&env, "Medical equipment donation");
    let milestones = Vec::new(&env); // Empty milestones

    client.initialize(
        &donation_id,
        &donor,
        &recipient,
        &amount,
        &token,
        &description,
        &milestones,
    );
}

#[test]
#[should_panic(expected = "Unauthorized verifier")]
fn test_unauthorized_milestone_verification() {
    let env = Env::default();
    let contract_id = env.register(MedicalDonationEscrow, ());
    let client = MedicalDonationEscrowClient::new(&env, &contract_id);

    let donor = Address::generate(&env);
    let recipient = Address::generate(&env);
    let unauthorized = Address::generate(&env);
    let donation_id = 1u64;
    let amount = 1000i128;
    let token = symbol_short!("USDC");
    let description = String::from_str(&env, "Medical equipment donation");
    let mut milestones = Vec::new(&env);
    milestones.push_back(String::from_str(&env, "Equipment ordered"));

    // Initialize and fund donation
    client.initialize(
        &donation_id,
        &donor,
        &recipient,
        &amount,
        &token,
        &description,
        &milestones,
    );
    client.deposit(&donation_id, &donor, &amount, &token);

    // Try to verify milestone with unauthorized user
    client.verify_milestone(
        &donation_id,
        &unauthorized,
        &0u32,
        &String::from_str(&env, "Order confirmation #12345"),
    );
}

#[test]
fn test_request_and_approve_refund() {
    let env = Env::default();
    let contract_id = env.register(MedicalDonationEscrow, ());
    let client = MedicalDonationEscrowClient::new(&env, &contract_id);

    let donor = Address::generate(&env);
    let recipient = Address::generate(&env);
    let donation_id = 1u64;
    let amount = 1000i128;
    let token = symbol_short!("USDC");
    let description = String::from_str(&env, "Medical equipment donation");
    let mut milestones = Vec::new(&env);
    milestones.push_back(String::from_str(&env, "Equipment ordered"));

    // Initialize and fund donation
    client.initialize(
        &donation_id,
        &donor,
        &recipient,
        &amount,
        &token,
        &description,
        &milestones,
    );
    client.deposit(&donation_id, &donor, &amount, &token);

    // Request refund
    let reason = String::from_str(&env, "Equipment not available");
    client.request_refund(&donation_id, &donor, &reason);

    // Check refund request exists
    let refund_request = client.get_refund_request(&donation_id).unwrap();
    assert_eq!(refund_request.donation_id, donation_id);
    assert_eq!(refund_request.requester, donor);
    assert_eq!(refund_request.reason, reason);
    assert!(!refund_request.approved);

    // Approve refund (donor is admin by default)
    client.approve_refund(&donation_id, &donor);

    // Check donation is refunded
    let donation = client.get_donation(&donation_id).unwrap();
    assert_eq!(donation.status, crate::escrow::DonationStatus::Refunded);
}

#[test]
fn test_donation_metrics() {
    let env = Env::default();
    let contract_id = env.register(MedicalDonationEscrow, ());
    let client = MedicalDonationEscrowClient::new(&env, &contract_id);

    let donor = Address::generate(&env);
    let recipient = Address::generate(&env);
    let donation_id = 1u64;
    let amount = 1000i128;
    let token = symbol_short!("USDC");
    let description = String::from_str(&env, "Medical equipment donation");
    let mut milestones = Vec::new(&env);
    milestones.push_back(String::from_str(&env, "Equipment ordered"));
    milestones.push_back(String::from_str(&env, "Equipment delivered"));

    // Initialize and fund donation
    client.initialize(
        &donation_id,
        &donor,
        &recipient,
        &amount,
        &token,
        &description,
        &milestones,
    );
    client.deposit(&donation_id, &donor, &amount, &token);

    // Get initial metrics
    let metrics = client.get_donation_metrics(&donation_id).unwrap();
    assert_eq!(metrics.donation_id, donation_id);
    assert_eq!(metrics.total_milestones, 2);
    assert_eq!(metrics.completed_milestones, 0);
    assert_eq!(metrics.progress_percentage, 0);

    // Verify first milestone
    client.verify_milestone(
        &donation_id,
        &recipient,
        &0u32,
        &String::from_str(&env, "Order confirmation #12345"),
    );

    // Get updated metrics
    let metrics = client.get_donation_metrics(&donation_id).unwrap();
    assert_eq!(metrics.completed_milestones, 1);
    assert_eq!(metrics.progress_percentage, 50);
}

#[test]
fn test_milestone_verification_details() {
    let env = Env::default();
    let contract_id = env.register(MedicalDonationEscrow, ());
    let client = MedicalDonationEscrowClient::new(&env, &contract_id);

    let donor = Address::generate(&env);
    let recipient = Address::generate(&env);
    let donation_id = 1u64;
    let amount = 1000i128;
    let token = symbol_short!("USDC");
    let description = String::from_str(&env, "Medical equipment donation");
    let mut milestones = Vec::new(&env);
    milestones.push_back(String::from_str(&env, "Equipment ordered"));

    // Initialize and fund donation
    client.initialize(
        &donation_id,
        &donor,
        &recipient,
        &amount,
        &token,
        &description,
        &milestones,
    );
    client.deposit(&donation_id, &donor, &amount, &token);

    // Verify milestone
    let verification_data = String::from_str(&env, "Order confirmation #12345");
    client.verify_milestone(&donation_id, &recipient, &0u32, &verification_data);

    // Get milestone verification details
    let verification = client
        .get_milestone_verification(&donation_id, &0u32)
        .unwrap();
    assert_eq!(verification.milestone_index, 0);
    assert!(verification.verified);
    assert!(verification.verified_at.is_some());
    assert_eq!(verification.verifier, Some(recipient));
    assert_eq!(verification.verification_data, Some(verification_data));

    // Get all milestone verifications
    let all_verifications = client.get_all_milestone_verifications(&donation_id);
    assert_eq!(all_verifications.len(), 1);
    assert_eq!(all_verifications.get(0).unwrap().milestone_index, 0);
}

#[test]
fn test_calculate_refund_amount() {
    let env = Env::default();
    let contract_id = env.register(MedicalDonationEscrow, ());
    let client = MedicalDonationEscrowClient::new(&env, &contract_id);

    let donor = Address::generate(&env);
    let recipient = Address::generate(&env);
    let donation_id = 1u64;
    let amount = 1000i128;
    let token = symbol_short!("USDC");
    let description = String::from_str(&env, "Medical equipment donation");
    let mut milestones = Vec::new(&env);
    milestones.push_back(String::from_str(&env, "Equipment ordered"));
    milestones.push_back(String::from_str(&env, "Equipment delivered"));

    // Initialize and fund donation
    client.initialize(
        &donation_id,
        &donor,
        &recipient,
        &amount,
        &token,
        &description,
        &milestones,
    );
    client.deposit(&donation_id, &donor, &amount, &token);

    // Test refund calculations
    let full_refund = client.calculate_refund_amount(&donation_id, &0u32).unwrap();
    assert_eq!(full_refund, 1000); // Full refund if no milestones completed

    let half_refund = client.calculate_refund_amount(&donation_id, &1u32).unwrap();
    assert_eq!(half_refund, 500); // Half refund if 1 of 2 milestones completed

    let no_refund = client.calculate_refund_amount(&donation_id, &2u32).unwrap();
    assert_eq!(no_refund, 0); // No refund if all milestones completed
}

#[test]
fn test_validate_donation_parameters() {
    let env = Env::default();
    let contract_id = env.register(MedicalDonationEscrow, ());
    let client = MedicalDonationEscrowClient::new(&env, &contract_id);

    let amount = 1000i128;
    let token = symbol_short!("USDC");
    let mut milestones = Vec::new(&env);
    milestones.push_back(String::from_str(&env, "Equipment ordered"));

    // Valid parameters
    let result = client.validate_donation_parameters(&amount, &token, &milestones);
    assert!(result);

    // Invalid amount
    let result = client.validate_donation_parameters(&0i128, &token, &milestones);
    assert!(!result);

    // Empty milestones
    let empty_milestones = Vec::new(&env);
    let result = client.validate_donation_parameters(&amount, &token, &empty_milestones);
    assert!(!result);

    // Amount too large
    let large_amount = 2_000_000_000_000_000_000i128;
    let result = client.validate_donation_parameters(&large_amount, &token, &milestones);
    assert!(!result);
}

#[test]
fn test_queues() {
    let env = Env::default();
    let contract_id = env.register(MedicalDonationEscrow, ());
    let client = MedicalDonationEscrowClient::new(&env, &contract_id);

    let donor = Address::generate(&env);
    let recipient = Address::generate(&env);
    let donation_id1 = 1u64;
    let donation_id2 = 2u64;
    let amount = 1000i128;
    let token = symbol_short!("USDC");
    let description = String::from_str(&env, "Medical equipment donation");
    let mut milestones = Vec::new(&env);
    milestones.push_back(String::from_str(&env, "Equipment ordered"));

    // Initialize and fund first donation
    client.initialize(
        &donation_id1,
        &donor,
        &recipient,
        &amount,
        &token,
        &description,
        &milestones,
    );
    client.deposit(&donation_id1, &donor, &amount, &token);

    // Complete all milestones to add to release queue
    client.verify_milestone(
        &donation_id1,
        &recipient,
        &0u32,
        &String::from_str(&env, "Order confirmation #12345"),
    );

    // Check release queue
    let release_queue = client.get_release_queue();
    assert!(release_queue.contains(&donation_id1));

    // Initialize and fund second donation for refund test
    client.initialize(
        &donation_id2,
        &donor,
        &recipient,
        &amount,
        &token,
        &description,
        &milestones,
    );
    client.deposit(&donation_id2, &donor, &amount, &token);

    // Process refund for second donation (auto-approval)
    client.refund(&donation_id2, &donor);

    // Check refund queue (should contain the refunded donation)
    let refund_queue = client.get_refund_queue();
    assert!(refund_queue.contains(&donation_id2));

    // Check that second donation is refunded
    let donation2 = client.get_donation(&donation_id2).unwrap();
    assert_eq!(donation2.status, crate::escrow::DonationStatus::Refunded);

    // Check that first donation is still completed
    let donation1 = client.get_donation(&donation_id1).unwrap();
    assert_eq!(donation1.status, crate::escrow::DonationStatus::Completed);
}
