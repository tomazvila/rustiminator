# Rustimenator

A REST API for managing tags built with Rust, Axum, and SQLite.

## Setup

### SQLite setup
```bash
cargo install sqlx-cli --no-default-features --features sqlite # one time only, for sqlx-cli tool installation
export DATABASE_URL="sqlite:./rustimenator.db"
sqlx database create --database-url sqlite:./rustimenator.db
sqlx migrate run --database-url sqlite:./rustimenator.db
cargo build
cargo sqlx prepare # sets up macros for ide
```

### Database
The application uses SQLite and automatically runs migrations on startup. Ensure your database URL is configured and the `./migrations` directory contains your migration files.

### Dependencies
- **axum**: Web framework
- **sqlx**: Async SQL toolkit with SQLite support
- **serde**: Serialization/deserialization
- **chrono**: Date and time handling

## Endpoints

### `POST /tag`
Creates a new tag.

**Request Body:**
```json
{
  "name": "string"
}
```

**Response (201 Created):**
```json
{
  "id": 123,
  "name": "example-tag",
  "message": "Tag created successfully"
}
```

**Error Responses:**
- `409 Conflict` - Tag name already exists (unique constraint violation)
- `500 Internal Server Error` - Database error

### `GET /tags`
Retrieves all tags ordered by creation date (newest first).

**Response (200 OK):**
```json
{
  "tags": [
    {
      "id": 123,
      "name": "example-tag",
      "created_at": "2025-06-22T10:30:00"
    },
    {
      "id": 122,
      "name": "another-tag",
      "created_at": "2025-06-21T15:45:00"
    }
  ],
  "count": 2
}
```

**Error Responses:**
- `500 Internal Server Error` - Database error

## Data Models

### Tag
```rust
pub struct Tag {
    pub id: i64,
    pub name: String,
    pub created_at: Option<chrono::NaiveDateTime>,
}
```

### Create Tag Request
```rust
pub struct CreateTagRequest {
    pub name: String,
}
```

### Create Tag Response
```rust
pub struct CreateTagResponse {
    pub id: i64,
    pub name: String,
    pub message: String,
}
```

### Get Tags Response
```rust
pub struct GetTagsResponse {
    pub tags: Vec<Tag>,
    pub count: usize,
}
```

## Usage Example

### Creating a tag:
```bash
curl -X POST http://localhost:3000/tag \
  -H "Content-Type: application/json" \
  -d '{"name": "rust"}'
```

### Getting all tags:
```bash
curl http://localhost:3000/tags
```

## Database Schema

The application expects a `tags` table with the following structure:
```sql
CREATE TABLE tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

## Error Handling

The API uses standard HTTP status codes and handles the following error cases:
- **Unique constraint violations**: Returns 409 Conflict when attempting to create a tag with a duplicate name
- **Database errors**: Returns 500 Internal Server Error for other database-related issues
- **Malformed requests**: Axum automatically handles JSON parsing errors with 400 Bad Request
