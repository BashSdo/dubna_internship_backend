pub mod common;

use dubna_internship::api;

#[tokio::test]
async fn retrieves_ticket() {
    let client = common::Client::new().auth("alice", "password").await;

    let ticket = client
        .add_ticket("Ticket 1", "Description 1", 1)
        .await
        .unwrap();
    let ticket = client.get_ticket(ticket.id).await.unwrap();

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
