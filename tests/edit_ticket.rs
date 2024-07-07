pub mod common;

use dubna_internship::api;
use reqwest::StatusCode;

#[tokio::test]
async fn edits_ticket_title() {
    let alice = common::Client::new().auth("alice", "password").await;
    let ticket = alice
        .add_ticket("Ticket 1", "Description 1", 1)
        .await
        .unwrap();
    let ticket = alice.edit_ticket_title(ticket.id, "Title 2").await.unwrap();
    assert_eq!(ticket.title, "Title 2");
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
async fn cant_edits_ticket_title_when_confirmed() {
    let alice = common::Client::new().auth("alice", "password").await;
    let ticket = alice
        .add_ticket("Ticket 1", "Description 1", 1)
        .await
        .unwrap();

    let bob = common::Client::new().auth("bob", "password").await;
    bob.confirm_ticket(ticket.id, 100).await.unwrap();

    let status = alice
        .edit_ticket_title(ticket.id, "Title 2")
        .await
        .unwrap_err();
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn cant_edits_ticket_title_when_not_initiator() {
    let alice = common::Client::new().auth("alice", "password").await;
    let ticket = alice
        .add_ticket("Ticket 1", "Description 1", 1)
        .await
        .unwrap();

    let bob = common::Client::new().auth("bob", "password").await;
    let status = bob
        .edit_ticket_title(ticket.id, "Title 2")
        .await
        .unwrap_err();
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn edits_ticket_description() {
    let alice = common::Client::new().auth("alice", "password").await;
    let ticket = alice
        .add_ticket("Ticket 1", "Description 1", 1)
        .await
        .unwrap();

    let bob = common::Client::new().auth("bob", "password").await;
    let ticket = bob
        .edit_ticket_description(ticket.id, "Description 2")
        .await
        .unwrap();

    assert_eq!(ticket.title, "Ticket 1");
    assert_eq!(ticket.description, "Description 2");
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
async fn cancels_ticket() {
    let alice = common::Client::new().auth("alice", "password").await;
    let ticket = alice
        .add_ticket("Ticket 1", "Description 1", 1)
        .await
        .unwrap();
    let ticket = alice.cancel_ticket(ticket.id).await.unwrap();
    assert_eq!(ticket.title, "Ticket 1");
    assert_eq!(ticket.description, "Description 1");
    assert_eq!(ticket.status, api::ticket::Status::Cancelled);
    assert_eq!(ticket.count, 1);
    assert_eq!(ticket.price, None);
    assert_eq!(ticket.initiator.id, api::user::Id::from(1));
    assert_eq!(ticket.initiator.name, "Alice");
    assert_eq!(ticket.initiator.role, api::user::Role::Initiator);
    assert_eq!(ticket.purchasing_manager, None);
    assert_eq!(ticket.accounting_manager, None);
}

#[tokio::test]
async fn cant_cancel_ticket_when_confirmed() {
    let alice = common::Client::new().auth("alice", "password").await;
    let ticket = alice
        .add_ticket("Ticket 1", "Description 1", 1)
        .await
        .unwrap();

    let bob = common::Client::new().auth("bob", "password").await;
    bob.confirm_ticket(ticket.id, 100).await.unwrap();

    let status = alice.cancel_ticket(ticket.id).await.unwrap_err();
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn cant_cancel_ticket_when_not_initiator() {
    let alice = common::Client::new().auth("alice", "password").await;
    let ticket = alice
        .add_ticket("Ticket 1", "Description 1", 1)
        .await
        .unwrap();

    let bob = common::Client::new().auth("bob", "password").await;
    let status = bob.cancel_ticket(ticket.id).await.unwrap_err();
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn confirms_ticket() {
    let alice = common::Client::new().auth("alice", "password").await;
    let ticket = alice
        .add_ticket("Ticket 1", "Description 1", 1)
        .await
        .unwrap();

    let bob = common::Client::new().auth("bob", "password").await;
    let ticket = bob.confirm_ticket(ticket.id, 100).await.unwrap();

    assert_eq!(ticket.title, "Ticket 1");
    assert_eq!(ticket.description, "Description 1");
    assert_eq!(ticket.status, api::ticket::Status::Confirmed);
    assert_eq!(ticket.count, 1);
    assert_eq!(ticket.price, Some(100.0));
    assert_eq!(ticket.initiator.id, api::user::Id::from(1));
    assert_eq!(ticket.initiator.name, "Alice");
    assert_eq!(ticket.initiator.role, api::user::Role::Initiator);
    assert_eq!(
        ticket.purchasing_manager.as_ref().map(|u| u.id),
        Some(api::user::Id::from(2))
    );
    assert_eq!(
        ticket.purchasing_manager.as_ref().map(|u| u.name.as_str()),
        Some("Bob")
    );
    assert_eq!(
        ticket.purchasing_manager.as_ref().map(|u| u.role),
        Some(api::user::Role::PurchasingManager)
    );
    assert_eq!(ticket.accounting_manager, None);
}

#[tokio::test]
async fn cant_confirm_ticket_when_not_purchasing_manager() {
    let alice = common::Client::new().auth("alice", "password").await;
    let ticket = alice
        .add_ticket("Ticket 1", "Description 1", 1)
        .await
        .unwrap();
    let status = alice.confirm_ticket(ticket.id, 100).await.unwrap_err();
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn cant_confirm_ticket_when_not_requested() {
    let alice = common::Client::new().auth("alice", "password").await;
    let ticket = alice
        .add_ticket("Ticket 1", "Description 1", 1)
        .await
        .unwrap();
    alice.cancel_ticket(ticket.id).await.unwrap();

    let bob = common::Client::new().auth("bob", "password").await;
    let status = bob.confirm_ticket(ticket.id, 100).await.unwrap_err();
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn denies_ticket() {
    let alice = common::Client::new().auth("alice", "password").await;
    let ticket = alice
        .add_ticket("Ticket 1", "Description 1", 1)
        .await
        .unwrap();

    let bob = common::Client::new().auth("bob", "password").await;
    let ticket = bob.deny_ticket(ticket.id).await.unwrap();

    assert_eq!(ticket.title, "Ticket 1");
    assert_eq!(ticket.description, "Description 1");
    assert_eq!(ticket.status, api::ticket::Status::Denied);
    assert_eq!(ticket.count, 1);
    assert_eq!(ticket.price, None);
    assert_eq!(ticket.initiator.id, api::user::Id::from(1));
    assert_eq!(ticket.initiator.name, "Alice");
    assert_eq!(ticket.initiator.role, api::user::Role::Initiator);
    assert_eq!(
        ticket.purchasing_manager.as_ref().map(|u| u.id),
        Some(api::user::Id::from(2))
    );
    assert_eq!(
        ticket.purchasing_manager.as_ref().map(|u| u.name.as_str()),
        Some("Bob")
    );
    assert_eq!(
        ticket.purchasing_manager.as_ref().map(|u| u.role),
        Some(api::user::Role::PurchasingManager)
    );
    assert_eq!(ticket.accounting_manager, None);
}

#[tokio::test]
async fn cant_deny_ticket_when_not_purchasing_manager() {
    let alice = common::Client::new().auth("alice", "password").await;
    let ticket = alice
        .add_ticket("Ticket 1", "Description 1", 1)
        .await
        .unwrap();
    let status = alice.deny_ticket(ticket.id).await.unwrap_err();
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn cant_deny_ticket_when_not_requested() {
    let alice = common::Client::new().auth("alice", "password").await;
    let ticket = alice
        .add_ticket("Ticket 1", "Description 1", 1)
        .await
        .unwrap();
    alice.cancel_ticket(ticket.id).await.unwrap();

    let bob = common::Client::new().auth("bob", "password").await;
    let status = bob.deny_ticket(ticket.id).await.unwrap_err();
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn mark_ticket_as_paid() {
    let alice = common::Client::new().auth("alice", "password").await;
    let ticket = alice
        .add_ticket("Ticket 1", "Description 1", 1)
        .await
        .unwrap();

    let bob = common::Client::new().auth("bob", "password").await;
    bob.confirm_ticket(ticket.id, 100).await.unwrap();

    let charlie = common::Client::new().auth("charlie", "password").await;
    let ticket = charlie.mark_ticket_as_paid(ticket.id).await.unwrap();

    assert_eq!(ticket.title, "Ticket 1");
    assert_eq!(ticket.description, "Description 1");
    assert_eq!(ticket.status, api::ticket::Status::PaymentCompleted);
    assert_eq!(ticket.count, 1);
    assert_eq!(ticket.price, Some(100.0));
    assert_eq!(ticket.initiator.id, api::user::Id::from(1));
    assert_eq!(ticket.initiator.name, "Alice");
    assert_eq!(ticket.initiator.role, api::user::Role::Initiator);
    assert_eq!(
        ticket.purchasing_manager.as_ref().map(|u| u.id),
        Some(api::user::Id::from(2))
    );
    assert_eq!(
        ticket.purchasing_manager.as_ref().map(|u| u.name.as_str()),
        Some("Bob")
    );
    assert_eq!(
        ticket.purchasing_manager.as_ref().map(|u| u.role),
        Some(api::user::Role::PurchasingManager)
    );
    assert_eq!(
        ticket.accounting_manager.as_ref().map(|u| u.id),
        Some(api::user::Id::from(3))
    );
    assert_eq!(
        ticket.accounting_manager.as_ref().map(|u| u.name.as_str()),
        Some("Charlie")
    );
    assert_eq!(
        ticket.accounting_manager.as_ref().map(|u| u.role),
        Some(api::user::Role::AccountingManager)
    );
}

#[tokio::test]
async fn cant_mark_ticket_as_paid_when_not_accounting_manager() {
    let alice = common::Client::new().auth("alice", "password").await;
    let ticket = alice
        .add_ticket("Ticket 1", "Description 1", 1)
        .await
        .unwrap();

    let bob = common::Client::new().auth("bob", "password").await;
    bob.confirm_ticket(ticket.id, 100).await.unwrap();
    let status = bob.mark_ticket_as_paid(ticket.id).await.unwrap_err();
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn cant_mark_ticket_as_paid_when_not_confirmed() {
    let alice = common::Client::new().auth("alice", "password").await;
    let ticket = alice
        .add_ticket("Ticket 1", "Description 1", 1)
        .await
        .unwrap();

    let charlie = common::Client::new().auth("charlie", "password").await;
    let status = charlie.mark_ticket_as_paid(ticket.id).await.unwrap_err();
    assert_eq!(status, StatusCode::BAD_REQUEST);
}
