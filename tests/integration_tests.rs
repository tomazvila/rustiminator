use axum_test::TestServer;
use rustimenator::{
    CreateTagResponse, CreateTaskResponse, CreateTimedEventResponse, GetEventsResponse,
    GetTagsResponse, GetTasksResponse, StopEventResponse, create_app, create_database_pool,
};

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

#[tokio::test]
async fn test_create_task() {
    let pool = create_database_pool(":memory:").await.unwrap();
    let app = create_app(pool);
    let server = TestServer::new(app).unwrap();

    let response = server
        .post("/task")
        .json(&json!({"task": "database-task"}))
        .await;

    response.assert_status_ok();
    let created_task: CreateTaskResponse = response.json();

    assert_eq!(created_task.task, "database-task");
    assert_eq!(created_task.message, "Task created successfully");
    assert!(created_task.id > 0);

    let response = server.get("/tasks").await;

    response.assert_status_ok();
    let task_response: GetTasksResponse = response.json();

    assert_eq!(task_response.count, 1);
    assert_eq!(task_response.tasks.len(), 1);

    let task = &task_response.tasks[0];
    assert_eq!(task.id, created_task.id);
    assert_eq!(task.task, "database-task");
    assert!(task.created_at.is_some());
}

#[tokio::test]
async fn test_create_and_stop_event() {
    let pool = create_database_pool(":memory:").await.unwrap();
    let app = create_app(pool);
    let server = TestServer::new(app).unwrap();

    let tag_response = server.post("/tag").json(&json!({"name": "work"})).await;
    tag_response.assert_status_ok();
    let tag: CreateTagResponse = tag_response.json();

    let task_response = server
        .post("/task")
        .json(&json!({"task": "Write documentation"}))
        .await;
    task_response.assert_status_ok();
    let task: CreateTaskResponse = task_response.json();

    let event_response = server
        .post("/events/start")
        .json(&json!({
            "task_id": task.id,
            "tag_ids": [tag.id]
        }))
        .await;
    event_response.assert_status_ok();
    let event: CreateTimedEventResponse = event_response.json();
    assert_eq!(event.message, "Event started successfully");
    assert!(event.id > 0);

    let get_events_response = server.get("/events").await;
    get_events_response.assert_status_ok();
    let events: GetEventsResponse = get_events_response.json();
    assert_eq!(events.count, 1);
    assert_eq!(events.events[0].id, event.id);
    assert_eq!(events.events[0].task_id, task.id);
    assert!(events.events[0].task.is_some());
    assert_eq!(
        events.events[0].task.as_ref().unwrap().task,
        "Write documentation"
    );
    assert_eq!(events.events[0].tags.len(), 1);
    assert_eq!(events.events[0].tags[0].name, "work");
    assert!(events.events[0].stopped_at.is_none());

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let stop_response = server.post(&format!("/events/stop/{}", event.id)).await;
    stop_response.assert_status_ok();
    let stop_result: StopEventResponse = stop_response.json();
    assert_eq!(stop_result.id, event.id);
    assert_eq!(stop_result.message, "Event stopped successfully");
    assert!(stop_result.duration_seconds >= 0);

    let get_events_response = server.get("/events").await;
    get_events_response.assert_status_ok();
    let events: GetEventsResponse = get_events_response.json();
    assert_eq!(events.count, 0);
}

#[tokio::test]
async fn test_create_event_with_multiple_tags() {
    let pool = create_database_pool(":memory:").await.unwrap();
    let app = create_app(pool);
    let server = TestServer::new(app).unwrap();

    let tag1_response = server.post("/tag").json(&json!({"name": "urgent"})).await;
    tag1_response.assert_status_ok();
    let tag1: CreateTagResponse = tag1_response.json();

    let tag2_response = server.post("/tag").json(&json!({"name": "bug-fix"})).await;
    tag2_response.assert_status_ok();
    let tag2: CreateTagResponse = tag2_response.json();

    let task_response = server
        .post("/task")
        .json(&json!({"task": "Fix critical bug"}))
        .await;
    task_response.assert_status_ok();
    let task: CreateTaskResponse = task_response.json();

    let event_response = server
        .post("/events/start")
        .json(&json!({
            "task_id": task.id,
            "tag_ids": [tag1.id, tag2.id]
        }))
        .await;
    event_response.assert_status_ok();

    let event: CreateTimedEventResponse = event_response.json();
    assert_eq!(event.message, "Event started successfully");

    let get_events_response = server.get("/events").await;
    get_events_response.assert_status_ok();
    let events: GetEventsResponse = get_events_response.json();
    assert_eq!(events.count, 1);
    assert_eq!(events.events[0].tags.len(), 2);

    let tag_names: Vec<String> = events.events[0]
        .tags
        .iter()
        .map(|t| t.name.clone())
        .collect();
    assert!(tag_names.contains(&"urgent".to_string()));
    assert!(tag_names.contains(&"bug-fix".to_string()));
}

