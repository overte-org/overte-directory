# ID/Addresses schemas

> [!CAUTION]
> This is a WIP design document and specifics are subject to change until finalization.

These are the planed schemas for entities in the federation system.

In general the schema for all entities are in the following form:

```
<entity-prefix><identifier>:<directory-domain>
```

Usually entities are identified by a alias. These aliases are not permanent and can be chenged to another entity.

To have a more stable reference, entities have associated permanent identifiers. These identifiers have the prefix `~`.

Currently, there are these entity prefixes are:
username: `@`
worlds: `#`
groups: `!`
