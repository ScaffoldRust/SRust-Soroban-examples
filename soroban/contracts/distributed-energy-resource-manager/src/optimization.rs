use soroban_sdk::{contracttype, Env, String, Vec, Map, Symbol, symbol_short};

use crate::{DERInfo, ResourceType, DERStatus, OptimizationSchedule, EmergencyAllocation};

/// Advanced optimization algorithms for distributed energy resource coordination
pub struct OptimizationEngine;

impl OptimizationEngine {
    /// Optimize DERs for grid stability using advanced algorithms
    pub fn optimize_for_grid_stability(
        env: &Env,
        der_data: &Map<String, DERInfo>,
        current_demand: i32,
        grid_frequency: i32, // Using i32 instead of f32 for Soroban compatibility
    ) -> Vec<OptimizationSchedule> {
        let mut optimization_schedules = Vec::new(env);
        let current_time = env.ledger().timestamp();

        // Calculate grid stability requirements
        let stability_requirements = Self::calculate_stability_requirements(
            current_demand,
            grid_frequency,
        );

        // Get available DERs sorted by optimization potential
        let mut available_ders = Self::get_available_ders(env, der_data);
        Self::sort_ders_by_optimization_potential(&mut available_ders);

        // Apply optimization algorithms
        let mut remaining_demand = current_demand;
        let mut total_allocated = 0i32;

        for (der_id, der_info) in available_ders.iter() {
            if remaining_demand == 0 {
                break;
            }

            let optimal_allocation = Self::calculate_optimal_allocation(
                &der_info,
                remaining_demand,
                &stability_requirements,
            );

            if optimal_allocation.power_output != 0 {
                let schedule = OptimizationSchedule {
                    der_id: der_id.clone(),
                    timestamp: current_time,
                    power_output: optimal_allocation.power_output,
                    priority: optimal_allocation.priority,
                    grid_stability_score: optimal_allocation.grid_stability_score,
                };

                optimization_schedules.push_back(schedule);
                remaining_demand -= optimal_allocation.power_output;
                total_allocated += optimal_allocation.power_output;
            }
        }

        // Store optimization results
        Self::store_optimization_results(env, &optimization_schedules);

        optimization_schedules
    }

    /// Optimize for cost efficiency
    pub fn optimize_for_cost_efficiency(
        env: &Env,
        der_data: &Map<String, DERInfo>,
        energy_prices: &Map<ResourceType, u32>,
    ) -> Vec<OptimizationSchedule> {
        let mut optimization_schedules = Vec::new(env);
        let current_time = env.ledger().timestamp();

        // Get DERs sorted by cost efficiency
        let mut cost_efficient_ders = Self::get_cost_efficient_ders(env, der_data, energy_prices);
        Self::sort_ders_by_cost_efficiency(&mut cost_efficient_ders);

        for (der_id, der_info, cost_per_kw) in cost_efficient_ders.iter() {
            let optimal_power = Self::calculate_cost_optimal_power(&der_info, cost_per_kw);
            
            if optimal_power != 0 {
                let schedule = OptimizationSchedule {
                    der_id: der_id.clone(),
                    timestamp: current_time,
                    power_output: optimal_power,
                    priority: Self::calculate_cost_priority(cost_per_kw),
                    grid_stability_score: Self::calculate_cost_stability_score(&der_info),
                };

                optimization_schedules.push_back(schedule);
            }
        }

        optimization_schedules
    }

