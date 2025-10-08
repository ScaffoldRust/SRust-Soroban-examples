#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, String, Symbol, Vec};

#[contracttype]
pub enum DataKey {
    Admin,
    Roles(Address),           // Maps address to role
    BatchEvents(Symbol),      // Maps batch ID to events
    BatchStatus(Symbol),      // Maps batch ID to current status
    AuthorizedEntities(Symbol), // Maps role to authorized addresses
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Role {
    Manufacturer,
    Distributor,
    Wholesaler,
    Pharmacy,
    Hospital,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Status {
    Created,
    InTransit,
    Received,
    Quarantined,
    Approved,
    Rejected,
    Dispensed,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct Event {
    timestamp: u64,
    entity: Address,
    event_type: Symbol,
    location: String,
    status: Status,
    metadata: Vec<String>,
}

#[contract]
pub struct PharmaceuticalSupplyChain;

#[contractimpl]
impl PharmaceuticalSupplyChain {
    pub fn initialize(env: Env, admin: Address) {
        admin.require_auth();

        if env.storage().instance().has(&DataKey::Admin) {
            panic!("contract already initialized");
        }

        env.storage().instance().set(&DataKey::Admin, &admin);
    }

    pub fn assign_role(env: Env, admin: Address, entity: Address, role: Role) {
        admin.require_auth();
        
        // Verify admin
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic!("only admin can assign roles");
        }

        // Store role assignment
        env.storage().instance().set(&DataKey::Roles(entity.clone()), &role);

        // Add to authorized entities for this role
        let role_symbol = match role {
            Role::Manufacturer => Symbol::new(&env, "MANUFACTURER"),
            Role::Distributor => Symbol::new(&env, "DISTRIBUTOR"),
            Role::Wholesaler => Symbol::new(&env, "WHOLESALER"),
            Role::Pharmacy => Symbol::new(&env, "PHARMACY"),
            Role::Hospital => Symbol::new(&env, "HOSPITAL"),
        };

        let mut authorized = env.storage().instance().get(&DataKey::AuthorizedEntities(role_symbol.clone()))
            .unwrap_or(Vec::new(&env));
        authorized.push_back(entity);
        env.storage().instance().set(&DataKey::AuthorizedEntities(role_symbol), &authorized);
    }

    pub fn get_role(env: Env, entity: Address) -> Option<Role> {
        env.storage().instance().get(&DataKey::Roles(entity))
    }

    pub fn is_authorized(env: Env, entity: Address, role: Role) -> bool {
        if let Some(assigned_role) = Self::get_role(env.clone(), entity.clone()) {
            assigned_role == role
        } else {
            false
        }
    }

    // Supply chain tracking methods
    pub fn log_event(
        env: Env,
        entity: Address,
        batch_id: Symbol,
        event_type: Symbol,
        location: String,
        status: Status,
        metadata: Vec<String>,
    ) {
        entity.require_auth();

        // Validate entity's role based on the event type
        let required_role = {
            let manufacture = Symbol::new(&env, "MANUFACTURE");
            let manufacture_create = Symbol::new(&env, "MANUFACTURE_CREATE");
            let distribute_ship = Symbol::new(&env, "DISTRIBUTE_SHIP");
            let wholesale = Symbol::new(&env, "WHOLESALE");
            let pharmacy_sym = Symbol::new(&env, "PHARMACY");
            let hospital_sym = Symbol::new(&env, "HOSPITAL");

            if event_type == manufacture || event_type == manufacture_create {
                Role::Manufacturer
            } else if event_type == distribute_ship {
                Role::Distributor
            } else if event_type == wholesale {
                Role::Wholesaler
            } else if event_type == pharmacy_sym {
                Role::Pharmacy
            } else if event_type == hospital_sym {
                Role::Hospital
            } else {
                panic!("invalid event type");
            }
        };

        if !Self::is_authorized(env.clone(), entity.clone(), required_role) {
            panic!("entity not authorized for this operation");
        }

        // Create and store the event
        let event = Event {
            timestamp: env.ledger().timestamp(),
            entity,
            event_type,
            location,
            status: status.clone(),
            metadata,
        };

        // Get existing events or create new vec
        let mut events = env
            .storage()
            .instance()
            .get(&DataKey::BatchEvents(batch_id.clone()))
            .unwrap_or(Vec::new(&env));

        // Add new event
        events.push_back(event);

        // Update batch events and status
        env.storage()
            .instance()
            .set(&DataKey::BatchEvents(batch_id.clone()), &events);
        env.storage()
            .instance()
            .set(&DataKey::BatchStatus(batch_id), &status);
    }

    pub fn get_batch_history(env: Env, batch_id: Symbol) -> Option<Vec<Event>> {
        env.storage().instance().get(&DataKey::BatchEvents(batch_id))
    }

    pub fn get_batch_status(env: Env, batch_id: Symbol) -> Option<Status> {
        env.storage().instance().get(&DataKey::BatchStatus(batch_id))
    }

    pub fn verify_batch(env: Env, batch_id: Symbol) -> bool {
        // Get batch history
        let events = match Self::get_batch_history(env.clone(), batch_id) {
            Some(e) => e,
            None => return false,
        };

        if events.is_empty() {
            return false;
        }

        // Verify event sequence
        let mut last_status: Option<Status> = None;
        for event in events.iter() {
            match (&last_status, event.status.clone()) {
                (None, Status::Created) => {
                    last_status = Some(event.status.clone());
                }
                (Some(Status::Created), Status::InTransit)
                | (Some(Status::InTransit), Status::Received)
                | (Some(Status::Received), Status::Quarantined)
                | (Some(Status::Quarantined), Status::Approved)
                | (Some(Status::Approved), Status::Dispensed)
                | (Some(_), Status::Rejected) => {
                    last_status = Some(event.status.clone());
                }
                _ => return false,
            }
        }

        true
    }

    // Stage management methods
    pub fn create_batch(
        env: Env,
        manufacturer: Address,
        batch_id: Symbol,
        location: String,
        metadata: Vec<String>,
    ) {
        let event_type = Symbol::new(&env, "MANUFACTURE_CREATE");
        Self::log_event(
            env,
            manufacturer,
            batch_id,
            event_type,
            location,
            Status::Created,
            metadata,
        )
    }

    pub fn ship_batch(
        env: Env,
        distributor: Address,
        batch_id: Symbol,
        location: String,
        metadata: Vec<String>,
    ) {
        let event_type = Symbol::new(&env, "DISTRIBUTE_SHIP");
        Self::log_event(
            env,
            distributor,
            batch_id,
            event_type,
            location,
            Status::InTransit,
            metadata,
        )
    }

    pub fn receive_batch(
        env: Env,
        receiver: Address,
        batch_id: Symbol,
        location: String,
        metadata: Vec<String>,
    ) {
        let event_type = Symbol::new(&env, "RECEIVE");
        Self::log_event(
            env,
            receiver,
            batch_id,
            event_type,
            location,
            Status::Received,
            metadata,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env};

    #[test]
    fn test_initialize() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(PharmaceuticalSupplyChain, ());
        let admin = Address::generate(&env);
        
        let client = PharmaceuticalSupplyChainClient::new(&env, &contract_id);
        
        client.initialize(&admin);
        
        // Verify admin was set
        env.as_contract(&contract_id, || {
            let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
            assert_eq!(stored_admin, admin);
        });
    }

    #[test]
    fn test_role_assignment() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(PharmaceuticalSupplyChain, ());
        let admin = Address::generate(&env);
        let entity = Address::generate(&env);
        
        let client = PharmaceuticalSupplyChainClient::new(&env, &contract_id);
        
        client.initialize(&admin);
        client.assign_role(&admin, &entity, &Role::Manufacturer);
        
        let role = client.get_role(&entity).unwrap();
        assert!(matches!(role, Role::Manufacturer));
        
        assert!(client.is_authorized(&entity, &Role::Manufacturer));
    }
}
