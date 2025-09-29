use soroban_sdk::{contracttype, Address, Env, String, Vec, Map, Symbol, symbol_short};

use crate::{DERInfo, ResourceType, DERStatus, OptimizationSchedule};

/// Core resource management functionality for Distributed Energy Resources
pub struct ResourceManager;

impl ResourceManager {
    /// Validate DER registration data
    pub fn validate_der_registration(
        der_id: &String,
        capacity: u32,
        resource_type: &ResourceType,
    ) -> Result<(), &'static str> {
        // Validate DER ID
        if der_id.len() == 0 || der_id.len() > 50 {
            return Err("Invalid DER ID: must be 1-50 characters");
        }

        // Validate capacity
        if capacity == 0 || capacity > 10000 {
            return Err("Invalid capacity: must be between 1 and 10,000 kW");
        }

        // Validate resource type specific constraints
        match resource_type {
            ResourceType::Solar => {
                if capacity > 5000 {
                    return Err("Solar capacity too high: maximum 5,000 kW");
                }
            }
            ResourceType::Wind => {
                if capacity > 3000 {
                    return Err("Wind capacity too high: maximum 3,000 kW");
                }
            }
            ResourceType::Battery => {
                if capacity > 2000 {
                    return Err("Battery capacity too high: maximum 2,000 kW");
                }
            }
            ResourceType::Hydro => {
                if capacity > 8000 {
                    return Err("Hydro capacity too high: maximum 8,000 kW");
                }
            }
            ResourceType::Geothermal => {
                if capacity > 6000 {
                    return Err("Geothermal capacity too high: maximum 6,000 kW");
                }
            }
            ResourceType::FuelCell => {
                if capacity > 1000 {
                    return Err("Fuel cell capacity too high: maximum 1,000 kW");
                }
            }
        }

