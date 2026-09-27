# The `patterns/` corpus

The portable, content-addressed set of `.ure` manifests. This tree is the
**authoritative** artifact. `database/` is a rebuildable index over it, and
`unia index --rebuild` must be able to reconstruct the database from this tree
alone.

## Layout

```
patterns/
├── _schema/          meta: JSON Schema, reserved keys, versioning rules
├── _quarantine/      failed verification. Retained for audit, never served
├── identity/         who the caller is
├── skill/            a capability an agent can invoke
├── config/           configuration consumed by other resources
├── job/              a unit of scheduled work
├── hardware/         a physical or virtual device
├── tool/             an external service or binary
├── doc/              reference material
└── <class>/<target>/ the manifest
```

`<target>` is one of `native-linux`, `native-macos`, `native-windows`, `cuda`,
`rocm`, `metal`, `wasm`, `browser`, `pure`.

Example: `patterns/actuator/native-linux/smart_valve.ure`

**Actuator is not a class in the enumeration.** It is the current
`category` value used by the two migrated manifests, and it does not appear in
`resource_class`. This is unresolved divergence **D8** (see `docs/SPEC.md`): the
specification's `category` is a free-text functional class with examples like
`actuator` and `sensor`, while the seven values in specification section 4.2
are a different, non-overlapping vocabulary. The directory currently uses
`category` values, the database uses `resource_class` values, and they are not
the same axis. Pick one before this corpus grows.

## Why two levels and not more

`ls patterns/actuator/native-linux` answers the two questions a human actually
asks — *what kind of thing is this* and *will it run here* — and nothing deeper
is needed, because everything else is a query the database answers better.

Deeper trees were rejected for three reasons: shell completion and `ls -R`
degrade quickly past three levels; a pattern that applies to several targets has
to be duplicated or symlinked once depth encodes target; and moving a pattern
between targets then becomes a file move, which pollutes history and breaks
content addressing.

## What is deliberately not in the path

**Lifecycle.** `champion` is a promotion, not a location. Encoding it in the
path would mean every promotion moves a file. It is an attribute, held in the
database and derivable from the `champion` table, so a promotion is a single row
change.

**Quality.** `qor`/`qos`/`qop` change over time and are never overwritten; they
belong in `observation`, not a filename.

**Identity.** A pattern's name is the DU-UUID, a content address. Two manifests
that serialise identically are the same pattern, whatever they are called. Human
readability lives in the `title` field, not the filename, so renaming a pattern
does not change its address. Run `unia-mcp get <du-uuid>` to resolve one.

## Dimensions

| Axis | Where it lives | Queryable |
|---|---|---|
| Resource class | directory | yes |
| Execution target | directory | yes |
| Lifecycle | `pattern.lifecycle` | yes |
| Aliases, learned vs authored | `alias.learned` | yes |
| Universal primitive group | `action.resolved_primitive` | yes |
| Sharing scope | `pattern.sharing` | yes |
| Complexity | `pattern.complexity` | yes |
| Quality, over time | `observation` | yes |
| Lineage | `pattern.parent_du_uuid`, `lineage` | yes |
| Champion | `champion` | yes |

## `_quarantine/`

Ten manifests that were committed to the repository root by an earlier version of
the test suite writing to the process working directory. Their content is
harvester output — `"Pattern [evolve:...]: Behavioral pattern derived from 1
successful interactions"` — not curated knowledge. They are retained here so
nothing is lost and so the decision is auditable, but they are not indexed and
not served. They can be deleted.

Two genuine manifests were rescued from that directory and migrated to
`patterns/actuator/native-linux/`: `smart_valve.ure` and `filesystem.ure`.

## Format note

`docs/ure-specification-formal.md` §3 states the implementation representation
is "JSON-LD/YAML". Every loader in the code uses `serde_json::from_str` and
accepts **JSON only**. The two migrated manifests were originally YAML and had
to be converted to be loadable at all. This is divergence **D3**, and the
correct fix is to decide which way it goes — add a YAML branch to the loader
(`serde_yaml` is already a dependency) or amend the formal specification to say
JSON. It is not resolved here because it is a specification decision, not an
implementation one.
