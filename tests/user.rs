pub mod common;

use dubna_internship::api;
use reqwest::StatusCode;

#[tokio::test]
async fn retreieves_current_user() {
    let user = common::Client::new()
        .auth("alice", "password")
        .await
        .user()
        .await
        .unwrap();
    assert_eq!(user.id, api::user::Id::from(1));
    assert_eq!(user.name, "Alice");
    assert_eq!(user.role, api::user::Role::Initiator);
}

#[tokio::test]
async fn fails_when_unauthorized() {
    let status = common::Client::new().user().await.unwrap_err();
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}
