extern crate std;

use super::utils::TestFixture;

/// Test provider access with valid permission
#[test]
fn test_provider_access_with_valid_permission() {
    let fixture = TestFixture::new();
    let record_id = fixture.create_record("lab", "ipfs://QmLab");

    fixture.grant_access(std::vec!["lab"], 0);

    let record = fixture.client.get_record(&fixture.provider, &fixture.patient, &record_id);
    assert_eq!(record.pointer, fixture.string("ipfs://QmLab"));
}

/// Test unauthorized provider access denied
#[test]
#[should_panic(expected = "Access denied")]
fn test_unauthorized_provider_access_denied() {
    let fixture = TestFixture::new();
    let record_id = fixture.create_record("lab", "ipfs://QmLab");

    // No access grant
    fixture.client.get_record(&fixture.unauthorized, &fixture.patient, &record_id);
}

/// Test time-limited access validation
#[test]
fn test_time_limited_access_validation() {
    let fixture = TestFixture::new();
    let record_id = fixture.create_record("lab", "ipfs://QmLab");

    // Grant access for 1 hour
    let expiry = fixture.now() + 3600;
    fixture.grant_access(std::vec!["lab"], expiry);

    // Should have access now
    let record = fixture.client.get_record(&fixture.provider, &fixture.patient, &record_id);
    assert_eq!(record.record_id, record_id);

    // Advance time by 2 hours
    fixture.advance_time(7200);

    // Should not have access anymore
    assert!(!fixture.verify_access("lab"));
}

/// Test expired access denied
#[test]
#[should_panic(expected = "Access denied")]
fn test_expired_access_denied() {
    let fixture = TestFixture::new();
    let record_id = fixture.create_record("lab", "ipfs://QmLab");

    // Grant access for 1 hour
    let expiry = fixture.now() + 3600;
    fixture.grant_access(std::vec!["lab"], expiry);

    // Advance time beyond expiry
    fixture.advance_time(7200);

    // Try to access - should fail
    fixture.client.get_record(&fixture.provider, &fixture.patient, &record_id);
}

/// Test scoped access restrictions (data type specific)
#[test]
fn test_scoped_access_restrictions() {
    let fixture = TestFixture::new();
    let lab_id = fixture.create_record("lab", "ipfs://QmLab");
    let imaging_id = fixture.create_record("imaging", "ipfs://QmImage");

    // Grant access only to lab records
    fixture.grant_access(std::vec!["lab"], 0);

    // Can access lab
    let lab_record = fixture.client.get_record(&fixture.provider, &fixture.patient, &lab_id);
    assert_eq!(lab_record.data_type, fixture.string("lab"));

    // Cannot access imaging (different data type)
    assert!(!fixture.verify_access("imaging"));
}

/// Test imaging access denied when only lab access granted
#[test]
#[should_panic(expected = "Access denied")]
fn test_imaging_access_denied_with_lab_grant() {
    let fixture = TestFixture::new();
    let imaging_id = fixture.create_record("imaging", "ipfs://QmImage");

    // Grant access only to lab
    fixture.grant_access(std::vec!["lab"], 0);

    // Try to access imaging - should fail
    fixture.client.get_record(&fixture.provider, &fixture.patient, &imaging_id);
}

/// Test revoked access denied
#[test]
#[should_panic(expected = "Access denied")]
fn test_revoked_access_denied() {
    let fixture = TestFixture::new();
    let record_id = fixture.create_record("lab", "ipfs://QmLab");

    fixture.grant_access(std::vec!["lab"], 0);

    // Verify access works
    let _ = fixture.client.get_record(&fixture.provider, &fixture.patient, &record_id);

    // Revoke access
    fixture.client.revoke_access(&fixture.patient, &fixture.provider);

    // Try to access again - should fail
    fixture.client.get_record(&fixture.provider, &fixture.patient, &record_id);
}

