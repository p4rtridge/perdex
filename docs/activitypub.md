---
outline: deep
---

# ActivityPub

Each Perdex instance is able to fetch comics and chapters from other compatible servers it follows, in a process known as “federation”. Federation is implemented using the ActivityPub protocol, in order to leverage existing tools and be compatible with other services such as Mastodon, Pleroma and many more.

Federation in Perdex is twofold: comics metadata and sequential pages are shared as activities for inter-server communication in what amounts to sharing parts of one's database, and user interaction via comments which are compatible with the kind of activity textual platforms like Mastodon use.


## Federate with Perdex

Perdex uses publishing groups (`Group` actor) to organise content. A `Group` is owned by an account (`Person` actor, referenced via `attributedTo`) and has a public `members` collection of contributing accounts.

**Publishing flow**

A `Comic` or `Chapter` is published by a group member and amplified by the group:

1. A `Person` who is a member of a `Group` sends `Create(Comic)` or `Create(Chapter)` with `attributedTo` set to the `Group` URI.
2. The receiving server validates that the actor is in `Group.members`.
3. The `Group` automatically sends `Announce` to its followers, propagating the content across the network.

Both `Comic` and `Chapter` carry `attributedTo` pointing to the publishing `Group`. The `Group` in turn carries `attributedTo` pointing to its owner `Person`, so any remote server can trace the full ownership chain.

**Cross-server membership**

A user on server B can join a group on server A by sending `Join(Group@A)`. Once accepted, they can `Create` content attributed to `Group@A` and the server will validate their membership by fetching `Group.members`.

**Platforms without the Group concept**

If your platform does not have publishing groups, we recommend one of the following approaches:
- Simulate a `Group` actor for each account by appending a suffix (e.g. `alice_group@your.instance`), auto-accepting all `Join` requests from that account's `Person` actor.
- Use a single global `Group` actor owned by a system-level `Person` that proxies all publications from your instance.

## Namespace Extensions

Perdex extends ActivityPub using a custom JSON-LD namespace `https://perdex.network/ns#` (aliased as `mag:`). All custom properties must be declared in the `@context` of the object that uses them.

| Term | Used on | Description |
|---|---|---|
| `mag:Comic` | — | Custom type extending `as:Document`; represents a root comic series |
| `mag:Chapter` | — | Custom type extending `as:Page`; represents a single chapter |
| `mag:chapters` | `Comic` | Link to the `OrderedCollection` of a comic's chapters |
| `mag:members` | `Group` | Link to the public `Collection` of group members |
| `mag:manuallyApprovesMembers` | `Group` | If `true`, join requests require owner approval; if `false`, auto-accepted |
| `mag:readingProgression` | `Comic`, `Chapter` | Reading direction: `"ltr"` or `"rtl"` |
| `mag:status` | `Comic` | Publication status: `"ongoing"`, `"completed"`, or `"hiatus"` |
| `mag:issueNumber` | `Chapter` | Chapter number within the comic series |
| `mag:volumeNumber` | `Chapter` | Volume number |
| `mag:language` | `Chapter` | ISO 639-1 language code (e.g. `"en"`, `"vi"`) |
| `mag:views` | `Comic`, `Chapter`, `Actor` | Cumulative view count |

## Supported Objects

