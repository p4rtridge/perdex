---
outline: deep
---

# ActivityPub Protocol Specification

## Introduction
The MagtivityPub protocol is a decentralized publishing and social networking specification based upon the ActivityStreams 2.0 data format. It provides a federated Server-to-Server (S2S) API for delivering, managing, and synchronizing serialized graphic literature (manga, webtoons, comics) across a decentralized network of independent servers.

By eliminating dependencies on specific third-party software paradigms, MagtivityPub establishes itself as a standalone set of behaviors and vocabulary extensions natively integrated into the broader Fediverse.


## Protocol Fundamentals: JSON-LD and ActivityStreams

ActivityPub relies intrinsically on JSON for Linking Data (JSON-LD), a methodology for encoding linked data using JSON. Within the Fediverse, JSON-LD allows diverse platforms to parse and understand complex data structures by defining the vocabulary through a strict `@context` array. Because the ActivityStreams 2.0 core vocabulary does not possess native classifications for comic publishing, MagtivityPub synthesizes the standard vocabulary with established ontologies.

Because the ActivityStreams 2.0 core vocabulary does not possess native classifications for comic publishing, MagtivityPub synthesizes the standard vocabulary with established ontologies—most notably the Bibliographic (`bib`) extension of Schema.org, which provides native properties for comics.

The base `@context` for all MagtivityPub objects must include the standard W3C ActivityStreams namespace, the W3C Security namespace (for cryptographic signatures), the Schema.org namespace, and the custom `mag:` namespace for domain-specific tracking.


| Namespace / Prefix | URI | Purpose in MagtivityPub Architecture |
|-----------|------|---------|
| Default | `https://www.w3.org/ns/activitystreams` | Core ActivityStreams 2.0 vocabulary (Create, Note, Actor). |
| sec | `https://w3id.org/security#` | Cryptographic signature verification (`publicKey`, `signature`). |
| schema | `https://schema.org/` | Advanced metadata mapping (`series`, `author`, `artist`). |
| mag | `https://magtivitypub.org/ns#` | Custom properties (`readingProgression`, `views`, `pages`). |

Through the strategic application of these namespaces, MagtivityPub ensures that its specialized payloads degrade gracefully. If a microblogging instance receives a MagtivityPub `Chapter` object, it will utilize the core ActivityStreams `Article` properties to render a fallback preview, ignoring the specialized `mag:` properties that it cannot process.

## Entity Specifications and Data Modeling

The structural integrity of MagtivityPub relies on a rigid hierarchical data model. The relationships between users, publishing groups, comic series, chapters, and individual pages must be deterministically mapped onto ActivityStreams entities.

### User (Actor: Person)

The foundational actor within the MagtivityPub ecosystem is the `Person`. A Person represents a registered human user interacting with an instance. Every Person is provisioned with a cryptographic key pair upon account creation, enabling them to cryptographically sign their activities and authorize S2S fetch requests.

The `Person` actor possesses dedicated endpoints crucial for federation. The inbox endpoint receives incoming activities (such as notifications of new chapters from followed groups or replies to comments), while the outbox exposes the actor's publicly broadcasted activities.

### Publishing Group (Actor: Group)

To support scanlation teams, independent publishing houses, and collaborative creator circles, MagtivityPub utilizes the `Group` actor type.

A `Group` operates as a dedicated publication feed. When remote users wish to subscribe to a comic series, they follow the `Group` responsible for its publication, adding their Actor URI to the Group's `followers` collection. To prevent orphaned entities and ensure accountability, a `Group` must be explicitly bound to a `Person` actor via the `attributedTo` property.

MagtivityPub heavily relies on FEP-1b12 (Group Federation) to dictate how `Group` actors interact with others. When a member of the Group publishes a new chapter, the `Group` actor wraps the submission in an `Announce` activity and distributes it to the Group's followers.

### Comic (Object: OrderedCollection / ComicSeries)

ActivityStreams lacks a native entity representing a serialized publication. To solve this, MagtivityPub models the root Comic as an `OrderedCollection`.

To inject profound semantic meaning, MagtivityPub maps the entity to the `schema:ComicSeries` type, defined in the Schema.org bibliographic extension as "A sequential publication of comic stories under a unifying title". This object contains global metadata for the series, including the title (`name`), synopsis (`summary`), and original publication dates (`published`).

