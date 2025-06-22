# Rustimenator

A REST API for managing tags, tasks, and timed events built with Rust, Axum, and SQLite.

## Setup

### SQLite setup

```bash
cargo install sqlx-cli --no-default-features --features sqlite  # one time only, for sqlx-cli tool installation
export DATABASE_URL="sqlite:./rustimenator.db"
sqlx database create --database-url sqlite:./rustimenator.db
sqlx migrate run --database-url sqlite:./rustimenator.db
cargo build
cargo sqlx prepare  # sets up macros for IDE support
```

### Database

The application uses SQLite and automatically runs migrations on startup. Ensure your `DATABASE_URL` is configured and the `./migrations` directory contains your migration files.

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

* `409 Conflict` – Tag name already exists (unique constraint violation)
* `500 Internal Server Error` – Database error

#### `GET /tags`

Retrieves all tags ordered by creation date (newest first).

**Response (200 OK):**

```json
{
  "tags": [
    { "id": 123, "name": "example-tag",    "created_at": "2025-06-22T10:30:00" },
    { "id": 122, "name": "another-tag",    "created_at": "2025-06-21T15:45:00" }
  ],
  "count": 2
}
```

**Error Responses:**

* `500 Internal Server Error` – Database error

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

* `409 Conflict` – Task already exists (unique constraint violation)
* `500 Internal Server Error` – Database error

#### `GET /tasks`

Retrieves all tasks ordered by creation date (newest first).

**Response (200 OK):**

```json
{
  "tasks": [
    { "id": 456, "task": "Complete documentation",    "created_at": "2025-06-22T11:00:00" },
    { "id": 455, "task": "Review pull requests",    "created_at": "2025-06-22T09:30:00" }
  ],
  "count": 2
}
```

**Error Responses:**

* `500 Internal Server Error` – Database error

### Events

Timed events allow you to start and stop work sessions on tasks, optionally tagging them for categorization.

#### `POST /events/start`

Begins a new timed event for a given task and associated tags. Sets `created_at` to the current timestamp.

**Request Body:**

```json
{
  "task_id": 456,
  "tag_ids": [123, 124]
}
```

**Response (201 Created):**

```json
{
  "id": 789,
  "message": "Event started successfully"
}
```

**Error Responses:**

* `400 Bad Request` – `task_id` or one of the `tag_ids` does not exist
* `500 Internal Server Error` – Database error

#### `GET /events`

Lists all active (running) timed events, including their tasks and tags.

**Response (200 OK):**

```json
{
  "events": [
    {
      "id": 789,
      "task_id": 456,
      "task": { "id": 456, "task": "Complete documentation", "created_at": "2025-06-22T11:00:00" },
      "tags": [ { "id": 123, "name": "rust", "created_at": "2025-06-20T08:15:00" } ],
      "created_at": "2025-06-22T12:00:00",
      "stopped_at": null
    }
  ],
  "count": 1
}
```

**Error Responses:**

* `500 Internal Server Error` – Database error

#### `POST /events/stop/{id}`

Stops a running event by ID, setting its `stopped_at` timestamp to now and returning the duration in seconds.

**Path Parameter:**

* `id` (integer) – The ID of the event to stop

**Response (200 OK):**

```json
{
  "id": 789,
  "message": "Event stopped successfully",
  "duration_seconds": 3600
}
```

**Error Responses:**

* `404 Not Found` – No active event found with the given ID
* `500 Internal Server Error` – Database error

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

### Events

#### Starting an event:

```bash
curl -X POST http://localhost:3000/events/start \
  -H "Content-Type: application/json" \
  -d '{"task_id": 456, "tag_ids": [123, 124]}'
```

#### Listing active events:

```bash
curl http://localhost:3000/events
```

#### Stopping an event:

```bash
curl -X POST http://localhost:3000/events/stop/789
```

## Error Handling

The API uses standard HTTP status codes and handles the following cases:

* **Unique constraint violations**: Returns `409 Conflict` when attempting to create a tag or task with a duplicate value
* **Missing or invalid references**: Returns `400 Bad Request` when starting an event with a non-existent task or tag
* **Not found**: Returns `404 Not Found` when stopping an event that does not exist or is already stopped
* **Database errors**: Returns `500 Internal Server Error` for other database-related issues
* **Malformed requests**: Axum automatically handles JSON parsing errors with `400 Bad Request`
