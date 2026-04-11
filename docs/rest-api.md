---
outline: deep
---

# REST API

MagtivityPub exposes a RESTful Client-to-Server (C2S) API over HTTP(S). This API is the primary interface for building custom manga readers, mobile apps, and web interfaces.

::: tip
The canonical machine-readable specification is the [OpenAPI 3.0 schema](/openapi.yaml). The documentation below is a human-friendly guide to the same surface.
:::

## Base URL

All API endpoints are served under the `/api/v1` prefix:

```
https://your-instance.example/api/v1
```

## Authentication

MagtivityPub uses **OAuth 2.0** for client authentication. All mutating endpoints require a valid bearer token.

### Obtaining a Token

Register your application to obtain client credentials, then exchange them for a bearer token:

```http
POST /api/v1/oauth/token
Content-Type: application/json

{
  "grant_type": "authorization_code",
  "client_id": "your-client-id",
  "client_secret": "your-client-secret",
  "redirect_uri": "https://your-app.example/callback",
  "code": "authorization-code"
}
```

**Response:**

```json
{
  "access_token": "eyJhbGciOiJSUzI1NiIs...",
  "token_type": "Bearer",
  "expires_in": 3600,
  "refresh_token": "dGhpcyBpcyBhIHJlZnJlc2g..."
}
```

### Using a Token

Include the token in the `Authorization` header of every request:

```http
GET /api/v1/comics
Authorization: Bearer eyJhbGciOiJSUzI1NiIs...
```

## Resources

### Comics

A **Comic** represents a manga, manhwa, manhua, or webcomic series. It maps to an ActivityPub `Collection` with the `mag:Comic` type extension.

#### List Comics

```http
GET /api/v1/comics
```

**Query Parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `page` | integer | `1` | Page number |
| `per_page` | integer | `20` | Results per page (max 100) |
| `sort` | string | `updated_at` | Sort field: `title`, `created_at`, `updated_at` |
| `order` | string | `desc` | Sort order: `asc`, `desc` |
| `status` | string | — | Filter by status: `ongoing`, `completed`, `hiatus` |
| `q` | string | — | Full-text search query |

**Response:**

```json
{
  "data": [
    {
      "id": "01HYX3K4M7N8P9Q2R5S6T7V8W9",
      "title": "Parallel Universe Kitchen",
      "slug": "parallel-universe-kitchen",
      "description": "A chef discovers each dish they cook opens a portal to an alternate reality...",
      "status": "ongoing",
      "cover_url": "https://cdn.example/covers/puk-cover.webp",
      "genres": ["fantasy", "slice-of-life", "comedy"],
      "authors": [
        {
          "id": "01HYX3A1B2C3D4E5F6G7H8J9K0",
          "name": "Tanaka Yuki",
          "role": "story"
        }
      ],
      "chapter_count": 42,
      "created_at": "2025-09-15T08:30:00Z",
      "updated_at": "2026-04-10T14:22:00Z"
    }
  ],
  "pagination": {
    "page": 1,
    "per_page": 20,
    "total": 237,
    "total_pages": 12
  }
}
```

#### Get Comic

```http
GET /api/v1/comics/:id
```

Returns a single comic resource with full metadata, including the author attribution array and genre tags.

#### Create Comic

```http
POST /api/v1/comics
Authorization: Bearer <token>
Content-Type: application/json

{
  "title": "Parallel Universe Kitchen",
  "description": "A chef discovers each dish they cook opens a portal...",
  "status": "ongoing",
  "genres": ["fantasy", "slice-of-life", "comedy"],
  "authors": [
    { "name": "Tanaka Yuki", "role": "story" },
    { "name": "Mori Akane", "role": "art" }
  ]
}
```

#### Update Comic

```http
PATCH /api/v1/comics/:id
Authorization: Bearer <token>
Content-Type: application/json

{
  "status": "completed"
}
```

#### Delete Comic

```http
DELETE /api/v1/comics/:id
Authorization: Bearer <token>
```

Returns `204 No Content` on success.

---

### Chapters

A **Chapter** belongs to a Comic and represents a single installment. It maps to an ActivityPub `Article` with the `mag:Chapter` type extension.

#### List Chapters

```http
GET /api/v1/comics/:comic_id/chapters
```

**Query Parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `page` | integer | `1` | Page number |
| `per_page` | integer | `50` | Results per page (max 200) |
| `sort` | string | `number` | Sort field: `number`, `created_at`, `updated_at` |
| `order` | string | `asc` | Sort order: `asc`, `desc` |
| `language` | string | — | Filter by ISO 639-1 language code |

**Response:**

```json
{
  "data": [
    {
      "id": "01HYX4A1B2C3D4E5F6G7H8J9K0",
      "comic_id": "01HYX3K4M7N8P9Q2R5S6T7V8W9",
      "number": 1,
      "volume": 1,
      "title": "The First Ingredient",
      "language": "en",
      "page_count": 24,
      "scanlation_group": {
        "id": "01HYX5Z9Y8X7W6V5U4T3S2R1Q0",
        "name": "Galaxy Scans"
      },
      "published_at": "2025-09-15T08:30:00Z",
      "created_at": "2025-09-15T08:30:00Z"
    }
  ],
  "pagination": {
    "page": 1,
    "per_page": 50,
    "total": 42,
    "total_pages": 1
  }
}
```

