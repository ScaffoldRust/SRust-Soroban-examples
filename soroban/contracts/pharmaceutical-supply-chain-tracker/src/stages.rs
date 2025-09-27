use soroban_sdk::{Address, Env, String, Symbol, Vec};
use crate::{contractimpl, tracker::SupplyChainTracker, DataKey, Event, Role, Status, PharmaceuticalSupplyChain};

pub trait StageManagement {
    fn create_batch(
        env: Env,
        manufacturer: Address,
        batch_id: Symbol,
        location: String,
        metadata: Vec<String>,
    );

    fn ship_batch(
        env: Env,
        distributor: Address,
        batch_id: Symbol,
        location: String,
        metadata: Vec<String>,
    );

    fn receive_batch(
        env: Env,
        receiver: Address,
        batch_id: Symbol,
        location: String,
        metadata: Vec<String>,
    );

    fn quarantine_batch(
        env: Env,
        entity: Address,
        batch_id: Symbol,
        location: String,
        metadata: Vec<String>,
    );

    fn approve_batch(
        env: Env,
        entity: Address,
        batch_id: Symbol,
        location: String,
        metadata: Vec<String>,
    );

    fn reject_batch(
        env: Env,
        entity: Address,
        batch_id: Symbol,
        location: String,
        metadata: Vec<String>,
    );

    fn dispense_batch(
        env: Env,
        pharmacy: Address,
        batch_id: Symbol,
        location: String,
        metadata: Vec<String>,
    );
}

#[contractimpl]
impl StageManagement for PharmaceuticalSupplyChain {
    fn create_batch(
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

    fn ship_batch(
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

    fn receive_batch(
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

    fn quarantine_batch(
        env: Env,
        entity: Address,
        batch_id: Symbol,
        location: String,
        metadata: Vec<String>,
    ) {
        let event_type = Symbol::new(&env, "QUARANTINE");
        Self::log_event(
            env,
            entity,
            batch_id,
            event_type,
            location,
            Status::Quarantined,
            metadata,
        )
    }

    fn approve_batch(
        env: Env,
        entity: Address,
        batch_id: Symbol,
        location: String,
        metadata: Vec<String>,
    ) {
        let event_type = Symbol::new(&env, "APPROVE");
        Self::log_event(
            env,
            entity,
            batch_id,
            event_type,
            location,
            Status::Approved,
            metadata,
        )
    }

    fn reject_batch(
        env: Env,
        entity: Address,
        batch_id: Symbol,
        location: String,
        metadata: Vec<String>,
    ) {
        let event_type = Symbol::new(&env, "REJECT");
        Self::log_event(
            env,
            entity,
            batch_id,
            event_type,
            location,
            Status::Rejected,
            metadata,
        )
    }

    fn dispense_batch(
        env: Env,
        pharmacy: Address,
        batch_id: Symbol,
        location: String,
        metadata: Vec<String>,
    ) {
        let event_type = Symbol::new(&env, "DISPENSE");
        Self::log_event(
            env,
            pharmacy,
            batch_id,
            event_type,
            location,
            Status::Dispensed,
            metadata,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::Address as _;

    #[test]
    fn test_batch_lifecycle() {
        let env = Env::default();
        let admin = Address::generate(&env);
        let manufacturer = Address::generate(&env);
        let distributor = Address::generate(&env);
        let pharmacy = Address::generate(&env);
        
        // Initialize contract and assign roles
        PharmaceuticalSupplyChain::initialize(&env, admin.clone());
        PharmaceuticalSupplyChain::assign_role(&env, admin.clone(), manufacturer.clone(), Role::Manufacturer);
        PharmaceuticalSupplyChain::assign_role(&env, admin.clone(), distributor.clone(), Role::Distributor);
        PharmaceuticalSupplyChain::assign_role(&env, admin, pharmacy.clone(), Role::Pharmacy);
        
        let batch_id = Symbol::new(&env, "BATCH001");
        let location = String::from_str(&env, "Factory 1");
        let metadata = Vec::from_array(&env, [String::from_str(&env, "Batch Info")]);
        
        // Test complete lifecycle
        PharmaceuticalSupplyChain::create_batch(
            env.clone(),
            manufacturer,
            batch_id.clone(),
            location.clone(),
            metadata.clone(),
        );
        
        PharmaceuticalSupplyChain::ship_batch(
            env.clone(),
            distributor.clone(),
            batch_id.clone(),
            location.clone(),
            metadata.clone(),
        );
        
        PharmaceuticalSupplyChain::receive_batch(
            env.clone(),
            pharmacy.clone(),
            batch_id.clone(),
            location.clone(),
            metadata.clone(),
        );
        
        PharmaceuticalSupplyChain::quarantine_batch(
            env.clone(),
            pharmacy.clone(),
            batch_id.clone(),
            location.clone(),
            metadata.clone(),
        );
        
        PharmaceuticalSupplyChain::approve_batch(
            env.clone(),
            pharmacy.clone(),
            batch_id.clone(),
            location.clone(),
            metadata.clone(),
        );
        
        PharmaceuticalSupplyChain::dispense_batch(
            env.clone(),
            pharmacy,
            batch_id.clone(),
            location,
            metadata,
        );
        
        // Verify final status
        let status = PharmaceuticalSupplyChain::get_batch_status(env.clone(), batch_id.clone()).unwrap();
        assert_eq!(status, Status::Dispensed);
        
        // Verify chain integrity
        assert!(PharmaceuticalSupplyChain::verify_batch(env, batch_id));
    }
}