extern crate std;

use super::utils::TestFixture;
use soroban_sdk::testutils::Address as _;

/// Test creating a medical record successfully
#[test]
fn test_create_medical_record() {
    let fixture = TestFixture::new();
    let record_id = fixture.create_record("lab", "ipfs://QmLabHash");
    assert_eq!(record_id, 1);

    let record = fixture.get_record(record_id);
    assert_eq!(record.record_id, 1);
    assert_eq!(record.patient, fixture.patient);
    assert_eq!(record.data_type, fixture.string("lab"));
    assert_eq!(record.pointer, fixture.string("ipfs://QmLabHash"));
    assert!(record.active);
}

/// Test granting access to a valid provider for specific record types
#[test]
fn test_grant_access_to_valid_provider() {
    let fixture = TestFixture::new();
    fixture.create_record("imaging", "ipfs://QmImageHash");

    fixture.grant_access(std::vec!["imaging"], 0);
    assert!(fixture.verify_access("imaging"));
}

/// Test granting access to non-existent record type
#[test]
fn test_grant_access_to_nonexistent_record_type() {
    let fixture = TestFixture::new();

    // Can grant access to any data type, even if no record exists yet
    fixture.grant_access(std::vec!["nonexistent"], 0);
    assert!(fixture.verify_access("nonexistent"));
}

/// Test revoking access from a provider
#[test]
fn test_revoke_access() {
    let fixture = TestFixture::new();
    fixture.create_record("lab", "ipfs://QmLabHash");

    fixture.grant_access(std::vec!["lab"], 0);
    assert!(fixture.verify_access("lab"));

    fixture.client.revoke_access(&fixture.patient, &fixture.provider);
    assert!(!fixture.verify_access("lab"));
}

/// Test revoking non-existent access grant
#[test]
#[should_panic(expected = "No grant")]
fn test_revoke_nonexistent_access() {
    let fixture = TestFixture::new();
    fixture.client.revoke_access(&fixture.patient, &fixture.provider);
}

/// Test patient authorization for sharing specific record types
#[test]
fn test_patient_authorization_for_specific_types() {
    let fixture = TestFixture::new();
    fixture.create_record("imaging", "ipfs://QmImage1");
    fixture.create_record("lab", "ipfs://QmLab1");

    // Grant access only to imaging
    fixture.grant_access(std::vec!["imaging"], 0);

    assert!(fixture.verify_access("imaging"));
    assert!(!fixture.verify_access("lab"));
}

/// Test updating a medical record
#[test]
fn test_update_record() {
    let fixture = TestFixture::new();
    let record_id = fixture.create_record("lab", "ipfs://QmOldHash");

    let new_pointer = fixture.string("ipfs://QmNewHash");
    let result = fixture.client.update_record(&fixture.patient, &record_id, &new_pointer);
    assert!(result);

    let record = fixture.get_record(record_id);
    assert_eq!(record.pointer, new_pointer);
}

/// Test updating record by unauthorized user
#[test]
#[should_panic(expected = "Record not found")]
fn test_update_record_unauthorized() {
    let fixture = TestFixture::new();
    let record_id = fixture.create_record("lab", "ipfs://QmOldHash");

    // Try to update as different patient (will fail with "Record not found" since unauthorized user has no records)
    let new_pointer = fixture.string("ipfs://QmNewHash");
    fixture.client.update_record(&fixture.unauthorized, &record_id, &new_pointer);
}

/// Test duplicate access grant attempt for the same provider
#[test]
fn test_duplicate_access_grant() {
    let fixture = TestFixture::new();

    // Grant access twice to same provider
    fixture.grant_access(std::vec!["imaging"], 0);
    fixture.grant_access(std::vec!["lab"], 0); // This replaces the previous grant

    // Only lab should be accessible (the latest grant)
    assert!(!fixture.verify_access("imaging")); // Previous grant replaced
    assert!(fixture.verify_access("lab"));
}

