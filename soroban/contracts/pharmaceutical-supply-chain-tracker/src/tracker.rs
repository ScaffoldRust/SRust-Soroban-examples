use soroban_sdk::{Address, Env, String, Symbol, Vec};
use crate::{contractimpl, DataKey, Event, Role, Status, PharmaceuticalSupplyChain};

pub trait SupplyChainTracker {
    fn log_event(
        env: Env,
        entity: Address,
        batch_id: Symbol,
        event_type: Symbol,
        location: String,
        status: Status,
        metadata: Vec<String>,
    );

    fn get_batch_history(env: Env, batch_id: Symbol) -> Option<Vec<Event>>;
    fn get_batch_status(env: Env, batch_id: Symbol) -> Option<Status>;
    fn verify_batch(env: Env, batch_id: Symbol) -> bool;
}

#[contractimpl]
impl SupplyChainTracker for PharmaceuticalSupplyChain {
    fn log_event(
        env: Env,
        entity: Address,
        batch_id: Symbol,
        event_type: Symbol,
        location: String,
        status: Status,
        metadata: Vec<String>,
    ) {
        // Require authorization from the entity
        entity.require_auth();

        // Validate entity's role based on the event type by comparing symbols
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

        if !PharmaceuticalSupplyChain::is_authorized(env.clone(), entity.clone(), required_role) {
            panic!("entity not authorized for this operation");
        }

        // Create and store the event
        let event = Event {
            timestamp: env.ledger().timestamp(),
            entity,
            event_type,
            location,
            status,
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

    fn get_batch_history(env: Env, batch_id: Symbol) -> Option<Vec<Event>> {
        env.storage().instance().get(&DataKey::BatchEvents(batch_id))
    }

    fn get_batch_status(env: Env, batch_id: Symbol) -> Option<Status> {
        env.storage().instance().get(&DataKey::BatchStatus(batch_id))
    }

    fn verify_batch(env: Env, batch_id: Symbol) -> bool {
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
                    // First event must be Created
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::Address as _;

    #[test]
    fn test_event_logging() {
        let env = Env::default();
        let admin = Address::generate(&env);
        let manufacturer = Address::generate(&env);
        
        // Initialize contract and assign role
        PharmaceuticalSupplyChain::initialize(&env, admin.clone());
        PharmaceuticalSupplyChain::assign_role(&env, admin, manufacturer.clone(), Role::Manufacturer);
        
        // Create batch and log event
        let batch_id = Symbol::new(&env, "BATCH001");
        let event_type = Symbol::new(&env, "MANUFACTURE");
        let location = String::from_str(&env, "Factory 1");
        let metadata = Vec::from_array(&env, [String::from_str(&env, "QA Passed")]);
        
        PharmaceuticalSupplyChain::log_event(
            env.clone(),
            manufacturer.clone(),
            batch_id.clone(),
            event_type,
            location,
            Status::Created,
            metadata,
        );
        
        // Verify batch history
        let history = PharmaceuticalSupplyChain::get_batch_history(env.clone(), batch_id).unwrap();
        assert_eq!(history.len(), 1);
        
        let event = history.get(0).unwrap();
        assert_eq!(event.entity, manufacturer);
        assert_eq!(event.status, Status::Created);
    }
}