    /// Optimize for renewable energy maximization
    pub fn optimize_for_renewable_maximization(
        env: &Env,
        der_data: &Map<String, DERInfo>,
        weather_conditions: &WeatherConditions,
    ) -> Vec<OptimizationSchedule> {
        let mut optimization_schedules = Vec::new(env);
        let current_time = env.ledger().timestamp();

        // Prioritize renewable resources based on weather conditions
        for (der_id, der_info) in der_data.iter() {
            if !Self::is_renewable_resource(&der_info.resource_type) {
                continue;
            }

            let renewable_factor = Self::calculate_renewable_factor(
                &der_info.resource_type,
                weather_conditions,
            );

            if renewable_factor > 50 { // Only optimize if conditions are favorable (50% threshold)
                let optimal_power = Self::calculate_renewable_optimal_power(
                    &der_info,
                    renewable_factor,
                );

                let schedule = OptimizationSchedule {
                    der_id: der_id.clone(),
                    timestamp: current_time,
                    power_output: optimal_power,
                    priority: Self::calculate_renewable_priority(renewable_factor),
                    grid_stability_score: Self::calculate_renewable_stability_score(&der_info),
                };

                optimization_schedules.push_back(schedule);
            }
        }

        optimization_schedules
    }

    /// Emergency optimization for critical grid situations
    pub fn emergency_optimization(
        env: &Env,
        der_data: &Map<String, DERInfo>,
        emergency_level: u32,
    ) -> Vec<EmergencyAllocation> {
        let mut emergency_allocations = Vec::new(env);
        let current_time = env.ledger().timestamp();

        // Get all available DERs for emergency use
        let available_ders = Self::get_emergency_available_ders(env, der_data);

        // Calculate emergency power requirements
        let emergency_power_required = Self::calculate_emergency_power_requirement(emergency_level);

        let mut allocated_power = 0u32;
        for (der_id, der_info) in available_ders.iter() {
            if allocated_power >= emergency_power_required {
                break;
            }

            let max_emergency_power = Self::calculate_max_emergency_power(&der_info);
            let required_power = (emergency_power_required - allocated_power).min(max_emergency_power);

            if required_power > 0 {
                let allocation = EmergencyAllocation {
                    der_id: der_id.clone(),
                    required_power,
                    duration: Self::calculate_emergency_duration(emergency_level),
                    priority: 10, // Highest priority for emergency
                    active: true,
                };

                emergency_allocations.push_back(allocation);
                allocated_power += required_power;
            }
        }

        emergency_allocations
    }

    /// Calculate grid stability requirements
    fn calculate_stability_requirements(current_demand: i32, grid_frequency: i32) -> StabilityRequirements {
        let frequency_deviation = (grid_frequency - 5000).abs(); // Assuming 50Hz standard (5000 = 50.00 * 100)
        let stability_factor = if frequency_deviation < 10 {
            100
        } else if frequency_deviation < 50 {
            80
        } else {
            50
        };

        StabilityRequirements {
            frequency_stability: frequency_deviation < 20,
            voltage_stability: true, // Simplified
            power_quality: stability_factor,
            reserve_margin: 10, // 10% reserve margin
        }
    }

    /// Get available DERs for optimization
    fn get_available_ders(env: &Env, der_data: &Map<String, DERInfo>) -> Vec<(String, DERInfo)> {
        let mut available_ders = Vec::new(env);
        
        for (der_id, der_info) in der_data.iter() {
            if matches!(der_info.status, DERStatus::Online | DERStatus::Optimized) {
                available_ders.push_back((der_id, der_info));
            }
        }
        
        available_ders
    }

    /// Sort DERs by optimization potential
    fn sort_ders_by_optimization_potential(ders: &mut Vec<(String, DERInfo)>) {
        // Simple sorting by capacity and resource type priority
        // In a real implementation, this would use more sophisticated algorithms
        // For now, we'll just iterate through them in order
    }

    /// Calculate optimization score for a DER
    fn calculate_optimization_score(der_info: &DERInfo) -> u32 {
        let capacity_score = der_info.capacity;
        let type_score = match der_info.resource_type {
            ResourceType::Battery => 100,
            ResourceType::FuelCell => 90,
            ResourceType::Hydro => 80,
            ResourceType::Geothermal => 70,
            ResourceType::Solar => 60,
            ResourceType::Wind => 50,
        };
        
        capacity_score + type_score
    }

