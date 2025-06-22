use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite, SqlitePool};

#[derive(Deserialize)]
pub struct CreateTagRequest {
    pub name: String,
}

#[derive(Deserialize)]
pub struct CreateTaskRequest {
    pub task: String,
}

#[derive(Serialize, Deserialize)]
pub struct CreateTagResponse {
    pub id: i64,
    pub name: String,
    pub message: String,
}

#[derive(Serialize, Deserialize)]
pub struct CreateTaskResponse {
    pub id: i64,
    pub task: String,
    pub message: String,
}

#[derive(Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct Tag {
    pub id: i64,
    pub name: String,
    pub created_at: Option<chrono::NaiveDateTime>,
}

#[derive(Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct Task {
    pub id: i64,
    pub task: String,
    pub created_at: Option<chrono::NaiveDateTime>,
}

#[derive(Serialize, Deserialize)]
pub struct GetTagsResponse {
    pub tags: Vec<Tag>,
    pub count: usize,
}

#[derive(Serialize, Deserialize)]
pub struct GetTasksResponse {
    pub tasks: Vec<Task>,
    pub count: usize,
}

#[derive(Deserialize)]
pub struct CreateTimedEventRequest {
    pub tag_ids: Vec<i64>,
    pub task_id: i64,
}

#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct TimedEvent {
    pub id: i64,
    pub task_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[sqlx(skip)]
    pub task: Option<Task>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[sqlx(skip)]
    pub tags: Vec<Tag>,
    pub created_at: Option<chrono::NaiveDateTime>,
    pub stopped_at: Option<chrono::NaiveDateTime>,
}

#[derive(Serialize, Deserialize)]
pub struct CreateTimedEventResponse {
    pub id: i64,
    pub message: String,
}

#[derive(Serialize, Deserialize)]
pub struct GetEventsResponse {
    pub events: Vec<TimedEvent>,
    pub count: usize,
}

#[derive(Serialize, Deserialize)]
pub struct StopEventResponse {
    pub id: i64,
    pub message: String,
    pub duration_seconds: i64,
}

pub fn create_app(pool: Pool<Sqlite>) -> Router {
    Router::new()
        .route("/tag", post(create_tag))
        .route("/tags", get(get_tags))
        .route("/task", post(create_task))
        .route("/tasks", get(get_tasks))
        .route("/events", get(get_events))
        .route("/events/start", post(create_event))
        .route("/events/stop/{id}", post(stop_event))
        .with_state(pool)
}