        Ok(())
    }

    /// Calculate DER efficiency based on resource type and conditions
    pub fn calculate_efficiency(
        resource_type: &ResourceType,
        age_days: u32,
        maintenance_score: u32,
    ) -> u32 {
        let base_efficiency: u32 = match resource_type {
            ResourceType::Solar => 85,
            ResourceType::Wind => 75,
            ResourceType::Battery => 90,
            ResourceType::Hydro => 95,
            ResourceType::Geothermal => 88,
            ResourceType::FuelCell => 92,
        };

        // Age degradation (1% per year)
        let age_penalty = age_days / 365;
        
        // Maintenance impact
        let maintenance_penalty = 10 - maintenance_score.min(10);
        
        // Calculate final efficiency
        let efficiency = base_efficiency.saturating_sub(age_penalty).saturating_sub(maintenance_penalty);
        efficiency.max(50) // Minimum 50% efficiency
    }

    /// Determine if a DER can participate in grid optimization
    pub fn can_participate_in_optimization(der_info: &DERInfo) -> bool {
        matches!(der_info.status, DERStatus::Online | DERStatus::Optimized)
    }

    /// Calculate DER availability score for grid operations
    pub fn calculate_availability_score(der_info: &DERInfo) -> u32 {
        let base_score = match der_info.status {
            DERStatus::Online => 10,
            DERStatus::Optimized => 9,
            DERStatus::Maintenance => 3,
            DERStatus::Emergency => 8,
            DERStatus::Offline => 0,
        };

        // Adjust based on resource type reliability
        let reliability_modifier = match der_info.resource_type {
            ResourceType::FuelCell => 1,
            ResourceType::Battery => 1,
            ResourceType::Hydro => 1,
            ResourceType::Geothermal => 0,
            ResourceType::Solar => 0, // No penalty for solar
            ResourceType::Wind => 0,  // No penalty for wind
        };

        (base_score as i32 + reliability_modifier).max(1).min(10) as u32
    }

    /// Generate DER performance metrics
    pub fn generate_performance_metrics(
        der_info: &DERInfo,
        historical_data: &Vec<OptimizationSchedule>,
    ) -> DERPerformanceMetrics {
        let total_optimizations = historical_data.len() as u32;
        let age_days = (der_info.last_update - der_info.registration_time) / 86400; // Convert to days
        
        let total_power: i32 = historical_data.iter().map(|s| s.power_output).sum();
        let average_power = if total_optimizations > 0 {
            total_power / total_optimizations as i32
        } else {
            0
        };

        let efficiency = Self::calculate_efficiency(
            &der_info.resource_type,
            age_days as u32,
            8, // Default maintenance score
        );

        let availability_score = Self::calculate_availability_score(der_info);

        DERPerformanceMetrics {
            total_optimizations,
            average_power_output: average_power,
            efficiency_percentage: efficiency,
            availability_score,
            capacity_utilization: if der_info.capacity > 0 {
                (average_power.abs() as u32 * 100) / der_info.capacity
            } else {
                0
            },
        }
    }

    /// Check if DER meets grid integration standards
    pub fn meets_grid_standards(der_info: &DERInfo) -> bool {
        // IEEE 1547 compliance checks
        let capacity_ok = der_info.capacity >= 10; // Minimum 10 kW for grid integration
        let status_ok = Self::can_participate_in_optimization(der_info);
        let location_ok = der_info.location.len() > 0;

        capacity_ok && status_ok && location_ok
    }

    /// Calculate DER's contribution to grid stability
    pub fn calculate_grid_stability_contribution(der_info: &DERInfo) -> u32 {
        let base_contribution = match der_info.resource_type {
            ResourceType::Battery => 9, // High for frequency regulation
            ResourceType::FuelCell => 8, // High for reliable generation
            ResourceType::Hydro => 7, // Good for load following
            ResourceType::Geothermal => 6, // Good for base load
            ResourceType::Solar => 4, // Variable but predictable
            ResourceType::Wind => 3, // Variable and less predictable
        };

        let capacity_factor = if der_info.capacity >= 1000 {
            2 // Large installations get bonus
        } else if der_info.capacity >= 100 {
            1 // Medium installations
        } else {
            0 // Small installations
        };

        (base_contribution + capacity_factor).min(10)
    }

    /// Validate DER status transition
    pub fn validate_status_transition(
        current_status: &DERStatus,
        new_status: &DERStatus,
    ) -> Result<(), &'static str> {
        match (current_status, new_status) {
            (DERStatus::Offline, DERStatus::Online) => Ok(()),
            (DERStatus::Online, DERStatus::Offline) => Ok(()),
            (DERStatus::Online, DERStatus::Maintenance) => Ok(()),
            (DERStatus::Maintenance, DERStatus::Online) => Ok(()),
            (DERStatus::Online, DERStatus::Emergency) => Ok(()),
            (DERStatus::Emergency, DERStatus::Online) => Ok(()),
            (DERStatus::Online, DERStatus::Optimized) => Ok(()),
            (DERStatus::Optimized, DERStatus::Online) => Ok(()),
            (DERStatus::Optimized, DERStatus::Emergency) => Ok(()),
            (DERStatus::Emergency, DERStatus::Optimized) => Ok(()),
            _ => Err("Invalid status transition"),
        }
    }

    /// Get DER maintenance recommendations
    pub fn get_maintenance_recommendations(der_info: &DERInfo) -> Vec<String> {
        let mut recommendations = Vec::new(&Env::default());
        let age_days = (der_info.last_update - der_info.registration_time) / 86400;

        match der_info.resource_type {
            ResourceType::Solar => {
                if age_days > 365 {
                    recommendations.push_back(String::from_str(&Env::default(), "Clean solar panels"));
                }
                if age_days > 730 {
                    recommendations.push_back(String::from_str(&Env::default(), "Check inverter connections"));
                }
            }
            ResourceType::Wind => {
                if age_days > 180 {
                    recommendations.push_back(String::from_str(&Env::default(), "Inspect turbine blades"));
                }
                if age_days > 365 {
                    recommendations.push_back(String::from_str(&Env::default(), "Check gearbox oil"));
                }
            }
            ResourceType::Battery => {
                if age_days > 90 {
                    recommendations.push_back(String::from_str(&Env::default(), "Check battery health"));
                }
                if age_days > 180 {
                    recommendations.push_back(String::from_str(&Env::default(), "Calibrate battery management system"));
                }
            }
            ResourceType::Hydro => {
                if age_days > 365 {
                    recommendations.push_back(String::from_str(&Env::default(), "Inspect turbine components"));
                }
                if age_days > 730 {
                    recommendations.push_back(String::from_str(&Env::default(), "Check dam infrastructure"));
                }
            }
            ResourceType::Geothermal => {
                if age_days > 180 {
                    recommendations.push_back(String::from_str(&Env::default(), "Check heat exchanger"));
                }
                if age_days > 365 {
                    recommendations.push_back(String::from_str(&Env::default(), "Inspect well integrity"));
                }
            }
            ResourceType::FuelCell => {
                if age_days > 90 {
                    recommendations.push_back(String::from_str(&Env::default(), "Check fuel cell stack"));
                }
                if age_days > 180 {
                    recommendations.push_back(String::from_str(&Env::default(), "Replace fuel filters"));
                }
            }
        }

        recommendations
    }
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DERPerformanceMetrics {
    pub total_optimizations: u32,
    pub average_power_output: i32,
    pub efficiency_percentage: u32,
    pub availability_score: u32,
    pub capacity_utilization: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DERMaintenanceRecord {
    pub der_id: String,
    pub maintenance_type: String,
    pub performed_by: Address,
    pub timestamp: u64,
    pub notes: String,
    pub cost: u32, // in micro-units
}

impl ResourceManager {
    /// Record maintenance activity
    pub fn record_maintenance(
        env: &Env,
        der_id: String,
        maintenance_type: String,
        performed_by: Address,
        notes: String,
        cost: u32,
    ) -> bool {
        let maintenance_record = DERMaintenanceRecord {
            der_id: der_id.clone(),
            maintenance_type,
            performed_by: performed_by.clone(),
            timestamp: env.ledger().timestamp(),
            notes,
            cost,
        };

        // Store maintenance record
        let mut maintenance_data: Map<String, Vec<DERMaintenanceRecord>> = env
            .storage()
            .instance()
            .get(&symbol_short!("MAINT"))
            .unwrap_or_else(|| Map::new(env));

        let mut records = maintenance_data.get(der_id.clone()).unwrap_or_else(|| Vec::new(env));
        records.push_back(maintenance_record);
        maintenance_data.set(der_id.clone(), records);
        env.storage().instance().set(&symbol_short!("MAINT"), &maintenance_data);

        // Emit maintenance event
        env.events().publish(
            (Symbol::new(env, "maintenance_recorded"), der_id),
            (performed_by, cost)
        );

        true
    }

    /// Get maintenance history for a DER
    pub fn get_maintenance_history(
        env: &Env,
        der_id: String,
    ) -> Vec<DERMaintenanceRecord> {
        let maintenance_data: Map<String, Vec<DERMaintenanceRecord>> = env
            .storage()
            .instance()
            .get(&symbol_short!("MAINT"))
            .unwrap_or_else(|| Map::new(env));

        maintenance_data.get(der_id).unwrap_or_else(|| Vec::new(env))
    }
}