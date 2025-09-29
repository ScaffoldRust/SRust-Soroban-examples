use soroban_sdk::{
    contracttype, Address, Env, String,
};

#[derive(Clone, Debug, PartialEq, Eq)]
#[contracttype]
pub enum DataStatus {
    Submitted,              // Initial submission state
    UnderVerification,     // Currently being verified
    Verified,              // Successfully verified
    Rejected,              // Failed verification
}

#[derive(Clone, PartialEq)]
#[contracttype]
pub enum StudyPhase {
    Phase1,         // Safety and dosage
    Phase2,         // Efficacy and side effects
    Phase3,         // Efficacy and monitoring
    Phase4,         // Post-marketing surveillance
}

#[derive(Clone, PartialEq)]
#[contracttype]
pub enum DataType {
    PatientOutcome,         // Treatment outcomes
    AdverseEvent,           // Adverse events/side effects
    LabMeasurement,         // Laboratory test results
    VitalSigns,             // Vital signs measurements
    Dosage,                 // Medication dosage records
    Compliance,             // Patient compliance data
}

/// ICH-GCP compliant trial metadata
#[derive(Clone)]
#[contracttype]
pub struct TrialMetadata {
    pub data_type: DataType,
    pub patient_id_hash: String,        // Hashed patient identifier (HIPAA compliant)
    pub visit_number: u32,              // Study visit/timepoint
    pub measurement_date: u64,          // When data was collected
    pub protocol_deviation: bool,       // Any protocol deviations
    pub source_verified: bool,          // Source data verification status
}

#[derive(Clone)]
#[contracttype]
pub struct ClinicalData {
    pub trial_id: String,
    pub data_hash: String,              // Hash of the actual data
    pub metadata_hash: String,          // Hash of metadata
    pub metadata: TrialMetadata,        // Structured metadata
    pub submitter: Address,             // Address of data submitter
    pub submission_time: u64,           // Timestamp of submission
    pub status: DataStatus,             // Current verification status
    pub verifications: u32,             // Number of verifications received
    pub last_updated: u64,              // Last status update timestamp
}

#[derive(Clone)]
#[contracttype]
pub struct VerificationEvent {
    pub data_hash: String,
    pub verifier: Address,
    pub timestamp: u64,
    pub status: DataStatus,
    pub notes: String,                  // Verification notes/comments
    pub gcp_compliant: bool,            // GCP compliance flag
}

impl ClinicalData {
    pub fn new(
        env: &Env,
        trial_id: String,
        data_hash: String,
        metadata_hash: String,
        metadata: TrialMetadata,
        submitter: Address,
    ) -> Self {
        Self {
            trial_id,
            data_hash,
            metadata_hash,
            metadata,
            submitter,
            submission_time: env.ledger().timestamp(),
            status: DataStatus::Submitted,
            verifications: 0,
            last_updated: env.ledger().timestamp(),
        }
    }

    pub fn update_status(&mut self, env: &Env, new_status: DataStatus) {
        self.status = new_status;
        self.last_updated = env.ledger().timestamp();
    }

    pub fn add_verification(&mut self) {
        self.verifications += 1;
    }

    /// Validate ICH-GCP compliance
    pub fn validate_gcp_compliance(&self) -> bool {
        // Check required metadata fields
        if self.metadata.patient_id_hash.len() == 0 {
            return false;
        }
        if self.metadata.visit_number == 0 {
            return false;
        }
        if self.metadata.measurement_date == 0 {
            return false;
        }
        // Source verification required for GCP compliance
        if !self.metadata.source_verified {
            return false;
        }
        true
    }
}