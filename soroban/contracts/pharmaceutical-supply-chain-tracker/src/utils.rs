use soroban_sdk::{Env, String, Vec};

pub struct Utils;

impl Utils {
    pub fn validate_metadata(_env: &Env, metadata: &Vec<String>) -> bool {
        if metadata.len() == 0 {
            return false;
        }

        // Ensure no empty strings in metadata
        for item in metadata.iter() {
            if item.len() == 0 {
                return false;
            }
        }

        true
    }

    pub fn validate_location(env: &Env, location: &str) -> bool {
        // Basic location format validation
        // Expected format: "Name, Address, City, Country"
        let mut parts = Vec::new(env);
        for part in location.split(',') {
            parts.push_back(String::from_str(env, part.trim()));
        }

        if parts.len() < 4 {
            return false;
        }

        // Ensure each part has content
        for part in parts.iter() {
            if part.len() == 0 {
                return false;
            }
        }

        true
    }

    pub fn validate_temperature_range(min: i32, max: i32) -> bool {
        // Validate temperature range for cold chain management
        // Typical pharmaceutical storage: 2°C to 8°C (35°F to 46°F)
        min >= -20 && min < max && max <= 30
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{Env, String, Vec};

    #[test]
    fn test_metadata_validation() {
        let env = Env::default();
        let valid_metadata = Vec::from_array(
            &env,
            [
                String::from_str(&env, "Temperature: 5C"),
                String::from_str(&env, "Humidity: 45%"),
            ],
        );
        assert!(Utils::validate_metadata(&env, &valid_metadata));

        let empty_metadata = Vec::new(&env);
        assert!(!Utils::validate_metadata(&env, &empty_metadata));
    }

    #[test]
    fn test_location_validation() {
        let env = Env::default();
        assert!(Utils::validate_location(&env, "Pharmacy One, 123 Main St, Boston, USA"));
        assert!(!Utils::validate_location(&env, "Incomplete Address"));
        assert!(!Utils::validate_location(&env, ",,,")); // Empty parts
    }

    #[test]
    fn test_temperature_validation() {
        assert!(Utils::validate_temperature_range(2, 8)); // Standard range
        assert!(Utils::validate_temperature_range(-15, 25)); // Extended range
        assert!(!Utils::validate_temperature_range(10, 5)); // Invalid range
        assert!(!Utils::validate_temperature_range(-30, -25)); // Too cold
        assert!(!Utils::validate_temperature_range(35, 40)); // Too hot
    }
}