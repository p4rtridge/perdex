---
outline: deep
---

# ActivityPub

Each Perdex instance is able to fetch comics and chapters from other compatible servers it follows, in a process known as “federation”. Federation is implemented using the ActivityPub protocol, in order to leverage existing tools and be compatible with other services such as Mastodon, Pleroma and many more.

Federation in Perdex is twofold: comics metadata and sequential pages are shared as activities for inter-server communication in what amounts to sharing parts of one's database, and user interaction via comments which are compatible with the kind of activity textual platforms like Mastodon use.


## Federate with Perdex

Perdex has the concept of publishing groups (`Group` actor) that are owned by accounts (`Person` actor). In ActivityPub, a Perdex chapter is created by the account and `Announce`d by the publishing group.

To federate chapters, Perdix requires the chapter to have an `attributedTo` field that contains a `Group` actor. The `Group` actor also contains an `attributedTo` field pointing to the owner `Person` actor.

If your reading platform doesn't have the concept of groups, we recommend you either:
- simulate a `Group` actor representing your account (automatically adding `_group` to your account username example)
- put your `Person` actor as a `Group` actor in `attributedTo` that is owned by a global `Person` actor that would own all your fake groups.


## Supported Objects

- [Actor](#actor)
- [Document](#document)
- [Page](#page)
- Image (mapped to Chapter pages)
- Note (mapped to Comment)

## Supported Activities

- [Create](#create)
- [Update](#update)
- [Delete](#delete)
- [Follow](#follow)
- [Accept](#accept)
- [Announce](#announce)
- [Undo](#undo)
- [Like](#like)
- [Dislike](#dislike)
- [View](#view)

## Objects

### Actor

A Perdex Actor can be:

- An account (`Person`), used to publish or comment comics, report abuses or create publishing groups.
- A publishing group (`Group`), owned by an account and also used to publish chapters. The publisher is a `Group` because we can have multiple accounts that manage the same group.

When a chapter is published, the account sends a `Create` activity and the group sends an `Announce` activity, both referencing the `Page` object. A `Document` object has the `attributedTo` property to know the `Person` and the `Group` that own it.

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
      "comics": {
        "@type": "@id",
        "@id": "mag:comics"
      },
      "views": {
        "@type": "sc:Number",
        "@id": "mag:views"
      }
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
  "comics": "https://manga.instance.tld/groups/elyria-scans/comics",
  "attributedTo": "https://manga.instance.tld/users/admin-alice",
  "views": 1000
}
```

:::

### Document

::: info
This object extends the ActivityPub specification, and therefore some properties are not part of it.
:::

It uses the [Document](https://www.w3.org/TR/activitystreams-vocabulary/#dfn-document) ActivityStreams type to represent a root comic series.

::: code-group

```json [Document Example]
{
  "@context": [
    "https://www.w3.org/ns/activitystreams",
    "https://w3id.org/security/v1",
    {
      "mag": "https://perdex.network/ns#",
      "sc": "http://schema.org/",
      "author": "sc:author",
      "artist": "sc:artist",
      "readingProgression": "mag:readingProgression",
      "comics": {
        "@type": "@id",
        "@id": "mag:comics"
      },
      "views": {
        "@type": "sc:Number",
        "@id": "mag:views"
      }
    }
  ],
  "id": "https://manga.instance.tld/comics/019a2b-chronicles-of-elyria",
  "type": "Document",
  "name": "Chronicles of Elyria",
  "summary": "An expansive epic following the fall of the Elyrian Empire.",
  "published": "2024-03-15T10:00:00Z",
  "author": {
    "type": "Person",
    "name": "Elara Vance"
  },
  "artist": {
    "type": "Person",
    "name": "Kaelen Thorne"
  },
  "readingProgression": "rtl",
  "views": 154020,
  "attributedTo": "https://manga.instance.tld/groups/elyria-scans"
}
```

:::

### Page

::: info
This object extends the ActivityPub specification, and therefore some properties are not part of it.
:::

It uses the [Page](https://www.w3.org/TR/activitystreams-vocabulary/#dfn-page) ActivityStreams type to represent a discrete, sequential chapter. The sequential images that make up the chapter are wrapped in an [OrderedCollection](https://www.w3.org/TR/activitystreams-vocabulary/#dfn-orderedcollection) provided in the standard attachment array.

::: code-group

```json [Page Example]
{
  "@context": [
    "https://www.w3.org/ns/activitystreams",
    "https://w3id.org/security/v1",
    {
      "mag": "https://perdex.network/ns#",
      "sc": "http://schema.org/",
      "author": "sc:author",
      "artist": "sc:artist",
      "issueNumber": "mag:issueNumber",
      "readingProgression": "mag:readingProgression",
      "comics": {
        "@type": "@id",
        "@id": "mag:comics"
      },
      "views": {
        "@type": "sc:Number",
        "@id": "mag:views"
      }
    }
  ],
  "id": "https://manga.instance.tld/chapters/99120-elyria-ch156",
  "type": "Page",
  "name": "The Shattered Crown",
  "issueNumber": "156",
  "published": "2024-06-20T14:30:00Z",
  "partOf": "https://manga.instance.tld/comics/019a2b-chronicles-of-elyria",
  "attributedTo": [
    "https://manga.instance.tld/groups/elyria-scans"
  ],
  "to": [
    "https://www.w3.org/ns/activitystreams#Public"
  ],
  "cc": [
    "https://manga.instance.tld/groups/elyria-scans/followers"
  ],
  "views": 12050
}
```

:::

## Activities

### Create

Create is an activity standardized in the ActivityPub specification (see [Create Activity](https://www.w3.org/TR/activitypub/#create-activity-inbox)). The Create activity is used when posting a new object. This has the side effect that the `object` embedded within the Activity (in the `object` property) is created.

**Supported on**
- [Document](#document)
- [Page](#page)

::: code-group

```json [Create Activity Example]
{
  "@context": [
    "https://www.w3.org/ns/activitystreams",
    "https://w3id.org/security/v1"
  ],
  "type": "Create",
  "to": [
    "https://www.w3.org/ns/activitystreams#Public"
  ],
  "cc": [
    "https://manga.instance.tld/groups/elyria-scans/followers"
  ],
  "actor": "https://manga.instance.tld/users/admin-alice",
  "object": {}
}
```

:::


### Update

Update is an activity standardized in the ActivityPub specification (see [Update Activity](https://www.w3.org/TR/activitypub/#update-activity-inbox)). The Update activity is used when updating an already existing object.

**Supported on**
- [Document](#document)
- [Page](#page)
- [Actor](#actor)

### Delete

Delete is an activity standardized in the ActivityPub specification (see [Delete Activity](https://www.w3.org/TR/activitypub/#delete-activity-outbox)).

**Supported on**
- [Document](#document)
- [Page](#page)
- [Actor](#actor)

### Follow

Follow is an activity standardized in the ActivityPub specification (see [Follow Activity](https://www.w3.org/TR/activitypub/#follow-activity-inbox)). The Follow activity is used to subscribe to the activities of another actor.

**Supported on**
- [Actor](#actor)

### Accept

Accept is an activity standardized in the ActivityPub specification (see [Accept Activity](https://www.w3.org/TR/activitypub/#accept-activity-inbox)). The Accept activity is used to accept a Follow activity.

**Supported on**
- [Actor](#actor)

### Announce

Announce is an activity standardized in the ActivityPub specification (see [Announce Activity](https://www.w3.org/TR/activitypub/#announce-activity-outbox)). The Announce activity is used to announce the sharing of an object.

**Supported on**
- [Page](#page)

### Undo

Undo is an activity standardized in the ActivityPub specification (see [Undo Activity](https://www.w3.org/TR/activitypub/#undo-activity-inbox)). The Undo activity is used to undo a previous activity.

**Supported on**
- [Follow](#follow)
- [Announce](#announce)

### Like

Like is an activity standardized in the ActivityPub specification (see [Like Activity](https://www.w3.org/TR/activitypub/#like-activity-inbox)).

**Supported on**
- [Document](#document)
- [Note](#note)

### Dislike

Dislike is an activity standardized in the ActivityStream specification (see [Dislike Activity](https://www.w3.org/TR/activitystreams-vocabulary/#dfn-dislike)).

**Supported on**
- [Document](#document)
- [Note](#note)


### View

View is an activity standardized in the ActivityStream specification (see [View Activity](https://www.w3.org/TR/activitystreams-vocabulary/#dfn-view)).

A Perdex platform sends a `View` activity every time a user read a comic so the origin server can increment the comic's view counts.

The `View` activity includes an `expires` attribute, it means a user is currently reading the comic. This kind of event is sent periodically until the user stops reading the comic. The same `View` action can be sent multiple times using a different expires attribute, meaning the user is still reading the comic.

**Supported on**
- [Document](#document)