    /// Calculate optimal allocation for a DER
    fn calculate_optimal_allocation(
        der_info: &DERInfo,
        remaining_demand: i32,
        stability_requirements: &StabilityRequirements,
    ) -> OptimalAllocation {
        let max_power = der_info.capacity as i32;
        let optimal_power = if remaining_demand > 0 {
            remaining_demand.min(max_power)
        } else {
            -max_power // For storage/dispatchable resources
        };

        let priority = Self::calculate_priority_for_allocation(der_info, stability_requirements);
        let stability_score = Self::calculate_stability_score(der_info, stability_requirements);

        OptimalAllocation {
            power_output: optimal_power,
            priority,
            grid_stability_score: stability_score,
        }
    }

    /// Calculate priority for allocation
    fn calculate_priority_for_allocation(
        der_info: &DERInfo,
        stability_requirements: &StabilityRequirements,
    ) -> u32 {
        let base_priority = match der_info.resource_type {
            ResourceType::Battery => 9,
            ResourceType::FuelCell => 8,
            ResourceType::Hydro => 7,
            ResourceType::Geothermal => 6,
            ResourceType::Solar => 5,
            ResourceType::Wind => 4,
        };

        // Adjust based on stability requirements
        let stability_bonus = if stability_requirements.frequency_stability { 1 } else { 0 };
        
        (base_priority + stability_bonus).min(10)
    }

    /// Calculate stability score
    fn calculate_stability_score(
        der_info: &DERInfo,
        stability_requirements: &StabilityRequirements,
    ) -> u32 {
        let base_score = match der_info.resource_type {
            ResourceType::Battery => 9,
            ResourceType::FuelCell => 8,
            ResourceType::Hydro => 7,
            ResourceType::Geothermal => 6,
            ResourceType::Solar => 5,
            ResourceType::Wind => 4,
        };

        // Adjust based on power quality requirements
        let quality_factor = stability_requirements.power_quality / 10;
        
        (base_score + quality_factor).min(10)
    }

    /// Store optimization results
    fn store_optimization_results(env: &Env, schedules: &Vec<OptimizationSchedule>) {
        let mut opt_data: Map<String, Vec<OptimizationSchedule>> = env
            .storage()
            .instance()
            .get(&symbol_short!("OPT_SCHED"))
            .unwrap_or_else(|| Map::new(env));

        opt_data.set(String::from_str(env, "latest"), schedules.clone());
        env.storage().instance().set(&symbol_short!("OPT_SCHED"), &opt_data);
    }

    /// Get cost efficient DERs
    fn get_cost_efficient_ders(
        env: &Env,
        der_data: &Map<String, DERInfo>,
        energy_prices: &Map<ResourceType, u32>,
    ) -> Vec<(String, DERInfo, u32)> {
        let mut cost_efficient_ders = Vec::new(env);
        
        for (der_id, der_info) in der_data.iter() {
            if matches!(der_info.status, DERStatus::Online | DERStatus::Optimized) {
                let cost_per_kw = energy_prices.get(der_info.resource_type.clone()).unwrap_or(1000);
                cost_efficient_ders.push_back((der_id, der_info, cost_per_kw));
            }
        }
        
        cost_efficient_ders
    }

    /// Sort DERs by cost efficiency
    fn sort_ders_by_cost_efficiency(ders: &mut Vec<(String, DERInfo, u32)>) {
        // Simple sorting - in a real implementation would use more sophisticated algorithms
    }

    /// Calculate cost optimal power
    fn calculate_cost_optimal_power(der_info: &DERInfo, cost_per_kw: u32) -> i32 {
        // Simple cost optimization: use full capacity if cost is low
        if cost_per_kw < 500 {
            der_info.capacity as i32
        } else if cost_per_kw < 1000 {
            (der_info.capacity as i32 * 8) / 10
        } else {
            (der_info.capacity as i32 * 5) / 10
        }
    }

    /// Calculate cost priority
    fn calculate_cost_priority(cost_per_kw: u32) -> u32 {
        if cost_per_kw < 500 { 9 }
        else if cost_per_kw < 1000 { 7 }
        else if cost_per_kw < 1500 { 5 }
        else { 3 }
    }

