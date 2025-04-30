#![cfg(test)]

use super::*;
use crate::burn::DataKey as BurnDataKey;
use crate::governance::{DataKey as GovernanceDataKey, GovernanceProposal, ProposalStatus, Vote};
use crate::parameters::StablecoinParameters;
use soroban_sdk::{symbol_short, testutils::Address as _, vec, Address, Env, String, Symbol, Vec};

// Test module for StablecoinContract, covering minting, burning, governance, and oracle handling
mod tests {
    use super::*;

    // Helper to initialize contract with default parameters
    fn setup_contract(env: &Env) -> (StablecoinContractClient<'_>, Address, Symbol) {
        let contract_id = env.register(StablecoinContract, ()); // Register with empty constructor
        let client = StablecoinContractClient::new(&env, &contract_id);

        // Initialize parameters
        let params = StablecoinParameters {
            min_collateral_ratio: 15000, // 150%
            max_mint_amount: 1_000_000_000,
            rebalance_interval: 86400,
        };
        client.init_parameters(&params);

        let usdc = symbol_short!("USDC");
        (client, contract_id, usdc)
    }

    // Helper to set oracle price
    fn set_oracle_price(
        client: &StablecoinContractClient<'_>,
        oracle: Address,
        asset_pair: Symbol,
        rate: i128,
        timestamp: u64,
    ) {
        client.set_price(&oracle, &asset_pair, &rate, &timestamp);
    }