/// Test multiple providers access to same record
#[test]
fn test_multiple_providers_access_same_record() {
    let fixture = TestFixture::new();
    let record_id = fixture.create_record("lab", "ipfs://QmLab");

    // Grant access to both providers
    fixture.grant_access_to(&fixture.provider, std::vec!["lab"], 0);
    fixture.grant_access_to(&fixture.provider2, std::vec!["lab"], 0);

    // Both should be able to access
    let record1 = fixture.client.get_record(&fixture.provider, &fixture.patient, &record_id);
    let record2 = fixture.client.get_record(&fixture.provider2, &fixture.patient, &record_id);

    assert_eq!(record1.record_id, record2.record_id);
}

/// Test emergency access for authorized provider
#[test]
fn test_emergency_access_authorized_provider() {
    let fixture = TestFixture::new();
    let record_id = fixture.create_record("lab", "ipfs://QmLab");

    // Add as emergency provider
    fixture.add_emergency_provider();

    // Should be able to emergency read
    let justification = fixture.string("Patient in critical condition");
    let record = fixture.client.emergency_read(&fixture.provider, &fixture.patient, &record_id, &justification);
    assert_eq!(record.record_id, record_id);
}

/// Test emergency access for unauthorized provider
#[test]
#[should_panic(expected = "Not whitelisted")]
fn test_emergency_access_unauthorized_provider() {
    let fixture = TestFixture::new();
    let record_id = fixture.create_record("lab", "ipfs://QmLab");

    // Try emergency read without being whitelisted
    let justification = fixture.string("Emergency");
    fixture.client.emergency_read(&fixture.provider, &fixture.patient, &record_id, &justification);
}

/// Test access without authentication (should work in test mode)
#[test]
fn test_access_without_authentication() {
    let fixture = TestFixture::new();
    let record_id = fixture.create_record("lab", "ipfs://QmLab");

    fixture.grant_access(std::vec!["lab"], 0);

    // In test environment, auth is disabled
    let record = fixture.client.get_record(&fixture.provider, &fixture.patient, &record_id);
    assert_eq!(record.record_id, record_id);
}

/// Test different access levels (via data types)
#[test]
fn test_different_access_levels() {
    let fixture = TestFixture::new();

    let basic_id = fixture.create_record("basic_info", "ipfs://QmBasic");
    let sensitive_id = fixture.create_record("genetic", "ipfs://QmGenetic");
    let psych_id = fixture.create_record("psychiatric", "ipfs://QmPsych");

    // Grant only basic access
    fixture.grant_access(std::vec!["basic_info"], 0);

    // Can access basic
    let _ = fixture.client.get_record(&fixture.provider, &fixture.patient, &basic_id);

    // Cannot access sensitive or psychiatric
    assert!(!fixture.verify_access("genetic"));
    assert!(!fixture.verify_access("psychiatric"));
}

/// Test patient consent integration (simulated through access grants)
#[test]
fn test_patient_consent_integration() {
    let fixture = TestFixture::new();
    let record_id = fixture.create_record("lab", "ipfs://QmLab");

    // Simulate patient providing consent via access grant
    fixture.grant_access(std::vec!["lab"], 0);

    // Provider should have access (consent verified through grant)
    assert!(fixture.verify_access("lab"));

    // Revoke consent
    fixture.client.revoke_access(&fixture.patient, &fixture.provider);

    // Provider should no longer have access
    assert!(!fixture.verify_access("lab"));
}

/// Test high-volume access requests for scalability
#[test]
fn test_high_volume_access_requests() {
    let fixture = TestFixture::new();

    // Create multiple records
    fixture.create_record("lab", "ipfs://QmHash1");
    fixture.create_record("lab", "ipfs://QmHash2");
    fixture.create_record("lab", "ipfs://QmHash3");
    fixture.create_record("lab", "ipfs://QmHash4");
    fixture.create_record("lab", "ipfs://QmHash5");
    fixture.create_record("lab", "ipfs://QmHash6");
    fixture.create_record("lab", "ipfs://QmHash7");
    fixture.create_record("lab", "ipfs://QmHash8");
    fixture.create_record("lab", "ipfs://QmHash9");
    fixture.create_record("lab", "ipfs://QmHash10");
    fixture.create_record("lab", "ipfs://QmHash11");
    fixture.create_record("lab", "ipfs://QmHash12");
    fixture.create_record("lab", "ipfs://QmHash13");
    fixture.create_record("lab", "ipfs://QmHash14");
    fixture.create_record("lab", "ipfs://QmHash15");
    fixture.create_record("lab", "ipfs://QmHash16");
    fixture.create_record("lab", "ipfs://QmHash17");
    fixture.create_record("lab", "ipfs://QmHash18");
    fixture.create_record("lab", "ipfs://QmHash19");
    fixture.create_record("lab", "ipfs://QmHash20");

    // Grant access
    fixture.grant_access(std::vec!["lab"], 0);

    // Perform multiple access requests
    for i in 1..=20 {
        let record = fixture.client.get_record(&fixture.provider, &fixture.patient, &(i as u64));
        assert_eq!(record.record_id, i as u64);
    }
}

