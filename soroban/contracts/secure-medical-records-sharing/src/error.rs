use soroban_sdk::{contracttype, contracterror};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum MedicalRecordsError {
    // Authentication and authorization errors
    Unauthorized = 1000,
    InsufficientPermissions = 1001,
    AccessExpired = 1002,
    AccessRevoked = 1003,
    
    // Record management errors
    RecordNotFound = 2000,
    RecordAlreadyExists = 2001,
    InvalidRecordData = 2002,
    RecordNotOwnedByPatient = 2003,
    
    // Provider management errors
    ProviderNotRegistered = 3000,
    ProviderAlreadyRegistered = 3001,
    ProviderNotVerified = 3002,
    
    // Access control errors
    AccessAlreadyGranted = 4000,
    AccessNotGranted = 4001,
    InvalidAccessLevel = 4002,
    EmergencyAccessDenied = 4003,
    
    // Consent management errors
    ConsentRequired = 5000,
    ConsentExpired = 5001,
    ConsentDenied = 5002,
    ConsentContractNotSet = 5003,
    
    // System errors
    InvalidAdmin = 6000,
    ContractNotInitialized = 6001,
    InvalidTimestamp = 6002,
    StorageError = 6003,
    
    // Validation errors
    InvalidInput = 7000,
    InvalidRecordType = 7001,
    InvalidSensitivityLevel = 7002,
    ExpirationTooShort = 7003,
    ExpirationTooLong = 7004,
    
    // Audit errors
    AuditLogCorrupted = 8000,
    AuditLogFull = 8001,
    InvalidSequenceNumber = 8002,
}