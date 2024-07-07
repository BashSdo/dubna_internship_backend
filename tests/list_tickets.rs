pub mod common;

use dubna_internship::api;

// NOTE: Should be executed as serial test to avoid conflicts with other tests.
#[tokio::test]
async fn limit_tickets() {
    let client = common::Client::new().auth("alice", "password").await;

    client
        .add_ticket("Ticket 1", "Description 1", 1)
        .await
        .unwrap();
    client
        .add_ticket("Ticket 2", "Description 2", 2)
        .await
        .unwrap();
    client
        .add_ticket("Ticket 3", "Description 3", 3)
        .await
        .unwrap();
    client
        .add_ticket("Ticket 4", "Description 4", 4)
        .await
        .unwrap();

    let res = client.get_tickets(0, 2).await.map(|list| list.tickets);
    match res.as_deref() {
        Ok([first, second]) => {
            assert_eq!(first.title, "Ticket 4");
            assert_eq!(first.description, "Description 4");
            assert_eq!(first.status, api::ticket::Status::Requested);
            assert_eq!(first.count, 4);
            assert_eq!(first.price, None);
            assert_eq!(first.initiator.id, api::user::Id::from(1));
            assert_eq!(first.initiator.name, "Alice");
            assert_eq!(first.initiator.role, api::user::Role::Initiator);
            assert_eq!(first.purchasing_manager, None);
            assert_eq!(first.accounting_manager, None);

            assert_eq!(second.title, "Ticket 3");
            assert_eq!(second.description, "Description 3");
            assert_eq!(second.status, api::ticket::Status::Requested);
            assert_eq!(second.count, 3);
            assert_eq!(second.price, None);
            assert_eq!(second.initiator.id, api::user::Id::from(1));
            assert_eq!(second.initiator.name, "Alice");
            assert_eq!(second.initiator.role, api::user::Role::Initiator);
            assert_eq!(second.purchasing_manager, None);
            assert_eq!(second.accounting_manager, None);
        }
        found => panic!("expected two tickets, found {found:?}"),
    }
}

// NOTE: Should be executed as serial test to avoid conflicts with other tests.
#[tokio::test]
async fn skips_tickets() {
    let client = common::Client::new().auth("alice", "password").await;

    client
        .add_ticket("Ticket 1", "Description 1", 1)
        .await
        .unwrap();
    client
        .add_ticket("Ticket 2", "Description 2", 2)
        .await
        .unwrap();
    client
        .add_ticket("Ticket 3", "Description 3", 3)
        .await
        .unwrap();
    client
        .add_ticket("Ticket 4", "Description 4", 4)
        .await
        .unwrap();

    let res = client.get_tickets(2, 2).await.map(|list| list.tickets);
    match res.as_deref() {
        Ok([first, second]) => {
            assert_eq!(first.title, "Ticket 2");
            assert_eq!(first.description, "Description 2");
            assert_eq!(first.status, api::ticket::Status::Requested);
            assert_eq!(first.count, 2);
            assert_eq!(first.price, None);
            assert_eq!(first.initiator.id, api::user::Id::from(1));
            assert_eq!(first.initiator.name, "Alice");
            assert_eq!(first.purchasing_manager, None);
            assert_eq!(first.accounting_manager, None);

            assert_eq!(second.title, "Ticket 1");
            assert_eq!(second.description, "Description 1");
            assert_eq!(second.status, api::ticket::Status::Requested);
            assert_eq!(second.count, 1);
            assert_eq!(second.price, None);
            assert_eq!(second.initiator.id, api::user::Id::from(1));
            assert_eq!(second.initiator.name, "Alice");
            assert_eq!(second.purchasing_manager, None);
            assert_eq!(second.accounting_manager, None);
        }
        found => panic!("expected two tickets, found {found:?}"),
    }
}