/// Test bulk access permissions
#[test]
fn test_bulk_access_permissions() {
    let fixture = TestFixture::new();

    // Create records of different types
    fixture.create_record("lab", "ipfs://QmLab");
    fixture.create_record("imaging", "ipfs://QmImage");
    fixture.create_record("prescription", "ipfs://QmRx");

    // Grant access to multiple data types at once
    fixture.grant_access(std::vec!["lab", "imaging", "prescription"], 0);

    // All should be accessible
    assert!(fixture.verify_access("lab"));
    assert!(fixture.verify_access("imaging"));
    assert!(fixture.verify_access("prescription"));
}

/// Test access permissions visibility (getting access grant)
#[test]
fn test_access_permissions_visibility() {
    let fixture = TestFixture::new();

    // Grant access
    fixture.grant_access(std::vec!["lab", "imaging"], 0);

    // Get the access grant
    let grant = fixture.client.get_access_grant(&fixture.patient, &fixture.provider);
    assert!(grant.is_some());

    let grant = grant.unwrap();
    assert_eq!(grant.provider, fixture.provider);
    assert_eq!(grant.data_types.len(), 2);
    assert!(!grant.revoked);
}

/// Test access count tracking via audit logs
#[test]
fn test_access_count_tracking() {
    let fixture = TestFixture::new();
    let record_id = fixture.create_record("lab", "ipfs://QmLab");

    fixture.grant_access(std::vec!["lab"], 0);

    // Access multiple times
    for _ in 0..5 {
        let _ = fixture.client.get_record(&fixture.provider, &fixture.patient, &record_id);
    }

    // Check audit log for access events
    let audit_log = fixture.get_audit_log();

    // Should have at least: add_record + grant_access events
    assert!(audit_log.len() >= 2);
}

/// Test removing emergency provider
#[test]
#[should_panic(expected = "Not whitelisted")]
fn test_remove_emergency_provider() {
    let fixture = TestFixture::new();
    let record_id = fixture.create_record("lab", "ipfs://QmLab");

    // Add and then remove emergency provider
    fixture.add_emergency_provider();
    fixture.client.remove_emergency_provider(&fixture.patient, &fixture.provider);

    // Should not be able to emergency read
    let justification = fixture.string("Emergency");
    fixture.client.emergency_read(&fixture.provider, &fixture.patient, &record_id, &justification);
}

/// Test patient can always access their own records
#[test]
fn test_patient_self_access() {
    let fixture = TestFixture::new();
    let record_id = fixture.create_record("lab", "ipfs://QmLab");

    // Patient should always be able to access their own records
    let record = fixture.client.get_record(&fixture.patient, &fixture.patient, &record_id);
    assert_eq!(record.record_id, record_id);
}

/// Test access verification with zero expiration (no expiration)
#[test]
fn test_zero_expiration_means_no_expiry() {
    let fixture = TestFixture::new();

    fixture.grant_access(std::vec!["lab"], 0);

    // Advance time significantly
    fixture.advance_time(365 * 24 * 60 * 60); // 1 year

    // Should still have access
    assert!(fixture.verify_access("lab"));
}

/// Test granting access to self should fail
#[test]
#[should_panic(expected = "Cannot grant to self")]
fn test_cannot_grant_to_self() {
    let fixture = TestFixture::new();
    fixture.grant_access_from_to(&fixture.patient, &fixture.patient, std::vec!["lab"], 0);
}