#### Get Chapter

```http
GET /api/v1/comics/:comic_id/chapters/:id
```

#### Create Chapter

```http
POST /api/v1/comics/:comic_id/chapters
Authorization: Bearer <token>
Content-Type: application/json

{
  "number": 43,
  "volume": 5,
  "title": "The Secret Spice",
  "language": "en"
}
```

#### Joint Releases

A chapter can be attributed to multiple scanlation groups via the **joint-release attribution array**:

```http
POST /api/v1/comics/:comic_id/chapters
Authorization: Bearer <token>
Content-Type: application/json

{
  "number": 43,
  "title": "The Secret Spice",
  "language": "en",
  "attribution": [
    { "group_id": "01HYX5Z9Y8X7W6V5U4T3S2R1Q0", "role": "translation" },
    { "group_id": "01HYX6A1B2C3D4E5F6G7H8J9K0", "role": "cleaning" },
    { "group_id": "01HYX7M1N2P3Q4R5S6T7U8V9W0", "role": "typesetting" }
  ]
}
```

---

### Pages

**Pages** are the individual images within a chapter. They are managed through a dedicated upload/retrieval interface.

#### List Pages

```http
GET /api/v1/comics/:comic_id/chapters/:chapter_id/pages
```

**Response:**

```json
{
  "data": [
    {
      "number": 1,
      "image_url": "https://cdn.example/pages/01HYX4.../001.webp",
      "width": 800,
      "height": 1200,
      "size_bytes": 184320
    }
  ]
}
```

#### Upload Pages

Pages are uploaded as multipart form data. The server generates thumbnails asynchronously.

```http
POST /api/v1/comics/:comic_id/chapters/:chapter_id/pages
Authorization: Bearer <token>
Content-Type: multipart/form-data

--boundary
Content-Disposition: form-data; name="pages[]"; filename="001.png"
Content-Type: image/png

<binary data>
--boundary
Content-Disposition: form-data; name="pages[]"; filename="002.png"
Content-Type: image/png

<binary data>
--boundary--
```

**Response:** `202 Accepted` with a job status URL for tracking thumbnail generation.

```json
{
  "status": "processing",
  "pages_received": 2,
  "job_url": "/api/v1/jobs/01HYX8A1B2C3D4E5F6G7H8J9K0"
}
```

---

### Users & Profiles

#### Get Current User

```http
GET /api/v1/me
Authorization: Bearer <token>
```

#### Get User Profile

```http
GET /api/v1/users/:id
```

#### Follow a User

```http
POST /api/v1/users/:id/follow
Authorization: Bearer <token>
```

---

### Search

#### Full-Text Search

```http
GET /api/v1/search?q=isekai+chef&type=comic
```

| Parameter | Type | Description |
|-----------|------|-------------|
| `q` | string | Search query |
| `type` | string | Restrict results: `comic`, `chapter`, `user` |
| `page` | integer | Page number |
| `per_page` | integer | Results per page |

## Pagination

All list endpoints return paginated responses using offset-based pagination. The response includes a `pagination` object:

```json
{
  "pagination": {
    "page": 2,
    "per_page": 20,
    "total": 237,
    "total_pages": 12
  }
}
```

Navigate with `?page=N&per_page=N` query parameters. `Link` headers are also provided for RESTful clients:

```http
Link: </api/v1/comics?page=3&per_page=20>; rel="next",
      </api/v1/comics?page=1&per_page=20>; rel="prev",
      </api/v1/comics?page=12&per_page=20>; rel="last"
```

## Error Handling

All errors follow a consistent JSON structure:

```json
{
  "error": {
    "code": "not_found",
    "message": "The requested comic does not exist.",
    "details": []
  }
}
```

### HTTP Status Codes

| Code | Meaning |
|------|---------|
| `200` | Success |
| `201` | Resource created |
| `202` | Accepted (async processing) |
| `204` | No content (successful delete) |
| `400` | Bad request — validation errors |
| `401` | Unauthorized — missing or invalid token |
| `403` | Forbidden — insufficient permissions |
| `404` | Not found |
| `409` | Conflict — duplicate resource |
| `422` | Unprocessable entity — semantic validation failure |
| `429` | Too many requests — rate limited |
| `500` | Internal server error |

### Validation Errors

`400` and `422` responses include field-level detail:

```json
{
  "error": {
    "code": "validation_error",
    "message": "Request validation failed.",
    "details": [
      { "field": "title", "message": "is required" },
      { "field": "status", "message": "must be one of: ongoing, completed, hiatus" }
    ]
  }
}
```

## Rate Limiting

Authenticated requests are rate-limited per OAuth token. Current limits:

| Tier | Limit |
|------|-------|
| Read endpoints | 300 requests / minute |
| Write endpoints | 30 requests / minute |
| Upload endpoints | 10 requests / minute |

Rate limit status is communicated via response headers:

```http
X-RateLimit-Limit: 300
X-RateLimit-Remaining: 287
X-RateLimit-Reset: 1712890200
```

## OpenAPI Schema

The complete API specification is available as an [OpenAPI 3.0](https://spec.openapis.org/oas/v3.0.3) YAML document at:

```
https://your-instance.example/openapi.yaml
```

Use it with tools like [Swagger UI](https://swagger.io/tools/swagger-ui/), code generators, or any OpenAPI-compatible client.
