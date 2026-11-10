#![cfg(test)]

use soroban_sdk::{testutils::Address as _, testutils::Ledger as _, Address, String};

use super::utils::*;
use crate::{EnergySource, EventType, RECStatus};

#[test]
fn initialize_registers_issuer_and_starts_stats() {
    let setup = setup();
    let info = setup.client.get_issuer_info(&setup.issuer);
    assert_eq!(info.address, setup.issuer);
    assert_eq!(info.name, String::from_str(&setup.env, "Test issuer"));
    assert!(info.authorized);
    assert_eq!(info.registration_date, setup.env.ledger().timestamp());
    assert_eq!(info.total_issued, 0);
    assert_eq!(setup.client.get_contract_stats(), (0, 0, 0, 0));
}

#[test]
fn issuance_persists_rec_history_issuer_total_and_stats() {
    let setup = setup();
    setup.env.ledger().set_timestamp(42);
    let id = issue(&setup, 125);
    let rec = setup.client.get_rec_status(&id);

    assert_eq!(rec.issuer, setup.issuer);
    assert_eq!(rec.current_owner, setup.issuer);
    assert_eq!(rec.energy_source, EnergySource::Solar);
    assert_eq!(rec.capacity_mwh, 125);
    assert_eq!(rec.status, RECStatus::Issued);
    assert_eq!(rec.issuance_date, 42);
    let history = setup.client.get_rec_history(&id);
    assert_eq!(history.len(), 1);
    assert_eq!(history.get(0).unwrap().event_type, EventType::Issuance);
    assert_eq!(history.get(0).unwrap().capacity_mwh, 125);
    assert_eq!(
        setup.client.get_issuer_info(&setup.issuer).total_issued,
        125
    );
    assert_eq!(setup.client.get_contract_stats(), (1, 125, 0, 0));
}

#[test]
#[should_panic(expected = "Capacity must be positive")]
fn issuance_rejects_non_positive_capacity() {
    let setup = setup();
    issue(&setup, 0);
}

#[test]
#[should_panic(expected = "Issuer not registered")]
fn issuance_rejects_unregistered_issuer() {
    let setup = setup();
    let other = Address::generate(&setup.env);
    setup.client.issue_rec(
        &other,
        &EnergySource::Wind,
        &2024,
        &String::from_str(&setup.env, "Mexico"),
        &1,
        &String::from_str(&setup.env, "I-REC"),
        &soroban_sdk::BytesN::from_array(&setup.env, &[9u8; 32]),
        &soroban_sdk::Map::new(&setup.env),
    );
}
