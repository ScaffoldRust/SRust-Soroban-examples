#![cfg(test)]

use super::utils::*;
use crate::{EventType, RECStatus};
use soroban_sdk::testutils::Address as _;

#[test]
fn retirement_updates_status_history_and_stats() {
    let setup = setup();
    let id = issue(&setup, 80);
    let reason = soroban_sdk::String::from_str(&setup.env, "Used for reporting");
    assert!(setup.client.retire_rec(&id, &setup.issuer, &80, &reason));

    assert_eq!(setup.client.get_rec_status(&id).status, RECStatus::Retired);
    let history = setup.client.get_rec_history(&id);
    assert_eq!(history.len(), 2);
    let event = history.get(1).unwrap();
    assert_eq!(event.event_type, EventType::Retirement);
    assert_eq!(event.notes, reason);
    assert_eq!(setup.client.get_contract_stats(), (1, 80, 0, 80));
}

#[test]
fn transferred_owner_can_retire_rec() {
    let setup = setup();
    let id = issue(&setup, 80);
    let buyer = soroban_sdk::Address::generate(&setup.env);
    setup.client.transfer_rec(&id, &setup.issuer, &buyer, &80);
    setup.client.retire_rec(
        &id,
        &buyer,
        &80,
        &soroban_sdk::String::from_str(&setup.env, "Claimed"),
    );
    assert_eq!(setup.client.get_rec_status(&id).status, RECStatus::Retired);
    assert_eq!(setup.client.get_rec_history(&id).len(), 3);
}

#[test]
#[should_panic(expected = "REC already retired")]
fn retirement_cannot_be_repeated() {
    let setup = setup();
    let id = issue(&setup, 80);
    let reason = soroban_sdk::String::from_str(&setup.env, "Claimed");
    setup.client.retire_rec(&id, &setup.issuer, &80, &reason);
    setup.client.retire_rec(&id, &setup.issuer, &80, &reason);
}

#[test]
#[should_panic(expected = "Capacity must be positive")]
fn retirement_rejects_non_positive_capacity() {
    let setup = setup();
    let id = issue(&setup, 80);
    setup.client.retire_rec(
        &id,
        &setup.issuer,
        &0,
        &soroban_sdk::String::from_str(&setup.env, "Invalid"),
    );
}