Crucially, a comic series may contain hundreds of chapters. Serving a monolithic JSON-LD document containing the entire array of chapters exposes instances to severe response denial-of-service vulnerabilities. Therefore, the `Comic` object implements pagination using the `OrderedCollectionPage` construct. The root Comic object provides the `totalItems` count and a `first` link pointing to the initial page of results.

### Chapter (Object: Article / ComicIssue)

The `Chapter` represents a discrete, sequential update to a Comic. MagtivityPub models the `Chapter` fundamentally as an `Article` to ensure legacy Fediverse platforms render it as a multi-part, structured work.

Simultaneously, the `Chapter` is typed as `schema:ComicIssue`. Schema.org defines a `ComicIssue` as a serial publication that belongs to a larger series, identifiable by an issue number and variant descriptions.

The `Chapter` maintains its structural position through the `partOf` property, containing a direct URI reference back to the parent `Comic` collection. The sequential image payload of the `Chapter` is nested within a custom `mag:pages` property.

### Page (Object: Image / Document)

The `Page` object represents a single visual frame or scanned page within a chapter. It is fundamentally mapped to the ActivityStreams `Image` type.

The `url` property is strictly an array of `Link` objects. This array provides the client with multiple content negotiation options, pointing to different file formats (e.g., highly compressed next-gen formats vs. legacy formats) and varying dimensions. Furthermore, the `Page` object integrates the `blurhash` property to provide clients with a low-fidelity, base64-encoded placeholder image that renders instantly while the high-resolution asset loads.

### Granular Bibliographic Metadata (Artists, Inkers, Tags)

High-fidelity metadata is essential for the discoverability and archiving of graphic literature. Because scanlation uploaders are rarely the original creators, MagtivityPub strictly delineates network uploaders from the actual intellectual property creators utilizing the Schema.org `bib` extension properties:

- `schema:author / schema:creator`: Denotes the writer of the work.
- `schema:artist / schema:penciler`: Denotes the individual who draws the primary narrative artwork.
- `schema:publisherImprint`: The publishing division which originally published the comic.

Categorization is handled via the standard `Hashtag` type embedded within the object's `tag` array.


## Extensive JSON-LD Specification and Analysis

The following schemas demonstrate the precise JSON-LD structures transmitted during S2S federation.

### The Comic Object Schema (schema:ComicSeries)

This payload illustrates a `Comic` entity utilizing the `ComicSeries` type. Notice the pagination pointers and the granular application of bibliographic metadata.

```json
{
  "@context": [
    "https://www.w3.org/ns/activitystreams",
    "https://w3id.org/security/v1",
    {
      "schema": "http://schema.org/",
      "mag": "https://magtivitypub.org/ns#",
      "Hashtag": "as:Hashtag"
    }
  ],
  "id": "https://manga.instance.tld/comics/019a2b-chronicles-of-elyria",
  "type":,
  "name": "Chronicles of Elyria",
  "summary": "An expansive epic following the fall of the Elyrian Empire.",
  "published": "2024-03-15T10:00:00Z",
  "schema:author": {
    "type": "Person",
    "name": "Elara Vance"
  },
  "schema:penciler": {
    "type": "Person",
    "name": "Kaelen Thorne"
  },
  "schema:colorist": {
    "type": "Person",
    "name": "Jace Beleren"
  },
  "tag": [
    {
      "type": "Hashtag",
      "name": "#Fantasy",
      "href": "https://manga.instance.tld/tags/Fantasy"
    }
  ],
  "mag:readingProgression": "rtl",
  "mag:views": 154020,
  "attributedTo": "https://manga.instance.tld/groups/elyria-scans",
  "totalItems": 156,
  "first": "https://manga.instance.tld/comics/019a2b-chronicles-of-elyria/chapters?page=1",
  "last": "https://manga.instance.tld/comics/019a2b-chronicles-of-elyria/chapters?page=8"
}
```

The `mag:readingProgression` property represents a critical, domain-specific extension. By embedding this instruction directly into the JSON-LD, remote readers can automatically calibrate their user interface to support right-to-left swiping, honoring the original formatting of manga without requiring manual user intervention.

