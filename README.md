# Rustimenator

A REST API for managing tags and tasks built with Rust, Axum, and SQLite.

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

## Endpoints

### Tags

#### `POST /tag`
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

#### `GET /tags`
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

### Tasks

#### `POST /task`
Creates a new task.

**Request Body:**
```json
{
  "task": "string"
}
```

**Response (201 Created):**
```json
{
  "id": 456,
  "task": "Complete documentation",
  "message": "Task created successfully"
}
```

**Error Responses:**
- `409 Conflict` - Task already exists (unique constraint violation)
- `500 Internal Server Error` - Database error

#### `GET /tasks`
Retrieves all tasks ordered by creation date (newest first).

**Response (200 OK):**
```json
{
  "tasks": [
    {
      "id": 456,
      "task": "Complete documentation",
      "created_at": "2025-06-22T11:00:00"
    },
    {
      "id": 455,
      "task": "Review pull requests",
      "created_at": "2025-06-22T09:30:00"
    }
  ],
  "count": 2
}
```

**Error Responses:**
- `500 Internal Server Error` - Database error

## Usage Examples

### Tags

#### Creating a tag:
```bash
curl -X POST http://localhost:3000/tag \
  -H "Content-Type: application/json" \
  -d '{"name": "rust"}'
```

#### Getting all tags:
```bash
curl http://localhost:3000/tags
```

### Tasks

#### Creating a task:
```bash
curl -X POST http://localhost:3000/task \
  -H "Content-Type: application/json" \
  -d '{"task": "Complete API documentation"}'
```

#### Getting all tasks:
```bash
curl http://localhost:3000/tasks
```

## Error Handling

The API uses standard HTTP status codes and handles the following error cases:
- **Unique constraint violations**: Returns 409 Conflict when attempting to create a tag or task with a duplicate value
- **Database errors**: Returns 500 Internal Server Error for other database-related issues
- **Malformed requests**: Axum automatically handles JSON parsing errors with 400 Bad Request
