pub mod common;

use dubna_internship::api;

#[tokio::test]
async fn creates_valid_ticket() {
    let ticket = common::Client::new()
        .auth("alice", "password")
        .await
        .add_ticket("Ticket 1", "Description 1", 1)
        .await
        .unwrap();
    assert_eq!(ticket.title, "Ticket 1");
    assert_eq!(ticket.description, "Description 1");
    assert_eq!(ticket.status, api::ticket::Status::Requested);
    assert_eq!(ticket.count, 1);
    assert_eq!(ticket.price, None);
    assert_eq!(ticket.initiator.id, api::user::Id::from(1));
    assert_eq!(ticket.initiator.name, "Alice");
    assert_eq!(ticket.initiator.role, api::user::Role::Initiator);
    assert_eq!(ticket.purchasing_manager, None);
    assert_eq!(ticket.accounting_manager, None);
}

#[tokio::test]
async fn cant_created_when_not_initiator() {
    let status = common::Client::new()
        .auth("bob", "password")
        .await
        .add_ticket("Ticket 1", "Description 1", 1)
        .await
        .unwrap_err();
    assert_eq!(status, reqwest::StatusCode::BAD_REQUEST);
}
