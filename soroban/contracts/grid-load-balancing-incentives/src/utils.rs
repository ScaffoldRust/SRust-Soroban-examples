use soroban_sdk::{contracterror, contracttype, Address, Env};

#[contracttype]
#[derive(Copy, Clone)]
#[repr(u32)]
pub enum DataKey {
    Initialized = 0,
    Admin = 1,
    GridOperators = 2,
    Consumers = 3,
    Events = 4,
    Participations = 5,
    NextEventId = 6,
    TokenContract = 7,
    ParticipationCounter = 8,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct DemandResponseEvent {
    pub event_id: u64,
    pub grid_operator: Address,
    pub target_reduction_kw: u64,
    pub reward_per_kw: u64,
    pub start_time: u64,
    pub end_time: u64,
    pub status: EventStatus,
    pub total_participants: u64,
    pub total_reduction_achieved: u64,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct ParticipationRecord {
    pub participation_id: u64,
    pub event_id: u64,
    pub consumer: Address,
    pub baseline_usage_kw: u64,
    pub actual_usage_kw: u64,
    pub reduction_achieved_kw: u64,
    pub reward_amount: u64,
    pub meter_reading_start: u64,
    pub meter_reading_end: u64,
    pub verified_at: u64,
    pub status: ParticipationStatus,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
#[repr(u32)]
pub enum EventStatus {
    Active = 0,
    Completed = 1,
    Cancelled = 2,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
#[repr(u32)]
pub enum ParticipationStatus {
    Pending = 0,
    Verified = 1,
    Rewarded = 2,
    Rejected = 3,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[repr(u32)]
pub enum IncentiveError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    NotAuthorized = 3,
    InvalidInput = 4,
    GridOperatorNotRegistered = 5,
    ConsumerNotRegistered = 6,
    EventNotFound = 7,
    ParticipationNotFound = 8,
    EventNotActive = 9,
    EventExpired = 10,
    AlreadyParticipating = 11,
    InsufficientReduction = 12,
    RewardDistributionFailed = 13,
    InvalidMeterReading = 14,
}

pub fn get_next_event_id(env: &Env) -> u64 {
    let current_id: u64 = env
        .storage()
        .instance()
        .get(&DataKey::NextEventId)
        .unwrap_or(1);
    env.storage()
        .instance()
        .set(&DataKey::NextEventId, &(current_id + 1));
    current_id
}