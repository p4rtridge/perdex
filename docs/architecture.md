---
outline: deep
---

# Architecture

## Introduction

The digital distribution of serialized graphic literature—encompassing manga, webtoons, and traditional comic books—has historically been dominated by centralized platforms. These platforms impose stringent vendor lock-in, arbitrary content moderation policies, and unilateral algorithmic shifts that frequently disenfranchise both independent creators and collaborative scanlation communities. MagtivityPub is proposed as an open-source, decentralized, and federated publishing platform engineered specifically to mitigate these vulnerabilities. By leveraging the ActivityPub protocol, MagtivityPub establishes a resilient, peer-to-peer web of interoperable comic hosting instances, ensuring that data ownership remains with the creators and communities that generate it.

### Vocabulary

- `Fediverse`: Designates federated communities in general.
- `Comicverse`: A subset of the Fediverse specifically federating graphic literature and comic chapters.
- `Instance`: A server which runs MagtivityPub in the fediverse.
- `Origin/local instance`: The instance on which the comic chapter was uploaded and that serves the images behind an HTTP server.
- `Cache instance`: An instance that decided to mirror a remote comic chapter to improve bandwidth use and mitigate origin server overload.
- `Following`: The action of a MagtivityPub instance which will follow another instance (subscribe to its groups and comics).

### MagtivityPub instance

- An instance acts like a website: it has an administrator, and people can create an account on the instance. Users manage multiple publishing groups (scanlation teams) in which they decide to upload comics.
- An instance acts like a normal webserver: users can upload compressed sequential image archives and the instance will serve the extracted image files behind an HTTP server.
- An instance has an administrator that can follow other instances using the ActivityPub protocol so that remote comics can be displayed natively on the local instance.

## Global Overview

MagtivityPub is designed around a decoupled architecture separating the client interface from the backend server operations. This system must handle the intensive input/output operations associated with processing high-resolution sequential image archives while simultaneously managing the cryptographic and networking overhead of federated social messaging.

- `The client interface`: A standalone client application (web or mobile) that consumes the Client-to-Server (C2S) REST API to render manga readers, manage groups, and handle user interactions.
- `The perimeter gateway`: A reverse proxy utilized to handle SSL termination, rate limiting, and the direct serving of static image assets. Furthermore, instances can configure this gateway to serve content directly from S3-compatible object storage.
- `The application server`: A backend service that encompasses the C2S API endpoints, the ActivityPub Server-to-Server (S2S) API, and the asynchronous background task schedulers.
- `Persistent storage`: An ACID-compliant database used for the long-term storage of relational metadata (users, groups, chapters, follower collections).
- `The task queue`: A rapid key-value store or message broker used as a task queueing mechanism. The server offloads network delivery and archive extraction to this queue, ensuring that the primary HTTP threads remain unblocked.

## Server

### Architectural Requirements

The backend server must be capable of asynchronous I/O and concurrent processing. It interacts with the persistent database via an ORM/ODM layer and relies on an external job queueing service for managing high-latency federation tasks.

### Logical Component Structure

The backend architecture should mirror proven enterprise models separating concerns:

- `API Gateway / Controllers`: Defines the C2S REST API routes and the S2S `/inbox` endpoint logic.
- `ActivityPub Dispatcher`: Utility functions, cryptographic signers, and custom ActivityPub JSON-LD validators.
- `Data Access Layer`: Database models mapped to ActivityStreams entities.
- `Middlewares`: Request validation, OAuth2 authorization, and rate-limiting blocks.

### Concepts

#### Archive Extraction & Transcoding

When a user uploads a packaged comic archive (e.g., standard compressed formats) via the C2S API, the system offloads extraction to background worker processes. Images are parsed, resized, and transcoded into optimized web formats. The worker computationally synthesizes the ActivityPub `Article` and `OrderedCollection` JSON-LD structures to prepare for network delivery.

#### Communications between instances

Federation between MagtivityPub servers relies on exchanging strict `application/ld+json` payloads.

- ActivityPub messages are signed with JSON Linked Data Signatures using the private key of the account or group that authored the action.
- S2S requests are strictly authenticated via HTTP Message Signatures using the instance's private key to prevent spoofing.
- All outgoing HTTP POST requests to remote inboxes are retried several times with exponential backoff if they fail.
- A MagtivityPub instance maintains a global `Application` ActivityPub Actor. Other instances can follow this actor to receive instance-wide `Announce` activities for newly uploaded comics.

#### Redundancy between instances

To optimize bandwidth across the Fediverse and mitigate origin server failure during highly anticipated chapter releases, MagtivityPub implements a cache instance mechanism. The instance administrator can choose between multiple redundancy strategies (e.g., caching trending comics or recently uploaded chapters), configure minimum duplication lifetimes, and set their maximum cache storage size.

## Client