/// Test getting patient records
#[test]
fn test_get_patient_records() {
    let fixture = TestFixture::new();

    fixture.create_record("lab", "ipfs://QmLab1");
    fixture.create_record("imaging", "ipfs://QmImage1");
    fixture.create_record("lab", "ipfs://QmLab2");

    // Get all records
    let records = fixture.client.list_records(&fixture.patient, &fixture.patient, &0, &10, &None);
    assert_eq!(records.len(), 3);

    // Filter by data type
    let lab_type = Some(fixture.string("lab"));
    let lab_records = fixture.client.list_records(&fixture.patient, &fixture.patient, &0, &10, &lab_type);
    assert_eq!(lab_records.len(), 2);
}

/// Test pagination of patient records
#[test]
fn test_patient_records_pagination() {
    let fixture = TestFixture::new();

    // Create 5 records
    fixture.create_record("lab", "ipfs://QmHash1");
    fixture.create_record("lab", "ipfs://QmHash2");
    fixture.create_record("lab", "ipfs://QmHash3");
    fixture.create_record("lab", "ipfs://QmHash4");
    fixture.create_record("lab", "ipfs://QmHash5");

    // Get first 2 records
    let records1 = fixture.client.list_records(&fixture.patient, &fixture.patient, &0, &2, &None);
    assert_eq!(records1.len(), 2);

    // Get next 2 records
    let records2 = fixture.client.list_records(&fixture.patient, &fixture.patient, &2, &2, &None);
    assert_eq!(records2.len(), 2);

    // Get last record
    let records3 = fixture.client.list_records(&fixture.patient, &fixture.patient, &4, &2, &None);
    assert_eq!(records3.len(), 1);
}

/// Test access granting for non-existent record (edge case)
#[test]
fn test_grant_access_to_nonexistent_record() {
    let fixture = TestFixture::new();

    // Grant access without creating a record first
    fixture.grant_access(std::vec!["imaging"], 0);
    assert!(fixture.verify_access("imaging"));

    // Now create the record
    let record_id = fixture.create_record("imaging", "ipfs://QmImage");
    assert_eq!(record_id, 1);

    // Provider should still have access
    assert!(fixture.verify_access("imaging"));
}

/// Test unauthorized provider attempting to access records
#[test]
#[should_panic(expected = "Access denied")]
fn test_unauthorized_provider_access() {
    let fixture = TestFixture::new();
    let record_id = fixture.create_record("lab", "ipfs://QmLab");

    // Try to access without grant
    fixture.client.get_record(&fixture.provider, &fixture.patient, &record_id);
}

/// Test high-volume record creation for scalability
#[test]
fn test_high_volume_record_creation() {
    let fixture = TestFixture::new();

    // Create 50 records
    for i in 1..=50 {
        let hash = match i {
            1 => "ipfs://QmHash1", 2 => "ipfs://QmHash2", 3 => "ipfs://QmHash3", 4 => "ipfs://QmHash4", 5 => "ipfs://QmHash5",
            6 => "ipfs://QmHash6", 7 => "ipfs://QmHash7", 8 => "ipfs://QmHash8", 9 => "ipfs://QmHash9", 10 => "ipfs://QmHash10",
            11 => "ipfs://QmHash11", 12 => "ipfs://QmHash12", 13 => "ipfs://QmHash13", 14 => "ipfs://QmHash14", 15 => "ipfs://QmHash15",
            16 => "ipfs://QmHash16", 17 => "ipfs://QmHash17", 18 => "ipfs://QmHash18", 19 => "ipfs://QmHash19", 20 => "ipfs://QmHash20",
            21 => "ipfs://QmHash21", 22 => "ipfs://QmHash22", 23 => "ipfs://QmHash23", 24 => "ipfs://QmHash24", 25 => "ipfs://QmHash25",
            26 => "ipfs://QmHash26", 27 => "ipfs://QmHash27", 28 => "ipfs://QmHash28", 29 => "ipfs://QmHash29", 30 => "ipfs://QmHash30",
            31 => "ipfs://QmHash31", 32 => "ipfs://QmHash32", 33 => "ipfs://QmHash33", 34 => "ipfs://QmHash34", 35 => "ipfs://QmHash35",
            36 => "ipfs://QmHash36", 37 => "ipfs://QmHash37", 38 => "ipfs://QmHash38", 39 => "ipfs://QmHash39", 40 => "ipfs://QmHash40",
            41 => "ipfs://QmHash41", 42 => "ipfs://QmHash42", 43 => "ipfs://QmHash43", 44 => "ipfs://QmHash44", 45 => "ipfs://QmHash45",
            46 => "ipfs://QmHash46", 47 => "ipfs://QmHash47", 48 => "ipfs://QmHash48", 49 => "ipfs://QmHash49", _ => "ipfs://QmHash50",
        };
        let record_id = fixture.create_record("lab", hash);
        assert_eq!(record_id, i as u64);
    }

    let records = fixture.client.list_records(&fixture.patient, &fixture.patient, &0, &100, &None);
    assert_eq!(records.len(), 50);
}

