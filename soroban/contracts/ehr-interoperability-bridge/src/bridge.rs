use soroban_sdk::{Address, BytesN, Env, Map, String, Vec, Symbol, Bytes};
use crate::{DataKey, DataRequest, DataTransfer, EhrSystem, RequestStatus};

/// Register a new EHR system in the bridge
pub fn register_ehr_system(
    env: &Env,
    admin: Address,
    system_id: String,
    name: String,
    endpoint: String,
    supported_formats: Vec<String>,
    public_key: BytesN<32>,
    system_admin: Address,
) -> bool {
    // Verify admin authorization
    let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
    if admin != stored_admin {
        admin.require_auth();
    }

    // Create EHR system record
    let ehr_system = EhrSystem {
        system_id: system_id.clone(),
        name,
        endpoint,
        supported_formats,
        public_key,
        admin: system_admin,
        is_active: true,
    };

    // Store the system
    env.storage().persistent().set(&DataKey::EhrSystem(system_id.clone()), &ehr_system);

    // Update system registry
    let mut registry: Map<String, EhrSystem> = env.storage()
        .instance()
        .get(&DataKey::SystemRegistry)
        .unwrap_or(Map::new(env));
    
    registry.set(system_id.clone(), ehr_system);
    env.storage().instance().set(&DataKey::SystemRegistry, &registry);

    // Emit registration event
    env.events().publish(
        (Symbol::new(env, "ehr_registered"),),
        (system_id, admin),
    );

    true
}

/// Request data from another EHR system
pub fn request_data(
    env: &Env,
    requester: Address,
    sender_system: String,
    receiver_system: String,
    patient_id: String,
    data_types: Vec<String>,
    expiry_hours: u64,
) -> BytesN<32> {
    requester.require_auth();

    // Verify both systems exist and are active
    let sender = get_ehr_system(env, sender_system.clone());
    let receiver = get_ehr_system(env, receiver_system.clone());
    
    if sender.is_none() || receiver.is_none() {
        panic!("EHR system not found or inactive");
    }

    let sender_sys = sender.unwrap();
    let receiver_sys = receiver.unwrap();

    if !sender_sys.is_active || !receiver_sys.is_active {
        panic!("One or both EHR systems are inactive");
    }

    // Generate unique request ID
    let mut request_counter: u64 = env.storage()
        .instance()
        .get(&DataKey::NextRequestId)
        .unwrap_or(1);
    
    let request_id_bytes = Bytes::from_array(env, &request_counter.to_be_bytes());
    let request_id: BytesN<32> = env.crypto().sha256(&request_id_bytes).into();
    request_counter += 1;
    env.storage().instance().set(&DataKey::NextRequestId, &request_counter);

    // Create data request
    let current_time = env.ledger().timestamp();
    let expiry_time = current_time + (expiry_hours * 3600);

    let data_request = DataRequest {
        request_id: request_id.clone(),
        sender_system,
        receiver_system,
        patient_id,
        data_types,
        requester: requester.clone(),
        consent_verified: false,
        status: RequestStatus::Pending,
        timestamp: current_time,
        expiry: expiry_time,
    };

    // Store the request
    env.storage().persistent().set(&DataKey::DataRequest(request_id.clone()), &data_request);

    // Add to request queue
    let mut queue: Vec<BytesN<32>> = env.storage()
        .instance()
        .get(&DataKey::RequestQueue)
        .unwrap_or(Vec::new(env));
    
    queue.push_back(request_id.clone());
    env.storage().instance().set(&DataKey::RequestQueue, &queue);

    // Emit request event
    env.events().publish(
        (Symbol::new(env, "data_requested"),),
        (request_id.clone(), requester),
    );

    request_id
}

