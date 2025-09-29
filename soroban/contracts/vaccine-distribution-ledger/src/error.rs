use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    // Initialization errors
    AlreadyInitialized = 1,
    NotInitialized = 2,
    
    // Authorization errors
    Unauthorized = 3,
    AdminOnly = 4,
    ManufacturerOnly = 5,
    DistributorOnly = 6,
    AdministratorOnly = 7,
    AuthorizedPersonnelOnly = 8,
    
    // Batch errors
    BatchNotFound = 9,
    BatchAlreadyExists = 10,
    BatchExpired = 11,
    BatchRecalled = 12,
    BatchInactive = 13,
    InvalidBatchStatus = 14,
    
    // Distribution errors
    InsufficientQuantity = 15,
    InvalidQuantity = 16,
    DistributionNotFound = 17,
    InvalidDestination = 18,
    ColdChainBreach = 19,
    
    // Administration errors
    AdministrationNotFound = 20,
    InvalidPatientId = 21,
    DuplicateAdministration = 22,
    InvalidAdministrationData = 23,
    
    // Validation errors
    InvalidInput = 24,
    InvalidDate = 25,
    InvalidTemperature = 26,
    InvalidLocation = 27,
    
    // General errors
    DataNotFound = 28,
    OperationNotAllowed = 29,
    StorageError = 30,
    AccessDenied = 31,
}