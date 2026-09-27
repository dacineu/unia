# The `.ure` Universal Resource Format

**Specification version:** 0.1.0
**Status:** Draft. Not a standard. Not stable.
**Last updated:** 2026-09-27
**Author:** the unia project (`dacineu/unia`), published under MIT.

This document describes the format **as currently implemented in this
repository**. Where the implementation and older prose in this project disagree,
this document follows the implementation and the disagreement is recorded in
[Known divergences](#known-divergences) rather than papered over. A specification
that describes intended behaviour instead of actual behaviour is worse than no
specification, because implementations will target it and interop will fail.

> **Relationship to the formal specification.** The normative mathematical
> definition of `.ure` is [`ure-specification-formal.md`](./ure-specification-formal.md),
> which defines the Resource Tuple $\langle \mathcal{I}, \mathcal{S}, \mathcal{A}, \mathcal{C} \rangle$
> and the Bridge lifecycle. **That document takes precedence.** This one is a
> conformance record: it states what the code does today, and the divergences
> below are the gaps between the two. Where they conflict, the formal
> specification is the requirement and the code is the defect — except where
> noted, as with D3.

## 1. What the format is for

A `.ure` file describes one addressable resource: a thing an agent can act on
or with. The point of the format is to make that resource **independent of where
it lives and what it physically is**, so the same declaration can be satisfied by
a shell command on Linux, a Wasm module in a browser, or a GPU kernel.

The format is JSON. Not JSON5, not YAML, not TOML. JSON specifically, because
the canonical serialisation must be stable for content addressing (§4.1).

## 2. File structure

A `.ure` file is a single UTF-8 JSON object. The `#` line comments permitted in
some files in this repository are **not valid JSON** and will not parse.

```json
{
  "ure_version": "1.0",
  "resource_id": "valve-001",
  "category": "actuator",

  "state_space": {
    "flow_rate": {
      "type": "float",
      "range": [0.0, 1.0],
      "unit": "percentage"
    },
    "status": {
      "type": "enum",
      "values": ["open", "closed", "fault"]
    }
  },

  "action_primitives": [
    {
      "id": "emergency_shutdown",
      "aliases": ["emergency_shutdown", "emergency shutdown"],
      "params": {},
      "target_state": "flow_rate = 0.0",
      "constraints": ["status != 'fault'"]
    }
  ]
}
```

### 2.1 Top-level fields

| Field | Type | Required | Meaning |
| --- | --- | --- | --- |
| `ure_version` | string | Yes | Version of the *manifest schema*, currently `"1.0"`. Distinct from the identifier scheme version. |
| `resource_id` | string | Yes | Resource identity. Either a `DU-UUID` (§4) or a human-assigned name such as `valve-001`. |
| `category` | string | Yes | Coarse classification, e.g. `actuator`, `skill`, `hardware`, `doc`. |
| `state_space` | object | No | Observable state variables, keyed by name (§2.2). |
| `action_primitives` | array | No | Actions available on the resource (§2.3). |
| `guidance` | string | No | Natural-language description used by the learner and SLM paths. |
| `complexity_score` | number | No | `0.0`–`1.0`. Drives dive-depth selection in the orchestrator. Defaults to `0.5` when absent. |
| `resource_type` | string | No | Used by the WMIS adapter to map onto `ResourceType` (§5). |

Unknown fields **must** be preserved and ignored, so that a manifest written for
a later version remains loadable. Note that §4.1 makes an unknown field change
the identifier, so a consumer that strips unknown fields before hashing will
compute a different ID. Preserve first, hash second.

### 2.2 State variables

Each entry of `state_space` is a `StateType`:

| Field | Type | Meaning |
| --- | --- | --- |
| `type` | string | `float`, `string`, `int`, `bool`, or `enum`. |
| `range` | `[number, number]` | Inclusive bounds. Float and int only. |
| `unit` | string | Display unit, e.g. `percentage`. |
| `values` | array of string | Permitted values. Enum only. |

Exactly one of `range` or `values` is meaningful per variable. Supplying both is
undefined.

### 2.3 Actions

Each entry of `action_primitives` is a `UreAction`:

| Field | Type | Required | Meaning |
| --- | --- | --- | --- |
| `id` | string | Yes | Action identity. Primary mapping key. |
| `aliases` | array of string or `null` | Yes (may be `null`) | Alternate phrasings resolved during intent matching (§3.2). |
| `params` | object of string→string | Yes | Declared parameters. |
| `target_state` | string | Yes | State the action drives towards. |
| `constraints` | array of string | Yes | Preconditions. Currently **evaluated for effect only** — see Known divergences. |

`aliases` is a required field in the serialised form and may be `null`. Omitting
it is an error, not a default. This is an ergonomic wart: authors of new
manifests must write `"aliases": null` for an action with no aliases.

## 3. Intent resolution

### 3.1 The separation

Intent resolution lives in the **Primitive Bridge** (`src/bridge/primitive.rs`).
It turns natural-language intent plus a resource identity into a
`PrimitivePacket`. It never touches hardware. The **Actuator Nucleus**
(`src/nucleus`) takes packets and drives actuators. Neither imports the other.

### 3.2 Matching algorithm

`PrimitiveBridge::map_intent(resource_id, intent)` lowercases the intent, then
for each action in declaration order:

1. **Normalised containment.** If the lowercased intent contains the action `id`
   with `_` replaced by spaces, return that action.
2. **Alias containment.** If the lowercased intent contains any alias,
   lowercased, return that action.
3. Otherwise score the action with `SemanticMapper::compute_score` and keep the
   best.

**Consequence for authors:** because step 1 normalises underscores to spaces but
step 2 does not, an action whose `id` is `emergency_shutdown` matches the intent
`"emergency shutdown the valve"` but **not** `"perform an emergency_shutdown"`.
To serve both phrasings, list both in `aliases`. This asymmetry is a known
defect (Known divergences, D4) and is a compatibility hazard for independent
implementations.

### 3.3 Universal primitives

Resolution maps an action to a `UniversalPrimitive`. The mapping is currently
substring-based on the action `id`:

| `id` contains | Primitive |
| --- | --- |
| `shutdown` or `off` | `Reset` |
| `adjust` or `set` | `SetValue` |
| `check` or `get` | `GetValue` |
| anything else | `SetValue` |

The full vocabulary, which an implementation should support regardless of how it
currently resolves, is:

| Group | Primitives |
| --- | --- |
| State query | `GetState`, `GetValue`, `CheckSense` |
| State transition | `SetValue`, `Toggle`, `Increment`, `Reset` |
| Flow and signal | `Route`, `Pipe`, `Broadcast` |
| Temporal and event | `Delay`, `Watch`, `Pulse` |
| Compute and logic | `Compare`, `Transform`, `Validate` |

### 3.4 Packet structure

```rust
PrimitivePacket {
    header:  PacketHeader { timestamp, request_id, priority },
    payload: PacketPayload { primitive, resource_id, arguments },
    context: PacketContext { expected_state, timeout_ms },
}
```

`priority` is one of `Low`, `Medium`, `High`, `Critical`.

## 4. Identity

Two schemes exist in this project. They are not interchangeable, and the
project is currently ambiguous about which is normative.

### 4.1 DU-UUID (content-derived, implemented)

`DuUuid::generate(manifest, encryption_key)` in `src/identifiers`:

1. Clone the manifest and **remove `resource_id`**, so an identifier never
   contributes to its own value.
2. Serialise the remainder to JSON. `serde_json` is not built with
   `preserve_order`, so object keys are sorted by a `BTreeMap` and the
   serialisation is byte-stable for a given value. A comment in the source
   claims this is not guaranteed; that comment is wrong and should be deleted.
3. If a 32-byte key is supplied, encrypt with AES-256-GCM.
4. SHA-256 the result.
5. Take the first 16 bytes.
6. Force the UUID version field to 4 and the variant to RFC 4122.

Properties: the same manifest content yields the same UUID, and any content
change yields a different UUID. Both are covered by tests.

Two problems with this scheme, both of which an independent implementation will
hit:

- **The version field lies.** Step 6 labels a content-derived identifier as
  version 4, which RFC 4122 defines as *random*. A content-derived identifier
  should be version 5, or version 8 for a custom scheme. Consumers that
  legitimately treat v4 as random will mishandle these.
- **The AES-GCM nonce is fixed.** `encrypt_payload` uses the constant
  `b"ure_det_nonc"`. This is what makes the operation deterministic, and it is
  acceptable for producing a stable identifier, but GCM with a reused nonce leaks
  the XOR of two plaintexts and does not provide semantic security. **Do not use
  the encrypted form for confidentiality.** See [`SECURITY.md`](../SECURITY.md).

### 4.2 `ure_RES_CAT_LOC_UNIQ` (documented, not implemented)

The project also documents an identifier scheme spelled
`ure_RES_CAT_LOC_UNIQ` over UUID v4, with seven resource types: `identity`,
`skill`, `config`, `job`, `hardware`, `tool`, `doc`.

This is not implemented anywhere in the code. It is retained here because it is
the intended direction, and because the seven-type vocabulary is a useful and
adoptable part of the design. It is **not** normative in version 0.1.0.

## 5. WMIS resource mapping

`WmisAdapter::from_ure` maps a manifest onto a `WmisResource`, defaulting
`owner` to the caller:

| Manifest `resource_type` | `ResourceType` |
| --- | --- |
| `skill`, `mirror_actuator` | `CodeLibrary` |
| `ai_assistant` | `AIAssistant` |
| anything else | `Application` |

`WmisResource` carries `id`, `resource_type`, `owner`, `sharing_scope`, one of
`Circle`/`User`/`Team`/`Region`/`Country`/`Continent`/`Global`, a `capabilities`
list, a `QualityMetrics` triple, and free-form `metadata`.

`QualityMetrics` has three fields, not five:

| Field | Meaning |
| --- | --- |
| `qor` | Quality of Resource, intrinsic |
| `qos` | Quality of Service, real-time behaviour |
| `qop` | Quality of Product, output value |

Actuation is metered: `WmisEconomicLayer::charge_actuation` debits the calling
principal's token balance and refuses the actuation if the balance is
insufficient. A principal is granted a 100-token balance on first use.

## 6. Registry resolution

`ActuatorRegistry` resolves manifests by `{resource_id}.ure` inside a base
directory.

- `ActuatorRegistry::new(conn)` uses the process working directory.
- `ActuatorRegistry::with_base_dir(conn, dir)` uses an explicit directory.

`find_matching_actuators` scans the same base directory. Both should be
configured explicitly outside tests; depending on the working directory makes a
programme behave differently depending on where it was launched from.

## 7. Known divergences

Recorded rather than fixed, because each needs a design decision.

| # | Divergence | Impact |
| --- | --- | --- |
| D1 | README describes five quality metrics (A–E); the code has three (`qor`, `qos`, `qop`). | A manifest author targeting the README would produce a file the code ignores. |
| D2 | README specifies `ure_RES_CAT_LOC_UNIQ` with random UUID v4; the code implements content-derived DU-UUID. | Two incompatible identity schemes documented as one. |
| D3 | The formal specification permits "JSON-LD/YAML" as the implementation representation (§3). Every loader uses `serde_json::from_str` and accepts JSON only. | The two example manifests were YAML and could not be loaded by any code path. They have been converted to JSON, which resolves the immediate failure but leaves the spec and code disagreeing. Fix by adding a YAML branch to the loader (`serde_yaml` is already a dependency) or by amending the formal specification. |
| D8 | `category` is a free-text functional class (`actuator`, `sensor`, `memory`, `compute`) while the seven values in §4.2 are a different, non-overlapping vocabulary. The corpus directory uses `category`; the database schema uses `resource_class`. | Two axes are in use for the same dimension. `patterns/actuator/` is not one of the seven `resource_class` values. |
| D9 | The formal specification's packet is `{ resource_id, primitive, params }` where `primitive` is the action name. The code resolves it to a generic `UniversalPrimitive` (see D6). | The Nucleus receives a coarser instruction than the specification describes, and loses the action identity on the way. |
| ~~D4~~ **CLOSED** | Intent matching normalised underscores for action `id` but not for `aliases`. **Fixed:** `tokenize` in `src/mcp/store.rs` now splits on any non-alphanumeric character, so `write_file` and `write file` produce the same terms. | Was: an intent using underscores silently fell through to score-based matching, and `write output to results.txt` matched nothing at all. Now consistent with the underscore normalisation in `src/bridge/primitive.rs` and `src/induce/mod.rs`. |
| D5 | Action `constraints` are printed for operator visibility but not evaluated. | A manifest can declare a precondition that the system ignores. |
| D6 | `resolve_primitive` resolves by substring on the action `id`, so `SetValue` is the fallback for every unrecognised action. | Most of the §3.3 vocabulary is unreachable through the bridge today. |
| D7 | `quality_of_product` appears as a manifest key in tests, but `QualityMetrics` is constructed in code rather than read from the manifest. | The mapping from manifest to metrics is not implemented. |

## 8. Contributing to this specification

The specification is the asset, not the code. If you are implementing `.ure`
independently, this document plus the MIT licence is all you need.

Corrections to the specification are welcome as pull requests, particularly for
§7. If you find that the format does not work for your use case, say so on an
issue before implementing around it — that is the cheapest possible time to
change the format, and it will not get cheaper.

A change to this document is a change to the project. Bump the version, record
the date, and keep the divergences table.
