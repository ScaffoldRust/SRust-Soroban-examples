#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Address, BytesN, Env, String};

mod admin;
mod compliance;
mod issuance;
mod token;
mod transfer;

pub use admin::*;
pub use compliance::*;
pub use issuance::*;
pub use token::*;
pub use transfer::*;

#[derive(Clone)]
#[contracttype]
pub struct AssetMetadata {
    pub name: String,
    pub symbol: String,
    pub decimals: u32,
    pub asset_type: String,
}

#[derive(Clone)]
#[contracttype]
pub struct RegulatoryInfo {
    pub compliance_doc_hashes: BytesN<32>,
    pub jurisdiction: String,
}

#[derive(Clone)]
#[contracttype]
pub enum ComplianceStatus {
    Approved,
    Pending,
    Rejected,
}

#[derive(Clone)]
#[contracttype]
pub struct TokenInfo {
    pub metadata: AssetMetadata,
    pub regulatory_info: RegulatoryInfo,
    pub total_supply: i128,
    pub issuer: Address,
}

#[contract]
pub struct AssetTokenizationContract;

#[contractimpl]
impl AssetTokenizationContract {
    pub fn initialize(env: Env, admin: Address) {
        admin::initialize(&env, &admin);
    }

    // Tokenization functions
    pub fn tokenize(
        env: Env,
        issuer: Address,
        asset_metadata: AssetMetadata,
        regulatory_info: RegulatoryInfo,
        initial_amount: i128,
    ) -> u64 {
        issuance::tokenize(
            &env,
            &issuer,
            asset_metadata,
            regulatory_info,
            initial_amount,
        )
    }

    pub fn redeem(env: Env, caller: Address, token_id: u64, amount: i128) -> bool {
        issuance::redeem(&env, &caller, token_id, amount)
    }

    // Transfer functions
    pub fn transfer(env: Env, from: Address, to: Address, token_id: u64, amount: i128) -> bool {
        transfer::transfer(&env, &from, &to, token_id, amount)
    }

    pub fn approve(
        env: Env,
        owner: Address,
        spender: Address,
        token_id: u64,
        amount: i128,
    ) -> bool {
        transfer::approve(&env, &owner, &spender, token_id, amount)
    }

    pub fn transfer_from(
        env: Env,
        spender: Address,
        owner: Address,
        receiver: Address,
        token_id: u64,
        amount: i128,
    ) -> bool {
        transfer::transfer_from(&env, &spender, &owner, &receiver, token_id, amount)
    }

    // Compliance functions
    pub fn verify_compliance(env: Env, token_id: u64, user: Address) -> ComplianceStatus {
        compliance::verify_compliance(&env, token_id, &user)
    }

    pub fn update_compliance_status(
        env: Env,
        admin: Address,
        token_id: u64,
        user: Address,
        status: ComplianceStatus,
    ) -> bool {
        compliance::update_compliance_status(&env, &admin, token_id, &user, status)
    }

    // Admin functions
    pub fn update_asset_details(
        env: Env,
        admin: Address,
        token_id: u64,
        new_metadata: AssetMetadata,
    ) -> bool {
        admin::update_asset_details(&env, &admin, token_id, new_metadata)
    }

    pub fn freeze_account(env: Env, admin: Address, account: Address, reason: String) -> bool {
        admin::freeze_account(&env, &admin, &account, reason)
    }

    pub fn unfreeze_account(env: Env, admin: Address, account: Address) -> bool {
        admin::unfreeze_account(&env, &admin, &account)
    }

    // Query functions
    pub fn balance_of(env: Env, address: Address, token_id: u64) -> i128 {
        token::balance_of(&env, &address, token_id)
    }

    pub fn allowance(env: Env, owner: Address, spender: Address, token_id: u64) -> i128 {
        token::allowance(&env, &owner, &spender, token_id)
    }

    pub fn token_info(env: Env, token_id: u64) -> Option<TokenInfo> {
        token::token_info(&env, token_id)
    }

    pub fn is_frozen(env: Env, account: Address) -> bool {
        admin::is_frozen(&env, &account)
    }
}
mod test;