    // Test minting with sufficient collateral
    #[test]
    fn test_mint_with_proper_collateral() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, contract_id, usdc) = setup_contract(&env);
        let user = Address::generate(&env);

        // Deposit collateral (150 USDC for 100 stablecoins at 150% ratio)
        let collateral_amount = 150_000; // Scaled for precision
        client.deposit_collateral(&user, &usdc, &collateral_amount);

        // Set oracle price (1 USDC = 1 USD, scaled by 1e8)
        let oracle = Address::generate(&env);
        set_oracle_price(&client, oracle, usdc.clone(), 100_000_000, 1234567890);

        // Mint 100 stablecoins
        let mint_amount = 100_000;
        client.mint(&user, &mint_amount, &usdc);

        // Verify stablecoin balance
        let balance = env.as_contract(&contract_id, || {
            env.storage()
                .instance()
                .get::<_, i128>(&BurnDataKey::StablecoinBalance(user.clone()))
                .unwrap_or(0)
        });
        assert_eq!(
            balance, mint_amount,
            "Stablecoin balance should match minted amount"
        );

        // Verify reserves
        let total_reserve = client.get_total_reserve(&usdc);
        assert_eq!(
            total_reserve, collateral_amount,
            "Total reserve should match deposited collateral"
        );
    }

    // Test minting with insufficient collateral
    #[test]
    #[should_panic(expected = "Mint amount exceeds collateral-backed limit")]
    fn test_mint_under_collateral_threshold_fails() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _, usdc) = setup_contract(&env);
        let user = Address::generate(&env);

        // Deposit insufficient collateral (100 USDC for 100 stablecoins, below 150%)
        let collateral_amount = 100_000;
        client.deposit_collateral(&user, &usdc, &collateral_amount);

        // Set oracle price
        let oracle = Address::generate(&env);
        set_oracle_price(&client, oracle, usdc.clone(), 100_000_000, 1234567890);

        // Attempt to mint 100 stablecoins (should fail)
        let mint_amount = 100_000;
        client.mint(&user, &mint_amount, &usdc);
    }

    // Test minting above max_mint_amount
    #[test]
    #[should_panic(expected = "Mint amount exceeds collateral-backed limit")]
    fn test_mint_exceeds_max_mint_amount_fails() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _, usdc) = setup_contract(&env);
        let user = Address::generate(&env);

        // Deposit sufficient collateral (1.5B USDC for 1B stablecoins)
        let collateral_amount = 1_500_000_000;
        client.deposit_collateral(&user, &usdc, &collateral_amount);

        // Set oracle price
        let oracle = Address::generate(&env);
        set_oracle_price(&client, oracle, usdc.clone(), 100_000_000, 1234567890);

        // Attempt to mint above max_mint_amount (1B + 1)
        let mint_amount = 1_000_000_001;
        client.mint(&user, &mint_amount, &usdc);
    }

    // Test minting with zero amount
    #[test]
    #[should_panic]
    fn test_mint_zero_amount_fails() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _, usdc) = setup_contract(&env);
        let user = Address::generate(&env);

        // Deposit collateral
        client.deposit_collateral(&user, &usdc, &150_000);

        // Attempt to mint 0 stablecoins
        client.mint(&user, &0, &usdc);
    }

    // Test burning with zero amount
    #[test]
    #[should_panic]
    fn test_burn_zero_amount_fails() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _, usdc) = setup_contract(&env);
        let user = Address::generate(&env);

        // Attempt to burn 0 stablecoins
        client.burn(&user, &0, &usdc);
    }

    // Test burning stablecoins and releasing collateral
    #[test]
    fn test_burn_and_collateral_release() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, contract_id, usdc) = setup_contract(&env);
        let user = Address::generate(&env);

        // Deposit collateral and mint
        let collateral_amount = 150_000;
        let mint_amount = 100_000;
        client.deposit_collateral(&user, &usdc, &collateral_amount);
        let oracle = Address::generate(&env);
        set_oracle_price(&client, oracle, usdc.clone(), 100_000_000, 1234567890);
        client.mint(&user, &mint_amount, &usdc);

        // Verify initial state
        let initial_balance = env.as_contract(&contract_id, || {
            env.storage()
                .instance()
                .get::<_, i128>(&BurnDataKey::StablecoinBalance(user.clone()))
                .unwrap_or(0)
        });
        assert_eq!(
            initial_balance, mint_amount,
            "Initial stablecoin balance should match minted amount"
        );

        // Burn 50 stablecoins
        let burn_amount = 50_000;
        client.burn(&user, &burn_amount, &usdc);

        // Verify stablecoin balance
        let balance = env.as_contract(&contract_id, || {
            env.storage()
                .instance()
                .get::<_, i128>(&BurnDataKey::StablecoinBalance(user.clone()))
                .unwrap_or(0)
        });
        assert_eq!(
            balance,
            mint_amount - burn_amount,
            "Stablecoin balance should reflect burn"
        );

        // Verify collateral returned
        let params = client.get_parameters();
        let expected_collateral_return =
            (burn_amount as u128 * params.min_collateral_ratio as u128 / 10_000) as i128;
        let user_collateral = client.get_user_collateral(&user, &usdc);
        assert_eq!(
            user_collateral,
            collateral_amount + expected_collateral_return,
            "Collateral should reflect returned amount"
        );
    }

    // Test governance proposal creation, voting, and execution
    #[test]
    fn test_governance_proposal_and_voting() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, contract_id, _) = setup_contract(&env);
        let proposer = Address::generate(&env);
        let voter1 = Address::generate(&env);
        let voter2 = Address::generate(&env);

        // Create proposal to change min_collateral_ratio
        let action_type = String::from_str(&env, "adjust_parameters");
        let parameters = vec![&env, String::from_str(&env, "min_collateral_ratio:17500")];
        let proposal_id = client.propose_governance_action(&proposer, &action_type, &parameters);
        assert_eq!(proposal_id, 1, "Proposal ID should be 1");

        // Vote on proposal
        client.vote(&voter1, &proposal_id, &Vote::Yes);
        client.vote(&voter2, &proposal_id, &Vote::Yes);

        // Execute proposal
        client.execute_proposal(&proposer, &proposal_id);

        // Verify proposal status
        let proposal: GovernanceProposal = env.as_contract(&contract_id, || {
            env.storage()
                .instance()
                .get(&GovernanceDataKey::Proposal(proposal_id))
                .unwrap()
        });
        assert_eq!(
            proposal.status,
            ProposalStatus::Approved,
            "Proposal should be approved"
        );
    }

    // Test governance proposal rejection
    #[test]
    fn test_governance_proposal_rejected() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, contract_id, _) = setup_contract(&env);
        let proposer = Address::generate(&env);
        let voter1 = Address::generate(&env);
        let voter2 = Address::generate(&env);

        // Create proposal
        let action_type = String::from_str(&env, "adjust_parameters");
        let parameters = vec![&env, String::from_str(&env, "min_collateral_ratio:17500")];
        let proposal_id = client.propose_governance_action(&proposer, &action_type, &parameters);

        // Vote: one Yes, one No
        client.vote(&voter1, &proposal_id, &Vote::Yes);
        client.vote(&voter2, &proposal_id, &Vote::No);

        // Execute proposal
        client.execute_proposal(&proposer, &proposal_id);

        // Verify proposal is rejected
        let proposal: GovernanceProposal = env.as_contract(&contract_id, || {
            env.storage()
                .instance()
                .get(&GovernanceDataKey::Proposal(proposal_id))
                .unwrap()
        });
        assert_eq!(
            proposal.status,
            ProposalStatus::Rejected,
            "Proposal should be rejected"
        );
    }

    // Test governance double voting
    #[test]
    #[should_panic(expected = "Already voted")]
    fn test_governance_double_voting_fails() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _, _) = setup_contract(&env);
        let proposer = Address::generate(&env);
        let voter = Address::generate(&env);

        // Create proposal
        let action_type = String::from_str(&env, "adjust_parameters");
        let parameters = vec![&env, String::from_str(&env, "min_collateral_ratio:17500")];
        let proposal_id = client.propose_governance_action(&proposer, &action_type, &parameters);

        // Vote once
        client.vote(&voter, &proposal_id, &Vote::Yes);

        // Attempt to vote again
        client.vote(&voter, &proposal_id, &Vote::Yes);
    }

    // Test withdrawing collateral
    #[test]
    fn test_withdraw_collateral() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _, usdc) = setup_contract(&env);
        let user = Address::generate(&env);

        // Deposit collateral
        let collateral_amount = 150_000;
        client.deposit_collateral(&user, &usdc, &collateral_amount);

        // Withdraw 50,000
        let withdraw_amount = 50_000;
        client.withdraw_collateral(&user, &usdc, &withdraw_amount);

        // Verify user collateral
        let user_collateral = client.get_user_collateral(&user, &usdc);
        assert_eq!(
            user_collateral,
            collateral_amount - withdraw_amount,
            "User collateral should reflect withdrawal"
        );

        // Verify reserves
        let total_reserve = client.get_total_reserve(&usdc);
        assert_eq!(
            total_reserve,
            collateral_amount - withdraw_amount,
            "Total reserve should reflect withdrawal"
        );
    }
}