/// Test accessing record with different sensitivity levels
#[test]
fn test_record_access_with_different_sensitivity_levels() {
    let fixture = TestFixture::new();

    // Create records of different types
    let psych_id = fixture.create_record("psychiatric", "ipfs://QmPsych");
    let lab_id = fixture.create_record("lab", "ipfs://QmLab");
    let imaging_id = fixture.create_record("imaging", "ipfs://QmImage");

    // Grant access only to lab and imaging
    fixture.grant_access(std::vec!["lab", "imaging"], 0);

    // Should be able to access lab and imaging
    let lab_record = fixture.client.get_record(&fixture.provider, &fixture.patient, &lab_id);
    assert_eq!(lab_record.data_type, fixture.string("lab"));

    let imaging_record = fixture.client.get_record(&fixture.provider, &fixture.patient, &imaging_id);
    assert_eq!(imaging_record.data_type, fixture.string("imaging"));
}

/// Test that psychiatric records are properly access controlled
#[test]
#[should_panic(expected = "Access denied")]
fn test_psychiatric_record_access_denied() {
    let fixture = TestFixture::new();

    let psych_id = fixture.create_record("psychiatric", "ipfs://QmPsych");
    fixture.grant_access(std::vec!["lab", "imaging"], 0);

    // Should NOT be able to access psychiatric without explicit grant
    fixture.client.get_record(&fixture.provider, &fixture.patient, &psych_id);
}

/// Test bulk access grant to multiple providers
#[test]
fn test_bulk_grant_access() {
    let fixture = TestFixture::new();
    fixture.create_record("imaging", "ipfs://QmImage");

    // Grant to first provider
    fixture.grant_access_to(&fixture.provider, std::vec!["imaging"], 0);

    // Grant to second provider
    fixture.grant_access_to(&fixture.provider2, std::vec!["imaging"], 0);

    // Both should have access
    let data_type = fixture.string("imaging");
    assert!(fixture.client.verify_access(&fixture.patient, &fixture.provider, &data_type));
    assert!(fixture.client.verify_access(&fixture.patient, &fixture.provider2, &data_type));
}

/// Test granting access with invalid record identifier
#[test]
#[should_panic(expected = "Record not found")]
fn test_access_invalid_record_id() {
    let fixture = TestFixture::new();

    // Try to get non-existent record
    fixture.client.get_record(&fixture.patient, &fixture.patient, &999);
}

/// Test access grant with past expiration time
#[test]
fn test_grant_access_with_past_expiration() {
    let fixture = TestFixture::new();

    // In test environment, now() starts at 0, so we use a small past time
    let past_time = 1; // Very short expiration
    fixture.grant_access(std::vec!["lab"], past_time);

    // Advance time past expiration
    fixture.advance_time(100);

    // Should not have access since it's already expired
    assert!(!fixture.verify_access("lab"));
}

/// Test access grant with too long expiration time
#[test]
fn test_grant_access_with_too_long_expiration() {
    let fixture = TestFixture::new();

    // Set expiration far in the future (10 years)
    let future_time = fixture.now() + (10 * 365 * 24 * 60 * 60);
    fixture.grant_access(std::vec!["lab"], future_time);

    // Should have access
    assert!(fixture.verify_access("lab"));
}

/// Test granting access to unregistered provider
#[test]
fn test_grant_access_to_unregistered_provider() {
    let fixture = TestFixture::new();

    // In this implementation, any address can be a provider
    // No registration is required
    let unregistered_provider = soroban_sdk::Address::generate(&fixture.env);

    fixture.grant_access_from_to(&fixture.patient, &unregistered_provider, std::vec!["lab"], 0);

    let data_type = fixture.string("lab");
    assert!(fixture.client.verify_access(&fixture.patient, &unregistered_provider, &data_type));
}