### The Chapter Object Schema (schema:ComicIssue)

The Chapter payload explicitly defines the sequential images within the mag:pages collection.

```json
{
  "@context": [
    "https://www.w3.org/ns/activitystreams",
    "https://w3id.org/security/v1",
    {
      "schema": "http://schema.org/",
      "mag": "https://magtivitypub.org/ns#",
      "toot": "http://joinmastodon.org/ns#",
      "blurhash": "toot:blurhash"
    }
  ],
  "id": "https://manga.instance.tld/chapters/99120-elyria-ch156",
  "type": ["Article", "schema:ComicIssue"],
  "name": "Chapter 156: The Shattered Crown",
  "schema:issueNumber": "156",
  "published": "2024-06-20T14:30:00Z",
  "updated": "2024-06-21T09:15:00Z",
  "partOf": "https://manga.instance.tld/comics/019a2b-chronicles-of-elyria",
  "attributedTo": "https://manga.instance.tld/groups/elyria-scans",
  "to": [
    "https://www.w3.org/ns/activitystreams#Public"
  ],
  "cc": [
    "https://manga.instance.tld/groups/elyria-scans/followers"
  ],
  "mag:views": 12050,
  "mag:pages": {
    "type": "OrderedCollection",
    "totalItems": 2,
    "orderedItems":
      },
      {
        "type": "Image",
        "name": "Page 2",
        "blurhash": "U7C?8}4n00_300~q%M%M00D%~qD%-;M{M{M{",
        "url":
      }
    ]
  }
}
```

The addressing fields, `to` and `cc`, govern visibility across the network. The inclusion of the `#Public` magic URI in the `to` array dictates that the Chapter is globally readable. The inclusion of the Group's `followers` endpoint in the `cc` array signals to receiving instances that they must parse their local databases for users following the Group and insert the Chapter into their respective chronological timelines.

## Collaborative Publishing and Joint-Release Federation

A persistent operational challenge within the scanlation community is the coordination of joint-releases. Frequently, multiple independent groups collaborate on a single chapter. Architecting a unified joint-release mechanism over a decentralized protocol requires the precise manipulation of ActivityPub attribution arrays and Announce flows.

### The Multi-Actor Attribution Pattern

While ActivityStreams objects generally feature a single URI string in the `attributedTo` field, the JSON-LD specification permits the value to be an array of URIs.

MagtivityPub leverages this paradigm to support cross-instance Joint-Releases. If a Chapter is collaboratively produced by Group Alpha (hosted on `origin.tld`) and Group Beta (hosted on `remote.tld`), the Chapter's object is instantiated strictly on `origin.tld`, but its metadata explicitly acknowledges both entities:

```json
"attributedTo": [
  "https://origin.tld/groups/group-alpha",
  "https://remote.tld/groups/group-beta"
]
```

### Server-to-Server Flow for Joint-Releases

To prevent data duplication while maximizing the algorithmic reach of both groups, MagtivityPub implements a synchronized broadcasting model based on delegated creation.

1. `Upload and Instantiation`: An authorized editor belonging to Group Alpha authenticates via their local instance and uploads the archive to `origin.tld`. The local server extracts the images and generates the `Chapter` object.
2. `Attribution Binding and Verification`: During the upload sequence, the editor specifies Group Beta as a collaborator. The server appends Group Beta's URI to the `attributedTo` array. Crucially, to prevent bad actors from arbitrarily associating unconnected groups with malicious content, `origin.tld` must dispatch an automated `Offer` activity to `remote.tld` (aligning with FEP-0837 for Federated Agreements). Group Beta's server processes the `Offer` and issues an `Accept` activity if a mutual collaboration agreement exists.
3. `The Primary Create Activity`: Upon validation, Group Alpha's outbox generates a `Create` activity wrapping the `Chapter` object and dispatches it to the inboxes of all remote servers hosting its followers.
4. `The Triggered Announce Activity`: Because Group Beta successfully accepted the Offer and is listed in the `attributedTo` array, the protocol mandates that Group Beta alert its own audience. `remote.tld` automatically generates an `Announce` (functionally identical to a share or "boost") activity from Group Beta's outbox to all of Group Beta's followers, referencing the canonical URI of the Chapter hosted on `origin.tld`.

