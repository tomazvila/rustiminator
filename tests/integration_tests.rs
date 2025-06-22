use axum_test::TestServer;
use rustimenator::{CreateTagResponse, GetTagsResponse, create_app, create_database_pool};

use serde_json::json;

#[tokio::test]
async fn test_create_tag() {
    let pool = create_database_pool(":memory:").await.unwrap();
    let app = create_app(pool);
    let server = TestServer::new(app).unwrap();

    let response = server
        .post("/tag")
        .json(&json!({"name": "database-tag"}))
        .await;

    response.assert_status_ok();
    let created_tag: CreateTagResponse = response.json();

    assert_eq!(created_tag.name, "database-tag");
    assert_eq!(created_tag.message, "Tag created successfully");
    assert!(created_tag.id > 0);

    let response = server.get("/tags").await;

    response.assert_status_ok();
    let tags_response: GetTagsResponse = response.json();

    assert_eq!(tags_response.count, 1);
    assert_eq!(tags_response.tags.len(), 1);

    let tag = &tags_response.tags[0];
    assert_eq!(tag.id, created_tag.id);
    assert_eq!(tag.name, "database-tag");
    assert!(tag.created_at.is_some());
}
