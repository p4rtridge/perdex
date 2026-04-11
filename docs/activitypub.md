---
outline: deep
---

# ActivityPub

MagtivityPub extends the [ActivityPub](https://www.w3.org/TR/activitypub/) protocol to support decentralized comic publishing and discovery. This document covers Server-to-Server (S2S) federation specifics, including our custom vocabulary, data modeling, and interaction flows.

::: info
This page documents the **S2S federation layer**. For client-facing APIs, see the [REST API documentation](./rest-api). For how these layers fit together, see the [Architecture overview](./architecture).
:::

## Federation Basics

Each MagtivityPub instance exposes standard ActivityPub endpoints:

| Endpoint | Purpose |
|----------|---------|
| `/.well-known/webfinger` | Actor discovery via `acct:` URI |
| `/users/:id` | Actor profile (JSON-LD) |
| `/users/:id/inbox` | Receive activities from remote instances |
| `/users/:id/outbox` | Serve published activities |
| `/users/:id/followers` | Followers collection |
| `/users/:id/following` | Following collection |

All payloads use `application/ld+json; profile="https://www.w3.org/ns/activitystreams"` as the content type, with the `mag:` context extension included.

## The `mag:` Namespace

MagtivityPub introduces a custom JSON-LD namespace to represent comic-specific concepts that have no equivalent in the core ActivityStreams vocabulary.

### Context Definition

Every MagtivityPub activity includes the `mag:` context alongside the standard ActivityStreams context:

```json
{
  "@context": [
    "https://www.w3.org/ns/activitystreams",
    {
      "mag": "https://magtivitypub.org/ns#",
      "mag:Comic": "mag:Comic",
      "mag:Chapter": "mag:Chapter",
      "mag:Attribution": "mag:Attribution",
      "mag:coverImage": { "@id": "mag:coverImage", "@type": "@id" },
      "mag:chapterNumber": "mag:chapterNumber",
      "mag:volumeNumber": "mag:volumeNumber",
      "mag:pageCount": "mag:pageCount",
      "mag:language": "mag:language",
      "mag:status": "mag:status",
      "mag:genres": { "@id": "mag:genres", "@container": "@set" },
      "mag:attribution": { "@id": "mag:attribution", "@container": "@list" }
    }
  ]
}
```

### Custom Types

| Type | Extends | Description |
|------|---------|-------------|
| `mag:Comic` | `Collection` | A comic series (manga, manhwa, manhua, webcomic) |
| `mag:Chapter` | `Article` | A single chapter within a comic |
| `mag:Attribution` | `Relationship` | A scanlation group's role in a joint release |

### Custom Properties

| Property | Domain | Range | Description |
|----------|--------|-------|-------------|
| `mag:coverImage` | `mag:Comic` | URI | Cover art URL |
| `mag:status` | `mag:Comic` | String | `ongoing`, `completed`, `hiatus` |
| `mag:genres` | `mag:Comic` | Set of strings | Genre tags |
| `mag:chapterNumber` | `mag:Chapter` | Integer | Chapter ordinal |
| `mag:volumeNumber` | `mag:Chapter` | Integer | Volume ordinal |
| `mag:pageCount` | `mag:Chapter` | Integer | Number of pages |
| `mag:language` | `mag:Chapter` | String | ISO 639-1 language code |
| `mag:attribution` | `mag:Chapter` | List of `mag:Attribution` | Joint-release attribution |

## Data Modeling

### Comic as Collection

A **Comic** is modeled as an ActivityStreams `Collection` extended with the `mag:Comic` type. The collection's `items` are its chapters.

```json
{
  "@context": ["https://www.w3.org/ns/activitystreams", "...mag context..."],
  "id": "https://instance-a.example/comics/parallel-universe-kitchen",
  "type": ["Collection", "mag:Comic"],
  "name": "Parallel Universe Kitchen",
  "summary": "A chef discovers each dish they cook opens a portal to an alternate reality...",
  "mag:coverImage": "https://cdn.instance-a.example/covers/puk-cover.webp",
  "mag:status": "ongoing",
  "mag:genres": ["fantasy", "slice-of-life", "comedy"],
  "attributedTo": [
    {
      "type": "Person",
      "name": "Tanaka Yuki",
      "id": "https://instance-a.example/users/tanaka-yuki"
    }
  ],
  "totalItems": 42,
  "items": "https://instance-a.example/comics/parallel-universe-kitchen/chapters"
}
```

### Chapter as Article

A **Chapter** is modeled as an ActivityStreams `Article` extended with the `mag:Chapter` type. The article's `content` is not the page images themselves — pages are linked via `attachment`.

```json
{
  "@context": ["https://www.w3.org/ns/activitystreams", "...mag context..."],
  "id": "https://instance-a.example/chapters/01HYX4A1B2C3",
  "type": ["Article", "mag:Chapter"],
  "name": "Chapter 43: The Secret Spice",
  "mag:chapterNumber": 43,
  "mag:volumeNumber": 5,
  "mag:pageCount": 28,
  "mag:language": "en",
  "inReplyTo": "https://instance-a.example/comics/parallel-universe-kitchen",
  "published": "2026-04-10T14:22:00Z",
  "attributedTo": "https://instance-a.example/users/tanaka-yuki",
  "mag:attribution": [
    {
      "type": "mag:Attribution",
      "object": "https://instance-b.example/groups/galaxy-scans",
      "relationship": "translation"
    },
    {
      "type": "mag:Attribution",
      "object": "https://instance-c.example/groups/pixel-perfect",
      "relationship": "cleaning"
    },
    {
      "type": "mag:Attribution",
      "object": "https://instance-a.example/groups/typo-knights",
      "relationship": "typesetting"
    }
  ],
  "attachment": [
    {
      "type": "Image",
      "mediaType": "image/webp",
      "url": "https://cdn.instance-a.example/pages/01HYX4.../001.webp",
      "width": 800,
      "height": 1200
    }
  ]
}
```

### Joint-Release Attribution Arrays

The `mag:attribution` property is a **JSON-LD ordered list** of `mag:Attribution` objects. This preserves the order of credits and supports cross-instance attribution — each group is identified by their federated actor URI.

#### Attribution Roles

| Role | Description |
|------|-------------|
| `translation` | Translated the script to the target language |
| `cleaning` | Cleaned and redrawn page scans |
| `typesetting` | Applied translated text to cleaned pages |
| `redrawing` | Redrawn SFX or complex art elements |
| `proofreading` | Reviewed the final typeset for errors |
| `raw_provider` | Provided the raw (untranslated) scans |
| `quality_check` | Final quality assurance pass |

Roles are not exclusive — a single group can hold multiple roles in one release:

```json
{
  "mag:attribution": [
    {
      "type": "mag:Attribution",
      "object": "https://instance-b.example/groups/galaxy-scans",
      "relationship": ["translation", "typesetting", "proofreading"]
    }
  ]
}
```

## Federation Flows

### Publishing a New Chapter

When a chapter is created on the origin instance, the server dispatches a `Create` activity to all followers of the comic's author actor:

```json
{
  "@context": ["https://www.w3.org/ns/activitystreams", "...mag context..."],
  "id": "https://instance-a.example/activities/01HYX9",
  "type": "Create",
  "actor": "https://instance-a.example/users/tanaka-yuki",
  "published": "2026-04-10T14:22:00Z",
  "to": ["https://www.w3.org/ns/activitystreams#Public"],
  "cc": ["https://instance-a.example/users/tanaka-yuki/followers"],
  "object": {
    "type": ["Article", "mag:Chapter"],
    "id": "https://instance-a.example/chapters/01HYX4A1B2C3",
    "name": "Chapter 43: The Secret Spice",
    "mag:chapterNumber": 43,
    "inReplyTo": "https://instance-a.example/comics/parallel-universe-kitchen"
  }
}
```

### Following a Comic Author

A user on Instance B follows a comic author on Instance A:

```json
{
  "@context": "https://www.w3.org/ns/activitystreams",
  "id": "https://instance-b.example/activities/follow-01",
  "type": "Follow",
  "actor": "https://instance-b.example/users/reader-42",
  "object": "https://instance-a.example/users/tanaka-yuki"
}
```

Instance A responds with an `Accept` activity to confirm the follow:

```json
{
  "@context": "https://www.w3.org/ns/activitystreams",
  "id": "https://instance-a.example/activities/accept-01",
  "type": "Accept",
  "actor": "https://instance-a.example/users/tanaka-yuki",
  "object": "https://instance-b.example/activities/follow-01"
}
```

### Cross-Instance Joint Releases

When a chapter uses joint-release attribution referencing groups on remote instances, the origin server sends `Announce` activities to each attributed group's inbox, notifying them of the credit:

```json
{
  "@context": ["https://www.w3.org/ns/activitystreams", "...mag context..."],
  "id": "https://instance-a.example/activities/announce-01",
  "type": "Announce",
  "actor": "https://instance-a.example/users/tanaka-yuki",
  "object": "https://instance-a.example/chapters/01HYX4A1B2C3",
  "to": [
    "https://instance-b.example/groups/galaxy-scans",
    "https://instance-c.example/groups/pixel-perfect"
  ]
}
```

Remote instances can then display the chapter on the credited group's profile, linking back to the origin instance for the actual page content.

## HTTP Signatures

All S2S requests are signed using [HTTP Signatures (draft-cavage-http-signatures-12)](https://datatracker.ietf.org/doc/html/draft-cavage-http-signatures-12). MagtivityPub uses RSA-SHA256 key pairs generated per actor.

### Signed Headers

The following headers are included in every signature:

```
(request-target) host date digest content-type
```

### Example Signed Request

```http
POST /users/reader-42/inbox HTTP/1.1
Host: instance-b.example
Date: Thu, 10 Apr 2026 14:22:00 GMT
Digest: SHA-256=X48E9qOokqqrvdts8nOJRJN3OWDUoyWxBf7kbu9DBPE=
Content-Type: application/ld+json; profile="https://www.w3.org/ns/activitystreams"
Signature: keyId="https://instance-a.example/users/tanaka-yuki#main-key",
           algorithm="rsa-sha256",
           headers="(request-target) host date digest content-type",
           signature="..."
```

### Key Discovery

An actor's public key is embedded in their profile document:

```json
{
  "id": "https://instance-a.example/users/tanaka-yuki",
  "type": "Person",
  "publicKey": {
    "id": "https://instance-a.example/users/tanaka-yuki#main-key",
    "owner": "https://instance-a.example/users/tanaka-yuki",
    "publicKeyPem": "-----BEGIN PUBLIC KEY-----\n...\n-----END PUBLIC KEY-----"
  }
}
```

## Compatibility

MagtivityPub's S2S implementation is compatible with any ActivityPub-compliant server. Non-MagtivityPub instances will see:

- **Comics** as generic `Collection` objects with a human-readable `name` and `summary`.
- **Chapters** as generic `Article` objects with attached `Image` resources.
- **Attribution** as unrecognized `mag:` properties, which are safely ignored per JSON-LD processing rules.

MagtivityPub-aware instances gain access to the full structured vocabulary for richer display and interoperability.
