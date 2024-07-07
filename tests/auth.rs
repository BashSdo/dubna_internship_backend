pub mod common;

#[tokio::test]
async fn retreieves_access_token() {
    let client = common::Client::new().auth("alice", "password").await;
    assert!(client.auth_token.is_some());
}
