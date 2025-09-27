use soroban_sdk::{contracttype, symbol_short, Address, Env, String, Symbol, Vec};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DonationStatus {
    Pending,    // Donation created but not funded
    Funded,     // Funds deposited, waiting for milestone verification
    InProgress, // Some milestones verified, more pending
    Completed,  // All milestones verified, funds released
    Refunded,   // Donation refunded to donor
    Paused,     // Temporarily paused by admin
    Cancelled,  // Donation cancelled
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Donation {
    pub id: u64,
    pub donor: Address,
    pub recipient: Address,
    pub amount: i128,
    pub token: Symbol,
    pub description: String,
    pub milestones: Vec<String>,
    pub status: DonationStatus,
    pub created_at: u64,
    pub funded_at: Option<u64>,
    pub completed_at: Option<u64>,
    pub refunded_at: Option<u64>,
    pub admin: Address,
}

// Storage keys
const DONATIONS: Symbol = symbol_short!("DONATIONS");
const USER_DONATIONS: Symbol = symbol_short!("USER_DON");
pub const ADMIN: Symbol = symbol_short!("ADMIN");

pub fn initialize(
    env: &Env,
    donation_id: u64,
    donor: Address,
    recipient: Address,
    amount: i128,
    token: Symbol,
    description: String,
    milestones: Vec<String>,
) {
    // Validate inputs
    if amount <= 0 {
        panic!("Invalid amount");
    }

    if milestones.len() == 0 {
        panic!("No milestones provided");
    }

    // Check if donation already exists
    if get_donation(env, donation_id).is_some() {
        panic!("Donation already exists");
    }

    // Set admin if not set
    if env
        .storage()
        .instance()
        .get::<Symbol, Address>(&ADMIN)
        .is_none()
    {
        env.storage().instance().set(&ADMIN, &donor); // First donor becomes admin
    }

    let admin: Address = env
        .storage()
        .instance()
        .get::<Symbol, Address>(&ADMIN)
        .unwrap_or_else(|| panic!("No admin set"));

    let donation = Donation {
        id: donation_id,
        donor: donor.clone(),
        recipient,
        amount,
        token,
        description,
        milestones,
        status: DonationStatus::Pending,
        created_at: env.ledger().timestamp(),
        funded_at: None,
        completed_at: None,
        refunded_at: None,
        admin,
    };

    // Store donation
    env.storage()
        .persistent()
        .set(&storage_key_donation(donation_id), &donation);

    // Add to user's donation list
    add_user_donation(env, &donor, donation_id);
}

pub fn deposit(env: &Env, donation_id: u64, donor: Address, amount: i128, token: Symbol) {
    let mut donation =
        get_donation(env, donation_id).unwrap_or_else(|| panic!("Donation not found"));

    // Validate donation is in correct state
    if donation.status != DonationStatus::Pending {
        panic!("Invalid donation status");
    }

    // Validate donor matches
    if donation.donor != donor {
        panic!("Unauthorized donor");
    }

    // Validate amount and token match
    if donation.amount != amount || donation.token != token {
        panic!("Amount or token mismatch");
    }

    // Update donation status
    donation.status = DonationStatus::Funded;
    donation.funded_at = Some(env.ledger().timestamp());

    // Store updated donation
    env.storage()
        .persistent()
        .set(&storage_key_donation(donation_id), &donation);
}

pub fn get_donation(env: &Env, donation_id: u64) -> Option<Donation> {
    env.storage()
        .persistent()
        .get(&storage_key_donation(donation_id))
}

pub fn get_donation_status(env: &Env, donation_id: u64) -> Option<DonationStatus> {
    get_donation(env, donation_id).map(|d| d.status)
}

pub fn get_user_donations(env: &Env, user: Address) -> Vec<u64> {
    env.storage()
        .persistent()
        .get(&storage_key_user_donations(&user))
        .unwrap_or_else(|| Vec::new(&env))
}

pub fn pause_donation(env: &Env, donation_id: u64, admin: Address) {
    // Verify admin
    let stored_admin = env.storage().instance().get::<Symbol, Address>(&ADMIN);
    if stored_admin.is_none() {
        panic!("No admin set");
    }
    let stored_admin: Address = stored_admin.unwrap();

    if stored_admin != admin {
        panic!("Unauthorized admin");
    }

    let mut donation =
        get_donation(env, donation_id).unwrap_or_else(|| panic!("Donation not found"));

    if donation.status == DonationStatus::Paused {
        panic!("Donation already paused");
    }

    donation.status = DonationStatus::Paused;
    env.storage()
        .persistent()
        .set(&storage_key_donation(donation_id), &donation);
}

pub fn resume_donation(env: &Env, donation_id: u64, admin: Address) {
    // Verify admin
    let stored_admin = env.storage().instance().get::<Symbol, Address>(&ADMIN);
    if stored_admin.is_none() {
        panic!("No admin set");
    }
    let stored_admin: Address = stored_admin.unwrap();

    if stored_admin != admin {
        panic!("Unauthorized admin");
    }

    let mut donation =
        get_donation(env, donation_id).unwrap_or_else(|| panic!("Donation not found"));

    if donation.status != DonationStatus::Paused {
        panic!("Donation not paused");
    }

    // Resume to previous status based on funding state
    if donation.funded_at.is_some() {
        donation.status = DonationStatus::Funded;
    } else {
        donation.status = DonationStatus::Pending;
    }

    env.storage()
        .persistent()
        .set(&storage_key_donation(donation_id), &donation);
}

fn add_user_donation(env: &Env, user: &Address, donation_id: u64) {
    let mut user_donations = get_user_donations(env, user.clone());
    user_donations.push_back(donation_id);
    env.storage()
        .persistent()
        .set(&storage_key_user_donations(user), &user_donations);
}

pub fn storage_key_donation(donation_id: u64) -> (Symbol, u64) {
    (DONATIONS, donation_id)
}

fn storage_key_user_donations(user: &Address) -> (Symbol, Address) {
    (USER_DONATIONS, user.clone())
}