- [Actor](#actor)
- [Comic](#comic)
- [Chapter](#chapter)
- [Note](#note)
- [Image](#image)

## Supported Activities

- [Create](#create)
- [Update](#update)
- [Delete](#delete)
- [Follow](#follow)
- [Accept](#accept)
- [Reject](#reject)
- [Announce](#announce)
- [Undo](#undo)
- [Join](#join)
- [Leave](#leave)
- [Remove](#remove)
- [Like](#like)
- [Dislike](#dislike)
- [View](#view)

## Objects

### Actor

A Perdex Actor can be:

- An account (`Person`), used to publish or comment comics, report abuses or create publishing groups.
- A publishing group (`Group`), owned by an account and also used to publish chapters. The publisher is a `Group` because we can have multiple accounts that manage the same group.

When a chapter is published, the account sends a `Create` activity and the group sends an `Announce` activity, both referencing the `Chapter` object. A `Comic` object has the `attributedTo` property to know the `Person` and the `Group` that own it.

::: code-group

```json [Account Example]
{
  "@context": [
    "https://www.w3.org/ns/activitystreams",
    "https://w3id.org/security/v1",
    "https://purl.archive.org/socialweb/webfinger",
    {
      "mag": "https://perdex.network/ns#",
      "sc": "http://schema.org/",
      "manuallyApprovesFollowers": "as:manuallyApprovesFollowers",
      "comics": {
        "@type": "@id",
        "@id": "mag:comics"
      }
    }
  ],
  "id": "https://manga.instance.tld/users/admin-alice",
  "webfinger": "admin-alice@manga.instance.tld",
  "type": "Person",
  "preferredUsername": "admin-alice",
  "name": "Alice The Editor",
  "summary": "Manga enthusiast and lead translator.",
  "inbox": "https://manga.instance.tld/users/admin-alice/inbox",
  "outbox": "https://manga.instance.tld/users/admin-alice/outbox",
  "followers": "https://manga.instance.tld/users/admin-alice/followers",
  "following": "https://manga.instance.tld/users/admin-alice/following",
  "comics": "https://manga.instance.tld/users/admin-alice/comics",
  "manuallyApprovesFollowers": false,
  "views": 1000
}
```

```json [Group Example]
{
  "@context": [
    "https://www.w3.org/ns/activitystreams",
    "https://w3id.org/security/v1",
    {
      "mag": "https://perdex.network/ns#",
      "sc": "http://schema.org/",
      "manuallyApprovesMembers": "mag:manuallyApprovesMembers",
      "members": { "@type": "@id", "@id": "mag:members" },
      "comics": { "@type": "@id", "@id": "mag:comics" },
      "views": { "@type": "sc:Number", "@id": "mag:views" }
    }
  ],
  "id": "https://manga.instance.tld/groups/elyria-scans",
  "type": "Group",
  "preferredUsername": "elyria-scans",
  "name": "Elyria Scanlation Group",
  "summary": "Translating the best fantasy epics.",
  "inbox": "https://manga.instance.tld/groups/elyria-scans/inbox",
  "outbox": "https://manga.instance.tld/groups/elyria-scans/outbox",
  "followers": "https://manga.instance.tld/groups/elyria-scans/followers",
  "members": "https://manga.instance.tld/groups/elyria-scans/members",
  "comics": "https://manga.instance.tld/groups/elyria-scans/comics",
  "attributedTo": "https://manga.instance.tld/users/admin-alice",
  "manuallyApprovesMembers": false,
  "views": 1000
}
```

:::

### Comic

::: info
This object extends the ActivityPub specification and maps to the [Document](https://www.w3.org/TR/activitystreams-vocabulary/#dfn-document) ActivityStreams type. The custom `Comic` type is declared in the `mag:` namespace.
:::

A `Comic` represents a root comic series. It exposes:

- **`chapters`** — an endpoint returning the `OrderedCollection` of all chapters belonging to the comic.
- **`followers`** — an endpoint that allows users to `Follow` the comic directly and receive notifications on new chapter releases, regardless of which group or server publishes them.
- **`tag`** — an array of `Hashtag` objects (Mastodon-compatible) representing the comic's genres.
- **`author`** — an array of `Author` objects, each with their own URI endpoint.
- **`artist`** — an array of `Artist` objects, each with their own URI endpoint.

::: code-group

```json [Comic Example]
{
  "@context": [
    "https://www.w3.org/ns/activitystreams",
    "https://w3id.org/security/v1",
    {
      "mag": "https://perdex.network/ns#",
      "sc": "http://schema.org/",
      "Comic": "mag:Comic",
      "author": "sc:author",
      "artist": "sc:artist",
      "readingProgression": "mag:readingProgression",
      "status": "mag:status",
      "chapters": { "@type": "@id", "@id": "mag:chapters" },
      "views": { "@type": "sc:Number", "@id": "mag:views" }
    }
  ],
  "id": "https://manga.instance.tld/comics/019a2b-chronicles-of-elyria",
  "type": "Comic",
  "name": "Chronicles of Elyria",
  "summary": "An expansive epic following the fall of the Elyrian Empire.",
  "published": "2024-03-15T10:00:00Z",
  "author": [
    { "type": "Person", "id": "https://manga.instance.tld/authors/019x-elara-vance", "name": "Elara Vance" }
  ],
  "artist": [
    { "type": "Person", "id": "https://manga.instance.tld/artists/019x-kaelen-thorne", "name": "Kaelen Thorne" }
  ],
  "readingProgression": "rtl",
  "status": "ongoing",
  "tag": [
    { "type": "Hashtag", "name": "#fantasy", "href": "https://manga.instance.tld/tags/fantasy" },
    { "type": "Hashtag", "name": "#action",  "href": "https://manga.instance.tld/tags/action" }
  ],
  "chapters": {
    "type": "OrderedCollection",
    "id": "https://manga.instance.tld/comics/019a2b-chronicles-of-elyria/chapters",
    "totalItems": 156
  },
  "followers": "https://manga.instance.tld/comics/019a2b-chronicles-of-elyria/followers",
  "views": 154020,
  "attributedTo": "https://manga.instance.tld/groups/elyria-scans"
}
```

:::

### Chapter

::: info
This object extends the ActivityPub specification and maps to the [Page](https://www.w3.org/TR/activitystreams-vocabulary/#dfn-page) ActivityStreams type. The custom `Chapter` type is declared in the `mag:` namespace.
:::

A `Chapter` represents a discrete, sequential chapter. The sequential images that make up the chapter are wrapped in an [OrderedCollection](https://www.w3.org/TR/activitystreams-vocabulary/#dfn-orderedcollection) provided in the standard attachment array.

Key differences from standard ActivityPub platforms:

- **`attributedTo`** is an **array** to support joint-releases — multiple groups co-publishing a single chapter.
- **`cc`** must include the `followers` collection of every participating group as well as the parent comic's own `followers` endpoint, ensuring all subscribers receive the notification.

::: code-group

```json [Chapter Example (Single Group)]
{
  "@context": [
    "https://www.w3.org/ns/activitystreams",
    "https://w3id.org/security/v1",
    {
      "mag": "https://perdex.network/ns#",
      "sc": "http://schema.org/",
      "Chapter": "mag:Chapter",
      "issueNumber": "mag:issueNumber",
      "volumeNumber": "mag:volumeNumber",
      "language": "mag:language",
      "readingProgression": "mag:readingProgression",
      "views": { "@type": "sc:Number", "@id": "mag:views" }
    }
  ],
  "id": "https://manga.instance.tld/chapters/99120-elyria-ch156",
  "type": "Chapter",
  "name": "The Shattered Crown",
  "issueNumber": "156",
  "volumeNumber": "12",
  "language": "en",
  "published": "2024-06-20T14:30:00Z",
  "partOf": "https://manga.instance.tld/comics/019a2b-chronicles-of-elyria",
  "attributedTo": [
    "https://manga.instance.tld/groups/elyria-scans"
  ],
  "to": [
    "https://www.w3.org/ns/activitystreams#Public"
  ],
  "cc": [
    "https://manga.instance.tld/groups/elyria-scans/followers",
    "https://manga.instance.tld/comics/019a2b-chronicles-of-elyria/followers"
  ],
  "views": 12050
}
```

```json [Chapter Example (Joint Release)]
{
  "@context": [
    "https://www.w3.org/ns/activitystreams",
    "https://w3id.org/security/v1",
    {
      "mag": "https://perdex.network/ns#",
      "sc": "http://schema.org/",
      "Chapter": "mag:Chapter",
      "issueNumber": "mag:issueNumber",
      "volumeNumber": "mag:volumeNumber",
      "language": "mag:language",
      "readingProgression": "mag:readingProgression",
      "views": { "@type": "sc:Number", "@id": "mag:views" }
    }
  ],
  "id": "https://manga.instance.tld/chapters/99120-elyria-ch156",
  "type": "Chapter",
  "name": "The Shattered Crown",
  "issueNumber": "156",
  "volumeNumber": "12",
  "language": "vi",
  "published": "2024-06-20T14:30:00Z",
  "partOf": "https://manga.instance.tld/comics/019a2b-chronicles-of-elyria",
  "attributedTo": [
    "https://manga.instance.tld/groups/elyria-scans",
    "https://b.instance.tld/groups/shadow-tlrs"
  ],
  "to": [
    "https://www.w3.org/ns/activitystreams#Public"
  ],
  "cc": [
    "https://manga.instance.tld/groups/elyria-scans/followers",
    "https://b.instance.tld/groups/shadow-tlrs/followers",
    "https://manga.instance.tld/comics/019a2b-chronicles-of-elyria/followers"
  ],
  "views": 12050
}
```

:::

Announce flow for a joint-release:

```
Person (uploader)  →  Create(Chapter)
Group A            →  Announce(Chapter)   ← followers of Group A are notified
Group B            →  Announce(Chapter)   ← followers of Group B are notified
                                            ← direct followers of the Comic are also notified (via cc)
```

### Note

::: info
This object uses the standard [Note](https://www.w3.org/TR/activitystreams-vocabulary/#dfn-note) ActivityStreams type and is fully compatible with Mastodon and other AP platforms.
:::

A `Note` represents a comment left by a user on a `Comic`, a `Chapter`, or another `Note` (reply). Threading is achieved via the standard `inReplyTo` property.

- **Top-level comment**: `inReplyTo` points to a `Comic` or `Chapter`.
- **Reply**: `inReplyTo` points to another `Note`.

::: code-group

```json [Comment on a Comic]
{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Note",
  "id": "https://manga.instance.tld/comments/abc-001",
  "content": "This is an amazing comic, can\'t wait for the next chapter!",
  "attributedTo": "https://manga.instance.tld/users/admin-alice",
  "inReplyTo": "https://manga.instance.tld/comics/019a2b-chronicles-of-elyria",
  "published": "2024-06-21T09:00:00Z",
  "to": ["https://www.w3.org/ns/activitystreams#Public"],
  "cc": ["https://manga.instance.tld/users/admin-alice/followers"]
}
```

```json [Reply to a Comment]
{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Note",
  "id": "https://manga.instance.tld/comments/abc-002",
  "content": "Totally agree, the art style is incredible.",
  "attributedTo": "https://manga.instance.tld/users/bob-translator",
  "inReplyTo": "https://manga.instance.tld/comments/abc-001",
  "published": "2024-06-21T09:15:00Z",
  "to": [
    "https://www.w3.org/ns/activitystreams#Public",
    "https://manga.instance.tld/users/admin-alice"
  ],
  "cc": ["https://manga.instance.tld/users/bob-translator/followers"]
}
```

:::

### Image

An `Image` represents a single page within a chapter. `Image` objects are **not** published as standalone activities — they are always embedded inside a `Chapter`'s `attachment` field as an `OrderedCollection`.

Each `Image` carries:
- **`url`** — direct URL to the image file.
- **`mediaType`** — MIME type (`image/jpeg`, `image/png`, `image/webp`, etc.).
- **`width`** / **`height`** — pixel dimensions, used by clients for layout pre-calculation.
- **`name`** — optional page label (e.g. `"Page 1"`).

::: code-group

```json [Chapter with Image attachment]
{
  "@context": [
    "https://www.w3.org/ns/activitystreams",
    {
      "mag": "https://perdex.network/ns#",
      "Chapter": "mag:Chapter",
      "issueNumber": "mag:issueNumber",
      "language": "mag:language"
    }
  ],
  "type": "Chapter",
  "id": "https://manga.instance.tld/chapters/99120-elyria-ch156",
  "issueNumber": "156",
  "language": "en",
  "partOf": "https://manga.instance.tld/comics/019a2b-chronicles-of-elyria",
  "attributedTo": ["https://manga.instance.tld/groups/elyria-scans"],
  "attachment": {
    "type": "OrderedCollection",
    "totalItems": 24,
    "orderedItems": [
      {
        "type": "Image",
        "mediaType": "image/jpeg",
        "url": "https://manga.instance.tld/media/chapters/99120/page-001.jpg",
        "name": "Page 1",
        "width": 1200,
        "height": 1800
      },
      {
        "type": "Image",
        "mediaType": "image/jpeg",
        "url": "https://manga.instance.tld/media/chapters/99120/page-002.jpg",
        "name": "Page 2",
        "width": 1200,
        "height": 1800
      }
    ]
  }
}
```

:::

## Activities

### Create

Create is an activity standardized in the ActivityPub specification (see [Create Activity](https://www.w3.org/TR/activitypub/#create-activity-inbox)). The Create activity is used when posting a new object. This has the side effect that the `object` embedded within the Activity (in the `object` property) is created.

**Authorization**
- A `Comic` **must** have `attributedTo` set to a `Group`. Creating a standalone Comic without a Group is not permitted.
- A `Chapter` **must** have `attributedTo` set to one or more `Group`s.
- The actor must be a verified member of every Group referenced in `attributedTo`. The receiving server validates against `Group.members`.
- After a successful `Create`, each referenced `Group` automatically sends `Announce` to its followers.

**Supported on**
- [Comic](#comic)
- [Chapter](#chapter)
- [Note](#note)

::: code-group

```json [Create(Comic)]
{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Create",
  "actor": "https://manga.instance.tld/users/admin-alice",
  "to": ["https://www.w3.org/ns/activitystreams#Public"],
  "cc": ["https://manga.instance.tld/groups/elyria-scans/followers"],
  "object": {
    "type": "Comic",
    "id": "https://manga.instance.tld/comics/019a2b-chronicles-of-elyria",
    "name": "Chronicles of Elyria",
    "attributedTo": "https://manga.instance.tld/groups/elyria-scans"
  }
}
```

```json [Create(Chapter)]
{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Create",
  "actor": "https://manga.instance.tld/users/admin-alice",
  "to": ["https://www.w3.org/ns/activitystreams#Public"],
  "cc": [
    "https://manga.instance.tld/groups/elyria-scans/followers",
    "https://manga.instance.tld/comics/019a2b-chronicles-of-elyria/followers"
  ],
  "object": {
    "type": "Chapter",
    "id": "https://manga.instance.tld/chapters/99120-elyria-ch156",
    "issueNumber": "156",
    "partOf": "https://manga.instance.tld/comics/019a2b-chronicles-of-elyria",
    "attributedTo": ["https://manga.instance.tld/groups/elyria-scans"]
  }
}
```

```json [Create(Note)]
{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Create",
  "actor": "https://manga.instance.tld/users/admin-alice",
  "to": ["https://www.w3.org/ns/activitystreams#Public"],
  "cc": ["https://manga.instance.tld/users/admin-alice/followers"],
  "object": {
    "type": "Note",
    "id": "https://manga.instance.tld/comments/abc-001",
    "content": "This is an amazing comic!",
    "attributedTo": "https://manga.instance.tld/users/admin-alice",
    "inReplyTo": "https://manga.instance.tld/comics/019a2b-chronicles-of-elyria",
    "published": "2024-06-21T09:00:00Z"
  }
}
```

:::


### Update

Update is an activity standardized in the ActivityPub specification (see [Update Activity](https://www.w3.org/TR/activitypub/#update-activity-inbox)). The Update activity is used when updating an already existing object.

**Supported on**
- [Comic](#comic)
- [Chapter](#chapter)
- [Note](#note)
- [Actor](#actor)

### Delete

Delete is an activity standardized in the ActivityPub specification (see [Delete Activity](https://www.w3.org/TR/activitypub/#delete-activity-outbox)).

**Supported on**
- [Comic](#comic)
- [Chapter](#chapter)
- [Note](#note)
- [Actor](#actor)

### Follow

Follow is an activity standardized in the ActivityPub specification (see [Follow Activity](https://www.w3.org/TR/activitypub/#follow-activity-inbox)). The Follow activity is used to subscribe to the activities of another actor.

Perdex supports `Follow` on three distinct targets, each with different delivery semantics:

| Target | What you receive |
|---|---|
| `Person` | Activities from that user's outbox (standard social follow) |
| `Group` | All `Announce` activities from the group — both `Announce(Chapter)` on new chapter releases **and** `Announce(Comic)` when the group picks up a new comic project |
| `Comic` | Included in the `cc` of every new `Chapter` published under that comic, regardless of which group publishes it. Also acts as the user's personal reading list bookmark. Use `Undo(Follow)` to unfollow. |

**Supported on**
- [Actor](#actor)
- [Comic](#comic)

### Accept

Accept is an activity standardized in the ActivityPub specification (see [Accept Activity](https://www.w3.org/TR/activitypub/#accept-activity-inbox)). The Accept activity is used to accept a `Follow` or `Join` activity.

When a `Group` has `manuallyApprovesMembers: false`, the server automatically sends `Accept(Join)` and adds the actor to the `members` collection. When `manuallyApprovesMembers: true`, the group owner sends `Accept(Join)` manually.

**Supported on**
- [Follow](#follow)
- [Join](#join)

### Reject

Reject is an activity standardized in the ActivityPub specification (see [Reject Activity](https://www.w3.org/TR/activitypub/#reject-activity-inbox)). The Reject activity is used to decline a `Join` request when `manuallyApprovesMembers` is `true`.

**Supported on**
- [Join](#join)

### Announce

Announce is an activity standardized in the ActivityPub specification (see [Announce Activity](https://www.w3.org/TR/activitypub/#announce-activity-outbox)). The Announce activity is used to announce the sharing of an object.

In Perdex, a `Group` automatically sends `Announce` whenever one of its members creates a `Comic` or `Chapter` attributed to the Group. This notifies all Group followers of new content.

```
# Comic flow
Person (group member)  →  Create(Comic  { attributedTo: Group })
Group                  →  Announce(Comic)   ← Group followers notified of new project

# Chapter flow (single group)
Person (group member)  →  Create(Chapter { attributedTo: [Group] })
Group                  →  Announce(Chapter) ← Group followers + Comic followers notified

# Chapter flow (joint release)
Person (uploader)      →  Create(Chapter { attributedTo: [Group A, Group B] })
Group A                →  Announce(Chapter) ← Group A followers notified
Group B                →  Announce(Chapter) ← Group B followers notified
                                             ← Comic followers notified (via cc)
```

**Supported on**
- [Comic](#comic)
- [Chapter](#chapter)

### Undo

Undo is an activity standardized in the ActivityPub specification (see [Undo Activity](https://www.w3.org/TR/activitypub/#undo-activity-inbox)). The Undo activity is used to undo a previous activity.

**Supported on**
- [Follow](#follow)
- [Join](#join) (pending requests only)
- [Announce](#announce)
- [Like](#like)

### Like

Like is an activity standardized in the ActivityPub specification (see [Like Activity](https://www.w3.org/TR/activitypub/#like-activity-inbox)).

**Supported on**
- [Comic](#comic)
- [Note](#note)

### Dislike

Dislike is an activity standardized in the ActivityStream specification (see [Dislike Activity](https://www.w3.org/TR/activitystreams-vocabulary/#dfn-dislike)).

**Supported on**
- [Comic](#comic)
- [Note](#note)


### View

View is an activity standardized in the ActivityStream specification (see [View Activity](https://www.w3.org/TR/activitystreams-vocabulary/#dfn-view)).

A Perdex platform sends a `View` activity every time a user read a comic so the origin server can increment the comic's view counts.

The `View` activity includes an `expires` attribute, it means a user is currently reading the comic. This kind of event is sent periodically until the user stops reading the comic. The same `View` action can be sent multiple times using a different expires attribute, meaning the user is still reading the comic.

**Supported on**
- [Comic](#comic)
- [Chapter](#chapter)

### Join

Join is an activity standardized in the ActivityStreams specification (see [Join Activity](https://www.w3.org/TR/activitystreams-vocabulary/#dfn-join)). In Perdex, `Join` is used when a user requests to become a member of a publishing group.

If `manuallyApprovesMembers` is `false` on the target `Group`, the server automatically sends `Accept(Join)` and adds the actor to the `members` collection. If `manuallyApprovesMembers` is `true`, the request is pending until the group owner manually sends `Accept(Join)` or `Reject(Join)`.

Any member can upload chapters on behalf of the group once accepted.

**Supported on**
- [Actor](#actor) (targeting a `Group`)

::: code-group

```json [Join Request]
{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Join",
  "actor": "https://manga.instance.tld/users/bob-translator",
  "object": "https://manga.instance.tld/groups/elyria-scans"
}
```

```json [Accept(Join) Response]
{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Accept",
  "actor": "https://manga.instance.tld/groups/elyria-scans",
  "object": {
    "type": "Join",
    "actor": "https://manga.instance.tld/users/bob-translator",
    "object": "https://manga.instance.tld/groups/elyria-scans"
  }
}
```

```json [Reject(Join) Response]
{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Reject",
  "actor": "https://manga.instance.tld/groups/elyria-scans",
  "object": {
    "type": "Join",
    "actor": "https://manga.instance.tld/users/bob-translator",
    "object": "https://manga.instance.tld/groups/elyria-scans"
  }
}
```

:::

### Leave

Leave is an activity standardized in the ActivityStreams specification (see [Leave Activity](https://www.w3.org/TR/activitystreams-vocabulary/#dfn-leave)). A user sends `Leave` to voluntarily exit a group. The server removes the actor from the `members` collection.

**Supported on**
- [Actor](#actor) (targeting a `Group`)

::: code-group

```json [Leave Example]
{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Leave",
  "actor": "https://manga.instance.tld/users/bob-translator",
  "object": "https://manga.instance.tld/groups/elyria-scans"
}
```

:::

### Remove

Remove is an activity standardized in the ActivityPub specification (see [Remove Activity](https://www.w3.org/TR/activitypub/#remove-activity-outbox)). In Perdex, `Remove` is used by the group owner to kick a member from the group. The `object` is the `Person` being removed and the `target` is the `Group`'s `members` collection.

Only the group owner (`Group.attributedTo`) is authorized to send this activity.

**Supported on**
- [Actor](#actor) (removing a `Person` from a `Group`)

::: code-group

```json [Remove Example]
{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Remove",
  "actor": "https://manga.instance.tld/users/admin-alice",
  "object": "https://manga.instance.tld/users/bob-translator",
  "target": "https://manga.instance.tld/groups/elyria-scans/members"
}
```

:::