This second-order dynamic—triggering an automated `Announce` from secondary actors—is the linchpin of federated joint-releases. It guarantees that the Chapter surfaces natively in the timelines of users following Group Beta, while consolidating all data, views, and comment threads on the origin server.

## Content Hierarchy and Metadata Resolution

Maintaining the integrity of the content hierarchy (`Group` → `ComicSeries` → `ComicIssue` → `Page`) across an inherently chaotic decentralized network requires strict fallback methodologies. When a Chapter federates to a remote instance, the receiving server parses the `partOf` and `attributedTo` fields to reconstruct the ownership chain, apply correct branding, and enforce UI authorizations.

However, MagtivityPub must federate with platforms that lack the concept of publishing channels or comic series. If a user on a pure microblogging application follows a MagtivityPub Group, the receiving server will parse the federated `Chapter` merely as an `Article` linked to an `Actor`.

The primary mitigation strategy involves the automated simulation of a `Group` actor as a highly constrained Person profile. Because a `Group` possesses an `inbox`, an `outbox`, and an avatar, a microblogging platform will simply render the `Group` as a standard user profile. The hierarchical chain terminates securely at the `Group` level for these legacy clients, preserving attribution without causing parser failures.

## Social Interactions and User Actions

MagtivityPub maps traditional comic reader behaviors onto the standardized suite of ActivityPub primitives.

| MagtivityPub User Action | ActivityPub Activity Type | Target Object | Federation Mechanics and Network Result |
|---|------|---------|-----------|
| Subscribe to Group | `Follow` | `Group` | A `Follow` activity is POSTed to the Group's inbox. The origin server responds with an `Accept`, and the user is added to the Group's followers collection. |
| Bookmark a Comic | `Follow` | `OrderedCollection` | The user subscribes directly to the `ComicSeries` entity. |
| Like a Chapter | `Like` | `Article` | The user's instance sends a `Like` to the origin server. The origin server increments its global counter and optionally broadcasts an `Update`. |
| Share a Chapter | `Announce` | `Article` | The user boosts the Chapter. Their instance generates an `Announce` activity, appending the `Chapter` to their outbox and pushing it to their personal followers. |
| Post a Comment | `Note` | `Article` | The user generates a reply. The comment is typed as a `Note` utilizing the `inReplyTo` property pointing to the `Chapter` URI. |
| Track Reading Progress | `View` | `Article` | Internal state tracking to manage "last read" bookmarks. Can be federated as generic `View` activities to calculate global viewership. |

### The Mechanics of Federated Comments

Because a `Chapter` acts fundamentally as an `Article`, comments submitted from remote Fediverse instances are handled as standard ActivityStreams `Note` objects.

When a user on a remote instance encounters a federated `Chapter` in their timeline and replies, their instance crafts a `Create` activity wrapping a `Note`. The crucial component of this `Note` is the `inReplyTo` property, which contains the exact URI of the origin MagtivityPub `Chapter`. The remote server POSTs this activity to the MagtivityPub server's inbox.

The MagtivityPub instance receives the payload, verifies the HTTP Signature, resolves the `inReplyTo` URI, and appends the comment to the Chapter's `replies` Collection. As the arbiter of truth for its hosted content, the origin MagtivityPub server retains the authority to moderate this thread; if a federated comment violates community guidelines, the origin server issues a `Reject` or `Delete` activity, severing it from the local display.

## Vulnerability Mitigation and Strict Type Checking

The openness of the `attributedTo` and `partOf` properties introduces unique vulnerability vectors. Security researchers have previously identified flaws in federated software where an attacker could dispatch a `Create` activity targeting a remote collection, forging the `attributedTo` array to force the remote server to append malicious content to an innocent user's channel.

MagtivityPub counteracts this via strict permission validation during payload digestion. If an incoming `Create` or `Add` activity attempts to append a `Chapter` to a local `ComicSeries` or `Group`, the server explicitly verifies that the Actor signing the HTTP request is definitively registered within the local database as an authorized `Editor` or `Owner` of that specific entity. If the cryptographic signature belongs to an unauthorized actor, the activity is rejected entirely.

