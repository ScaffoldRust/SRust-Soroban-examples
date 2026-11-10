#![cfg(test)]

use super::utils::*;
use crate::{EventType, RECStatus};
use soroban_sdk::{testutils::Address as _, Address};

#[test]
fn transfer_updates_owner_status_history_and_stats() {
    let setup = setup();
    let id = issue(&setup, 100);
    let buyer = Address::generate(&setup.env);
    assert!(setup.client.transfer_rec(&id, &setup.issuer, &buyer, &60));

    let rec = setup.client.get_rec_status(&id);
    assert_eq!(rec.current_owner, buyer);
    assert_eq!(rec.status, RECStatus::Transferred);
    let history = setup.client.get_rec_history(&id);
    assert_eq!(history.len(), 2);
    let event = history.get(1).unwrap();
    assert_eq!(event.event_type, EventType::Transfer);
    assert_eq!(event.capacity_mwh, 60);
    assert_eq!(setup.client.get_contract_stats(), (1, 100, 60, 0));
}

#[test]
#[should_panic(expected = "Not the owner")]
fn transfer_rejects_wrong_owner() {
    let setup = setup();
    let id = issue(&setup, 100);
    let other = Address::generate(&setup.env);
    let buyer = Address::generate(&setup.env);
    setup.client.transfer_rec(&id, &other, &buyer, &10);
}

#[test]
#[should_panic(expected = "Insufficient capacity")]
fn transfer_rejects_excess_capacity() {
    let setup = setup();
    let id = issue(&setup, 100);
    let buyer = Address::generate(&setup.env);
    setup.client.transfer_rec(&id, &setup.issuer, &buyer, &101);
}