#[tokio::test]
async fn test_create_event_with_invalid_task() {
    let pool = create_database_pool(":memory:").await.unwrap();
    let app = create_app(pool);
    let server = TestServer::new(app).unwrap();

    let tag_response = server.post("/tag").json(&json!({"name": "test-tag"})).await;
    tag_response.assert_status_ok();
    let tag: CreateTagResponse = tag_response.json();

    let event_response = server
        .post("/events/start")
        .json(&json!({
            "task_id": 9999,
            "tag_ids": [tag.id]
        }))
        .await;
    event_response.assert_status_bad_request();
}

#[tokio::test]
async fn test_create_event_with_invalid_tag() {
    let pool = create_database_pool(":memory:").await.unwrap();
    let app = create_app(pool);
    let server = TestServer::new(app).unwrap();

    let task_response = server
        .post("/task")
        .json(&json!({"task": "test-task"}))
        .await;
    task_response.assert_status_ok();
    let task: CreateTaskResponse = task_response.json();

    let event_response = server
        .post("/events/start")
        .json(&json!({
            "task_id": task.id,
            "tag_ids": [9999]
        }))
        .await;
    event_response.assert_status_bad_request();
}

#[tokio::test]
async fn test_stop_non_existent_event() {
    let pool = create_database_pool(":memory:").await.unwrap();
    let app = create_app(pool);
    let server = TestServer::new(app).unwrap();

    let stop_response = server.post("/events/stop/9999").await;
    stop_response.assert_status_not_found();
}

#[tokio::test]
async fn test_stop_already_stopped_event() {
    let pool = create_database_pool(":memory:").await.unwrap();
    let app = create_app(pool);
    let server = TestServer::new(app).unwrap();

    let tag_response = server.post("/tag").json(&json!({"name": "test"})).await;
    tag_response.assert_status_ok();
    let tag: CreateTagResponse = tag_response.json();

    let task_response = server
        .post("/task")
        .json(&json!({"task": "test task"}))
        .await;
    task_response.assert_status_ok();
    let task: CreateTaskResponse = task_response.json();

    let event_response = server
        .post("/events/start")
        .json(&json!({
            "task_id": task.id,
            "tag_ids": [tag.id]
        }))
        .await;
    event_response.assert_status_ok();
    let event: CreateTimedEventResponse = event_response.json();

    let stop_response = server.post(&format!("/events/stop/{}", event.id)).await;
    stop_response.assert_status_ok();

    let second_stop_response = server.post(&format!("/events/stop/{}", event.id)).await;
    second_stop_response.assert_status_not_found();
}

#[tokio::test]
async fn test_multiple_running_events() {
    let pool = create_database_pool(":memory:").await.unwrap();
    let app = create_app(pool);
    let server = TestServer::new(app).unwrap();

    let tag1_response = server
        .post("/tag")
        .json(&json!({"name": "development"}))
        .await;
    tag1_response.assert_status_ok();
    let tag1: CreateTagResponse = tag1_response.json();

    let tag2_response = server.post("/tag").json(&json!({"name": "testing"})).await;
    tag2_response.assert_status_ok();
    let tag2: CreateTagResponse = tag2_response.json();

    let task1_response = server
        .post("/task")
        .json(&json!({"task": "Implement feature X"}))
        .await;
    task1_response.assert_status_ok();
    let task1: CreateTaskResponse = task1_response.json();

    let task2_response = server
        .post("/task")
        .json(&json!({"task": "Write unit tests"}))
        .await;
    task2_response.assert_status_ok();
    let task2: CreateTaskResponse = task2_response.json();

    let event1_response = server
        .post("/events/start")
        .json(&json!({
            "task_id": task1.id,
            "tag_ids": [tag1.id]
        }))
        .await;
    event1_response.assert_status_ok();
    let event1: CreateTimedEventResponse = event1_response.json();

    let event2_response = server
        .post("/events/start")
        .json(&json!({
            "task_id": task2.id,
            "tag_ids": [tag2.id]
        }))
        .await;
    event2_response.assert_status_ok();
    let event2: CreateTimedEventResponse = event2_response.json();

    let get_events_response = server.get("/events").await;
    get_events_response.assert_status_ok();
    let events: GetEventsResponse = get_events_response.json();
    assert_eq!(events.count, 2);

    let stop_response = server.post(&format!("/events/stop/{}", event1.id)).await;
    stop_response.assert_status_ok();

    let get_events_response = server.get("/events").await;
    get_events_response.assert_status_ok();
    let events: GetEventsResponse = get_events_response.json();
    assert_eq!(events.count, 1);
    assert_eq!(events.events[0].id, event2.id);
}
