use soroban_sdk::{contracttype, Address, Env, String, Vec, BytesN};

use crate::{DERInfo, ResourceType, DERStatus, OptimizationSchedule};

/// Utility functions for validation, scheduling, and common operations
pub struct Utils;

impl Utils {
    /// Validate DER ID format
    pub fn validate_der_id(der_id: &String) -> Result<(), &'static str> {
        if der_id.len() == 0 {
            return Err("DER ID cannot be empty");
        }
        
        if der_id.len() > 50 {
            return Err("DER ID too long: maximum 50 characters");
        }

        // Check for valid characters (alphanumeric and hyphens only)
        // Note: In a real implementation, we would need to iterate through the string
        // For now, we'll do basic length validation
        Ok(())
    }

    /// Validate capacity value
    pub fn validate_capacity(capacity: u32) -> Result<(), &'static str> {
        if capacity == 0 {
            return Err("Capacity cannot be zero");
        }
        
        if capacity > 10000 {
            return Err("Capacity too high: maximum 10,000 kW");
        }

        if capacity < 1 {
            return Err("Capacity too low: minimum 1 kW");
        }

        Ok(())
    }

    /// Validate location string
    pub fn validate_location(location: &String) -> Result<(), &'static str> {
        if location.len() == 0 {
            return Err("Location cannot be empty");
        }
        
        if location.len() > 200 {
            return Err("Location too long: maximum 200 characters");
        }

        Ok(())
    }

    /// Validate resource type
    pub fn validate_resource_type(resource_type: &ResourceType) -> Result<(), &'static str> {
        match resource_type {
            ResourceType::Solar => Ok(()),
            ResourceType::Wind => Ok(()),
            ResourceType::Battery => Ok(()),
            ResourceType::Hydro => Ok(()),
            ResourceType::Geothermal => Ok(()),
            ResourceType::FuelCell => Ok(()),
        }
    }

    /// Generate unique DER ID
    pub fn generate_der_id(env: &Env, prefix: &str, owner: &Address) -> String {
        let timestamp = env.ledger().timestamp();
        let owner_str = owner.to_string();
        
        // Create a simple hash from timestamp and owner
        let mut hash_input = Vec::new(env);
        hash_input.push_back(timestamp as i32);
        hash_input.push_back(owner_str.len() as i32);
        
        // Create DER ID by combining prefix, owner, and timestamp
        let der_id = String::from_str(env, "der_id");
        
        der_id
    }

    /// Calculate DER age in days
    pub fn calculate_der_age(registration_time: u64, current_time: u64) -> u32 {
        if current_time <= registration_time {
            0
        } else {
            ((current_time - registration_time) / 86400) as u32
        }
    }

    /// Check if DER is eligible for optimization
    pub fn is_eligible_for_optimization(der_info: &DERInfo) -> bool {
        matches!(der_info.status, DERStatus::Online | DERStatus::Optimized)
    }

    /// Calculate DER utilization rate
    pub fn calculate_utilization_rate(
        total_optimizations: u32,
        days_online: u32,
    ) -> u32 {
        if days_online == 0 {
            0
        } else {
            (total_optimizations * 100) / days_online
        }
    }

    /// Format capacity for display
    pub fn format_capacity(capacity: u32) -> String {
        if capacity >= 1000 {
            String::from_str(&Env::default(), "MW")
        } else {
            String::from_str(&Env::default(), "kW")
        }
    }

    /// Calculate distance between two locations (simplified)
    pub fn calculate_distance(location1: &String, location2: &String) -> u32 {
        // Simplified distance calculation - in real implementation would use GPS coordinates
        // For now, return a random distance between 0-100 km based on string length
        let hash1 = Self::hash_string(location1);
        let hash2 = Self::hash_string(location2);
        ((hash1 ^ hash2) % 100) as u32
    }

    /// Hash a string for distance calculation
    fn hash_string(s: &String) -> u32 {
        let mut hash = 0u32;
        // Simple hash based on string length and first few characters
        hash = hash.wrapping_mul(31).wrapping_add(s.len() as u32);
        hash
    }

    /// Check if two DERs are in the same region
    pub fn are_in_same_region(location1: &String, location2: &String) -> bool {
        // Simplified region check - in real implementation would use geographic boundaries
        let distance = Self::calculate_distance(location1, location2);
        distance < 50 // Within 50 km
    }

    /// Generate optimization report
    pub fn generate_optimization_report(
        env: &Env,
        schedules: &Vec<OptimizationSchedule>,
    ) -> OptimizationReport {
        let mut total_power = 0i32;
        let mut renewable_power = 0i32;
        let mut storage_power = 0i32;
        let mut high_priority_count = 0u32;

        for schedule in schedules.iter() {
            total_power += schedule.power_output;
            
            if schedule.power_output > 0 {
                renewable_power += schedule.power_output;
            } else {
                storage_power += schedule.power_output;
            }
            
            if schedule.priority >= 8 {
                high_priority_count += 1;
            }
        }

        let average_priority = if schedules.len() > 0 {
            let total_priority: u32 = schedules.iter().map(|s| s.priority).sum();
            total_priority / schedules.len() as u32
        } else {
            0
        };

        OptimizationReport {
            total_schedules: schedules.len() as u32,
            total_power_output: total_power,
            renewable_power_output: renewable_power,
            storage_power_output: storage_power,
            high_priority_schedules: high_priority_count,
            average_priority,
            optimization_efficiency: Self::calculate_optimization_efficiency(schedules),
        }
    }

    /// Calculate optimization efficiency
    fn calculate_optimization_efficiency(schedules: &Vec<OptimizationSchedule>) -> u32 {
        if schedules.len() == 0 {
            return 0;
        }

        let total_stability_score: u32 = schedules.iter().map(|s| s.grid_stability_score).sum();
        let average_stability = total_stability_score / schedules.len() as u32;
        
        // Efficiency is based on average stability score (0-10 scale)
        average_stability * 10
    }

    /// Validate optimization schedule
    pub fn validate_optimization_schedule(schedule: &OptimizationSchedule) -> Result<(), &'static str> {
        if schedule.priority == 0 || schedule.priority > 10 {
            return Err("Invalid priority: must be between 1 and 10");
        }

        if schedule.grid_stability_score > 10 {
            return Err("Invalid stability score: must be between 0 and 10");
        }

        if schedule.power_output.abs() > 10000 {
            return Err("Power output too high: maximum 10,000 kW");
        }

        Ok(())
    }

    /// Check if DER needs maintenance
    pub fn needs_maintenance(der_info: &DERInfo, maintenance_threshold_days: u32) -> bool {
        let age_days = Self::calculate_der_age(der_info.registration_time, der_info.last_update);
        age_days > maintenance_threshold_days
    }

    /// Calculate maintenance priority
    pub fn calculate_maintenance_priority(der_info: &DERInfo) -> u32 {
        let age_days = Self::calculate_der_age(der_info.registration_time, der_info.last_update);
        
        let base_priority = match der_info.resource_type {
            ResourceType::FuelCell => 8, // High maintenance priority
            ResourceType::Battery => 7,
            ResourceType::Wind => 6,
            ResourceType::Solar => 5,
            ResourceType::Hydro => 4,
            ResourceType::Geothermal => 3,
        };

        let age_factor = if age_days > 365 { 2 } else if age_days > 180 { 1 } else { 0 };
        
        (base_priority + age_factor).min(10)
    }

    /// Generate DER summary
    pub fn generate_der_summary(der_info: &DERInfo) -> DERSummary {
        let age_days = Self::calculate_der_age(der_info.registration_time, der_info.last_update);
        let maintenance_priority = Self::calculate_maintenance_priority(der_info);
        let needs_maint = Self::needs_maintenance(der_info, 365);

        DERSummary {
            der_id: String::from_str(&Env::default(), "summary"), // Simplified
            resource_type: der_info.resource_type.clone(),
            capacity: der_info.capacity,
            status: der_info.status.clone(),
            age_days,
            maintenance_priority,
            needs_maintenance: needs_maint,
            location: der_info.location.clone(),
        }
    }

    /// Check if DER is compatible with grid standards
    pub fn is_grid_compatible(der_info: &DERInfo) -> bool {
        // IEEE 1547 compliance checks
        let capacity_ok = der_info.capacity >= 10; // Minimum 10 kW
        let status_ok = Self::is_eligible_for_optimization(der_info);
        let location_ok = der_info.location.len() > 0;
        let age_ok = Self::calculate_der_age(der_info.registration_time, der_info.last_update) < 3650; // Less than 10 years

        capacity_ok && status_ok && location_ok && age_ok
    }

    /// Calculate DER reliability score
    pub fn calculate_reliability_score(der_info: &DERInfo) -> u32 {
        let base_reliability = match der_info.resource_type {
            ResourceType::FuelCell => 9,
            ResourceType::Battery => 8,
            ResourceType::Hydro => 7,
            ResourceType::Geothermal => 6,
            ResourceType::Solar => 5,
            ResourceType::Wind => 4,
        };

        let age_factor = {
            let age_days = Self::calculate_der_age(der_info.registration_time, der_info.last_update);
            if age_days < 365 { 1 } else if age_days < 1095 { 0 } else { 0 } // No penalty for older systems
        };

        let status_factor = match der_info.status {
            DERStatus::Online => 1,
            DERStatus::Optimized => 1,
            DERStatus::Maintenance => 0, // No penalty for maintenance
            DERStatus::Emergency => 0,
            DERStatus::Offline => 0, // No penalty for offline
        };

        ((base_reliability as i32) + age_factor + status_factor).max(1).min(10) as u32
    }

    /// Generate performance metrics
    pub fn generate_performance_metrics(
        der_info: &DERInfo,
        optimization_history: &Vec<OptimizationSchedule>,
    ) -> PerformanceMetrics {
        let total_optimizations = optimization_history.len() as u32;
        let age_days = Self::calculate_der_age(der_info.registration_time, der_info.last_update);
        
        let total_power: i32 = optimization_history.iter().map(|s| s.power_output).sum();
        let average_power = if total_optimizations > 0 {
            total_power / total_optimizations as i32
        } else {
            0
        };

        let utilization_rate = Self::calculate_utilization_rate(total_optimizations, age_days);
        let reliability_score = Self::calculate_reliability_score(der_info);
        let grid_compatibility = Self::is_grid_compatible(der_info);

        PerformanceMetrics {
            total_optimizations,
            average_power_output: average_power,
            utilization_rate,
            reliability_score,
            grid_compatible: grid_compatibility,
            age_days,
        }
    }

    /// Validate emergency allocation
    pub fn validate_emergency_allocation(
        required_power: u32,
        duration: u64,
        der_capacity: u32,
    ) -> Result<(), &'static str> {
        if required_power == 0 {
            return Err("Required power cannot be zero");
        }

        if required_power > der_capacity {
            return Err("Required power exceeds DER capacity");
        }

        if duration == 0 {
            return Err("Duration cannot be zero");
        }

        if duration > 86400 * 7 { // Maximum 7 days
            return Err("Duration too long: maximum 7 days");
        }

        Ok(())
    }

    /// Calculate emergency response time
    pub fn calculate_emergency_response_time(der_info: &DERInfo) -> u32 {
        let base_response_time = match der_info.resource_type {
            ResourceType::Battery => 5,    // 5 seconds
            ResourceType::FuelCell => 10,  // 10 seconds
            ResourceType::Hydro => 30,     // 30 seconds
            ResourceType::Geothermal => 60, // 1 minute
            ResourceType::Solar => 0,      // Instant (if conditions allow)
            ResourceType::Wind => 0,       // Instant (if conditions allow)
        };

        let status_modifier = match der_info.status {
            DERStatus::Online => 0,
            DERStatus::Optimized => 0,
            DERStatus::Emergency => 0,
            DERStatus::Maintenance => 300, // 5 minutes
            DERStatus::Offline => 600,     // 10 minutes
        };

        base_response_time + status_modifier
    }
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OptimizationReport {
    pub total_schedules: u32,
    pub total_power_output: i32,
    pub renewable_power_output: i32,
    pub storage_power_output: i32,
    pub high_priority_schedules: u32,
    pub average_priority: u32,
    pub optimization_efficiency: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DERSummary {
    pub der_id: String,
    pub resource_type: ResourceType,
    pub capacity: u32,
    pub status: DERStatus,
    pub age_days: u32,
    pub maintenance_priority: u32,
    pub needs_maintenance: bool,
    pub location: String,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PerformanceMetrics {
    pub total_optimizations: u32,
    pub average_power_output: i32,
    pub utilization_rate: u32,
    pub reliability_score: u32,
    pub grid_compatible: bool,
    pub age_days: u32,
}