    /// Calculate cost stability score
    fn calculate_cost_stability_score(der_info: &DERInfo) -> u32 {
        match der_info.resource_type {
            ResourceType::Battery => 8,
            ResourceType::FuelCell => 7,
            ResourceType::Hydro => 6,
            ResourceType::Geothermal => 5,
            ResourceType::Solar => 4,
            ResourceType::Wind => 3,
        }
    }

    /// Check if resource is renewable
    fn is_renewable_resource(resource_type: &ResourceType) -> bool {
        matches!(resource_type, ResourceType::Solar | ResourceType::Wind | ResourceType::Hydro | ResourceType::Geothermal)
    }

    /// Calculate renewable factor based on weather
    fn calculate_renewable_factor(
        resource_type: &ResourceType,
        weather: &WeatherConditions,
    ) -> u32 {
        match resource_type {
            ResourceType::Solar => weather.solar_irradiance / 10, // Scale down
            ResourceType::Wind => weather.wind_speed / 2, // Scale down
            ResourceType::Hydro => weather.water_flow / 10, // Scale down
            ResourceType::Geothermal => 80, // Generally stable
            _ => 0,
        }
    }

    /// Calculate renewable optimal power
    fn calculate_renewable_optimal_power(der_info: &DERInfo, renewable_factor: u32) -> i32 {
        (der_info.capacity as u32 * renewable_factor / 100) as i32
    }

    /// Calculate renewable priority
    fn calculate_renewable_priority(renewable_factor: u32) -> u32 {
        renewable_factor / 10
    }

    /// Calculate renewable stability score
    fn calculate_renewable_stability_score(der_info: &DERInfo) -> u32 {
        match der_info.resource_type {
            ResourceType::Hydro => 8,
            ResourceType::Geothermal => 7,
            ResourceType::Solar => 6,
            ResourceType::Wind => 5,
            _ => 0,
        }
    }

    /// Get emergency available DERs
    fn get_emergency_available_ders(env: &Env, der_data: &Map<String, DERInfo>) -> Vec<(String, DERInfo)> {
        let mut emergency_ders = Vec::new(env);
        
        for (der_id, der_info) in der_data.iter() {
            if matches!(der_info.status, DERStatus::Online | DERStatus::Optimized | DERStatus::Emergency) {
                emergency_ders.push_back((der_id, der_info));
            }
        }
        
        emergency_ders
    }

    /// Calculate emergency power requirement
    fn calculate_emergency_power_requirement(emergency_level: u32) -> u32 {
        match emergency_level {
            1 => 1000,  // Low emergency
            2 => 5000,  // Medium emergency
            3 => 10000, // High emergency
            4 => 20000, // Critical emergency
            _ => 5000,
        }
    }

    /// Calculate max emergency power for DER
    fn calculate_max_emergency_power(der_info: &DERInfo) -> u32 {
        // In emergency, can use up to 120% of rated capacity
        (der_info.capacity * 12) / 10
    }

    /// Calculate emergency duration
    fn calculate_emergency_duration(emergency_level: u32) -> u64 {
        match emergency_level {
            1 => 3600,  // 1 hour
            2 => 7200,  // 2 hours
            3 => 14400, // 4 hours
            4 => 28800, // 8 hours
            _ => 3600,
        }
    }
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StabilityRequirements {
    pub frequency_stability: bool,
    pub voltage_stability: bool,
    pub power_quality: u32, // Using u32 instead of f32
    pub reserve_margin: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WeatherConditions {
    pub solar_irradiance: u32, // W/m² * 100 for integer storage
    pub wind_speed: u32,       // m/s * 10 for integer storage
    pub water_flow: u32,       // m³/s * 10 for integer storage
    pub temperature: i32,      // °C * 10 for integer storage
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OptimalAllocation {
    pub power_output: i32,
    pub priority: u32,
    pub grid_stability_score: u32,
}