async fn create_tag(
    State(pool): State<Pool<Sqlite>>,
    Json(payload): Json<CreateTagRequest>,
) -> Result<Json<CreateTagResponse>, StatusCode> {
    println!("create tag");
    let result = sqlx::query!("INSERT INTO tags (name) VALUES (?)", payload.name)
        .execute(&pool)
        .await;

    match result {
        Ok(result) => {
            let response = CreateTagResponse {
                id: result.last_insert_rowid(),
                name: payload.name,
                message: "Tag created successfully".to_string(),
            };
            Ok(Json(response))
        }
        Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => {
            Err(StatusCode::CONFLICT)
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn create_task(
    State(pool): State<Pool<Sqlite>>,
    Json(payload): Json<CreateTaskRequest>,
) -> Result<Json<CreateTaskResponse>, StatusCode> {
    println!("create task");
    let result = sqlx::query!("INSERT INTO tasks (task) VALUES (?)", payload.task)
        .execute(&pool)
        .await;

    match result {
        Ok(result) => {
            let response = CreateTaskResponse {
                id: result.last_insert_rowid(),
                task: payload.task,
                message: "Task created successfully".to_string(),
            };
            Ok(Json(response))
        }
        Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => {
            Err(StatusCode::CONFLICT)
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_tags(State(pool): State<Pool<Sqlite>>) -> Result<Json<GetTagsResponse>, StatusCode> {
    println!("get tags");
    let result = sqlx::query_as!(
        Tag,
        "SELECT id, name, created_at FROM tags ORDER BY created_at DESC"
    )
    .fetch_all(&pool)
    .await;

    match result {
        Ok(tags) => {
            let count = tags.len();
            let response = GetTagsResponse { tags, count };
            Ok(Json(response))
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_tasks(State(pool): State<Pool<Sqlite>>) -> Result<Json<GetTasksResponse>, StatusCode> {
    println!("get tasks");
    let result = sqlx::query_as!(
        Task,
        "SELECT id, task, created_at FROM tasks ORDER BY created_at DESC"
    )
    .fetch_all(&pool)
    .await;

    match result {
        Ok(tasks) => {
            let count = tasks.len();
            let response = GetTasksResponse { tasks, count };
            Ok(Json(response))
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_events(
    State(pool): State<Pool<Sqlite>>,
) -> Result<Json<GetEventsResponse>, StatusCode> {
    println!("get events");

    let events = sqlx::query!(
        "SELECT id as \"id!\", task_id as \"task_id!\", created_at, stopped_at
         FROM events
         WHERE stopped_at IS NULL
         ORDER BY created_at DESC"
    )
    .fetch_all(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut events_with_details = Vec::new();

    for event in events {
        let task = sqlx::query_as!(
            Task,
            "SELECT id as \"id!\", task as \"task!\", created_at FROM tasks WHERE id = ?",
            event.task_id
        )
        .fetch_optional(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let tags = sqlx::query_as!(
            Tag,
            "SELECT t.id as \"id!\", t.name as \"name!\", t.created_at
             FROM tags t
             JOIN event_tags et ON et.tag_id = t.id
             WHERE et.event_id = ?",
            event.id
        )
        .fetch_all(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let timed_event = TimedEvent {
            id: event.id,
            task_id: event.task_id,
            task,
            tags,
            created_at: Some(event.created_at),
            stopped_at: event.stopped_at,
        };

        events_with_details.push(timed_event);
    }

    let count = events_with_details.len();
    Ok(Json(GetEventsResponse {
        events: events_with_details,
        count,
    }))
}

async fn create_event(
    State(pool): State<Pool<Sqlite>>,
    Json(payload): Json<CreateTimedEventRequest>,
) -> Result<Json<CreateTimedEventResponse>, StatusCode> {
    println!("create event");

    let mut tx = pool
        .begin()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let task_exists = sqlx::query!("SELECT id FROM tasks WHERE id = ?", payload.task_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if task_exists.is_none() {
        return Err(StatusCode::BAD_REQUEST);
    }

    for tag_id in &payload.tag_ids {
        let tag_exists = sqlx::query!("SELECT id FROM tags WHERE id = ?", tag_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        if tag_exists.is_none() {
            return Err(StatusCode::BAD_REQUEST);
        }
    }

    let event_result = sqlx::query!(
        "INSERT INTO events (task_id, created_at, stopped_at) VALUES (?, datetime('now'), NULL)",
        payload.task_id
    )
    .execute(&mut *tx)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let event_id = event_result.last_insert_rowid();

    for tag_id in payload.tag_ids {
        sqlx::query!(
            "INSERT INTO event_tags (event_id, tag_id) VALUES (?, ?)",
            event_id,
            tag_id
        )
        .execute(&mut *tx)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    tx.commit()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(CreateTimedEventResponse {
        id: event_id,
        message: "Event started successfully".to_string(),
    }))
}

async fn stop_event(
    State(pool): State<Pool<Sqlite>>,
    Path(event_id): Path<i64>,
) -> Result<Json<StopEventResponse>, StatusCode> {
    println!("stop event {}", event_id);

    let event = sqlx::query!(
        "SELECT id as \"id!\", created_at as \"created_at!\" FROM events WHERE id = ? AND stopped_at IS NULL",
        event_id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match event {
        Some(event_data) => {
            sqlx::query!(
                "UPDATE events SET stopped_at = datetime('now') WHERE id = ?",
                event_id
            )
            .execute(&pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            let duration_seconds = {
                let now = chrono::Local::now().naive_local();
                (now - event_data.created_at).num_seconds()
            };

            Ok(Json(StopEventResponse {
                id: event_id,
                message: "Event stopped successfully".to_string(),
                duration_seconds,
            }))
        }
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn create_database_pool(database_url: &str) -> Result<Pool<Sqlite>, sqlx::Error> {
    let pool = SqlitePool::connect(database_url).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}