/// Transfer data after consent and validation
pub fn transfer_data(
    env: &Env,
    request_id: BytesN<32>,
    data_hash: BytesN<32>,
    source_format: String,
    target_format: String,
    validator: Address,
) -> BytesN<32> {
    validator.require_auth();

    // Get and validate the request
    let mut request: DataRequest = env.storage()
        .persistent()
        .get(&DataKey::DataRequest(request_id.clone()))
        .expect("Request not found");

    // Check if request is valid for transfer
    if !request.consent_verified {
        panic!("Patient consent not verified");
    }

    if request.status != RequestStatus::ConsentVerified && request.status != RequestStatus::Approved {
        panic!("Request not approved for transfer");
    }

    if env.ledger().timestamp() > request.expiry {
        panic!("Request has expired");
    }

    // Generate transfer ID (simplified)
    let simple_data = Bytes::from_slice(env, b"transfer_id_data");
    let transfer_id: BytesN<32> = env.crypto().sha256(&simple_data).into();

    // Create transfer record
    let data_transfer = DataTransfer {
        transfer_id: transfer_id.clone(),
        request_id: request_id.clone(),
        data_hash,
        source_format,
        target_format,
        transfer_timestamp: env.ledger().timestamp(),
        validator: validator.clone(),
    };

    // Store the transfer
    env.storage().persistent().set(&DataKey::DataTransfer(transfer_id.clone()), &data_transfer);

    // Update request status
    request.status = RequestStatus::Completed;
    env.storage().persistent().set(&DataKey::DataRequest(request_id.clone()), &request);

    // Emit transfer event
    env.events().publish(
        (Symbol::new(env, "data_transferred"),),
        (transfer_id.clone(), request_id, validator),
    );

    transfer_id
}

/// Log successful data exchange for auditing
pub fn log_exchange(env: &Env, transfer_id: BytesN<32>, additional_metadata: String) {
    let transfer: DataTransfer = env.storage()
        .persistent()
        .get(&DataKey::DataTransfer(transfer_id.clone()))
        .expect("Transfer not found");

    let request: DataRequest = env.storage()
        .persistent()
        .get(&DataKey::DataRequest(transfer.request_id.clone()))
        .expect("Request not found");

    // Create audit log entry
    let audit_entry = (
        transfer_id.clone(),
        transfer.request_id.clone(),
        request.sender_system,
        request.receiver_system,
        request.patient_id,
        transfer.transfer_timestamp,
        additional_metadata,
    );

    // Store audit log
    let audit_id_bytes = Bytes::from_array(env, &transfer_id.to_array());
    let audit_id = env.crypto().sha256(&audit_id_bytes).into();
    env.storage().persistent().set(&DataKey::AuditLog(audit_id), &audit_entry);

    // Emit audit event
    env.events().publish(
        (Symbol::new(env, "exchange_logged"),),
        audit_entry,
    );
}

/// Get EHR system information
pub fn get_ehr_system(env: &Env, system_id: String) -> Option<EhrSystem> {
    env.storage().persistent().get(&DataKey::EhrSystem(system_id))
}

/// Get data request details
pub fn get_data_request(env: &Env, request_id: BytesN<32>) -> Option<DataRequest> {
    env.storage().persistent().get(&DataKey::DataRequest(request_id))
}

/// Get data transfer details
pub fn get_data_transfer(env: &Env, transfer_id: BytesN<32>) -> Option<DataTransfer> {
    env.storage().persistent().get(&DataKey::DataTransfer(transfer_id))
}

/// Update request status
pub fn update_request_status(
    env: &Env,
    requester: Address,
    request_id: BytesN<32>,
    new_status: RequestStatus,
) -> bool {
    requester.require_auth();

    let mut request: DataRequest = env.storage()
        .persistent()
        .get(&DataKey::DataRequest(request_id.clone()))
        .expect("Request not found");

    // Verify requester is authorized to update
    if request.requester != requester {
        panic!("Unauthorized to update request");
    }

    // Update status
    request.status = new_status.clone();
    env.storage().persistent().set(&DataKey::DataRequest(request_id.clone()), &request);

    // Emit status update event
    env.events().publish(
        (Symbol::new(env, "status_updated"),),
        (request_id, requester, new_status),
    );

    true
}

/// Get pending requests for a system
pub fn get_pending_requests(env: &Env, system_id: String) -> Vec<BytesN<32>> {
    let queue: Vec<BytesN<32>> = env.storage()
        .instance()
        .get(&DataKey::RequestQueue)
        .unwrap_or(Vec::new(env));

    let mut pending_requests = Vec::new(env);

    for request_id in queue.iter() {
        if let Some(request) = get_data_request(env, request_id.clone()) {
            if (request.sender_system == system_id || request.receiver_system == system_id) 
                && (request.status == RequestStatus::Pending || request.status == RequestStatus::ConsentVerified) {
                pending_requests.push_back(request_id);
            }
        }
    }

    pending_requests
}