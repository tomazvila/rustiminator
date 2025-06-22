use axum::{
    Json, Router,
    extract::State,
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

#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct Tag {
    pub id: i64,
    pub name: String,
    pub created_at: Option<chrono::NaiveDateTime>,
}

#[derive(Serialize, Deserialize, sqlx::FromRow)]
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

pub fn create_app(pool: Pool<Sqlite>) -> Router {
    Router::new()
        .route("/tag", post(create_tag))
        .route("/tags", get(get_tags))
        .route("/task", post(create_task))
        .route("/tasks", get(get_tasks))
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

pub async fn create_database_pool(database_url: &str) -> Result<Pool<Sqlite>, sqlx::Error> {
    let pool = SqlitePool::connect(database_url).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}
