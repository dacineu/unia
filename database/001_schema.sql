-- unia pattern index — PostgreSQL schema
--
-- Scope: this database is the *index* over the pattern corpus, not the corpus
-- itself. The corpus is the tree of `.ure` files under patterns/, which stays
-- the portable, content-addressed, version-controlled artifact. Postgres exists
-- because the filesystem cannot answer the queries the runtime needs: which
-- patterns apply to this intent, ranked, restricted to this environment, with
-- their measured quality.
--
-- Design rules:
--   1. A pattern's identity is its DU-UUID, never a surrogate key. Surrogate
--      SERIAL ids (as in the previous draft) make deduplication impossible and
--      break the link back to the file.
--   2. Nothing here is authoritative that is not derivable from the file. The
--      database is a rebuildable cache; `unia index --rebuild` must be able to
--      reconstruct all of it from patterns/ alone.
--   3. Quality is measured over time in `observation`, never overwritten in
--      place. A pattern that was promoted and later regressed must show both.
--
-- Apply with:  psql -f database/001_schema.sql

BEGIN;

CREATE EXTENSION IF NOT EXISTS pg_trgm;   -- trigram similarity for fuzzy intent match
CREATE EXTENSION IF NOT EXISTS btree_gin;  -- required for GIN on scalar + array together

-- ---------------------------------------------------------------------------
-- Enumerated dimensions
--
-- These are the axes the corpus is organised along. They are enums rather than
-- free text so that a typo cannot silently create a new dimension value.
-- ---------------------------------------------------------------------------

-- Resource class. The seven values come from the .ure specification section
-- 4.2. WMIS maps these onto its own ResourceType, which is a coarser set
-- (Compute, Storage, Data, Application, AIAssistant, CodeLibrary,
-- ExternalPlatform); the mapping lives in the WMIS adapter, not here.
CREATE TYPE resource_class AS ENUM (
    'identity',   -- who the caller is
    'skill',      -- a capability an agent can invoke
    'config',     -- configuration consumed by other resources
    'job',        -- a unit of scheduled work
    'hardware',   -- a physical or virtual device
    'tool',       -- an external service or binary
    'doc'         -- reference material
);

-- Execution environment. A pattern declares what it needs; the runtime only
-- offers patterns whose target matches the machine it is on. This is the axis
-- that makes "hardware independent" mean something concrete.
CREATE TYPE execution_target AS ENUM (
    'native-linux', 'native-macos', 'native-windows',
    'cuda', 'rocm', 'metal',
    'wasm', 'browser',
    'pure'          -- no environment dependency; runs anywhere
);

-- Lifecycle. Mirrors the harvest -> learn -> transduce -> project -> promote
-- pipeline in src/pipeline. `champion` is a promotion, not a location: it is
-- deliberately not part of any path, because promoting a pattern must not move
-- a file and rewrite history.
CREATE TYPE lifecycle_state AS ENUM (
    'harvested',   -- collected from a trace, not yet verified
    'crystallised',-- transduced to a deterministic .ure, not yet verified
    'candidate',   -- passed property tests, not yet promoted
    'champion',    -- serves traffic; wins ranking ties
    'quarantined', -- failed verification; retained for audit, never served
    'deprecated'   -- superseded, kept for lineage
);

-- The 20 universal primitives, grouped as in src/bridge/primitive.rs. Stored
-- per action so that a pattern can be queried by the primitive it discharges.
CREATE TYPE primitive_group AS ENUM (
    'state_query',      -- GetState, GetValue, CheckSense
    'state_transition', -- SetValue, Toggle, Increment, Reset
    'flow_signal',      -- Route, Pipe, Broadcast
    'temporal_event',   -- Delay, Watch, Pulse
    'compute_logic'     -- Compare, Transform, Validate
);

-- Sharing scope, from src/wmis. Retained because a pattern's visibility is a
-- property of the pattern, not of the file it happens to live in.
CREATE TYPE sharing_scope AS ENUM (
    'circle', 'user', 'team', 'region', 'country', 'continent', 'global'
);

-- ---------------------------------------------------------------------------
-- Core tables
-- ---------------------------------------------------------------------------

-- One row per distinct pattern body. `du_uuid` is the content address produced
-- by src/identifiers, so two manifests that serialise identically collapse to
-- one row and the file may be stored under either path.
CREATE TABLE pattern (
    du_uuid        UUID PRIMARY KEY,
    ure_version    TEXT        NOT NULL,
    resource_class resource_class NOT NULL,
    -- Human-facing name, which need not be unique. Kept out of the identity so
    -- that renaming a pattern does not change its content address.
    title          TEXT        NOT NULL,
    guidance       TEXT        NOT NULL DEFAULT '',
    -- 0.0-1.0, drives dive-depth selection in the orchestrator.
    complexity     REAL        NOT NULL DEFAULT 0.5
                   CHECK (complexity >= 0.0 AND complexity <= 1.0),
    lifecycle      lifecycle_state NOT NULL DEFAULT 'harvested',
    target         execution_target  NOT NULL DEFAULT 'pure',
    sharing        sharing_scope     NOT NULL DEFAULT 'global',
    -- Present only when the pattern can actually be executed locally. A
    -- NULL payload is the honest signal that acting on this pattern still
    -- costs a provider request; the runtime must not assume otherwise.
    payload_kind   TEXT CHECK (payload_kind IS NULL OR payload_kind IN ('wasm','native','script')),
    payload_ref    TEXT,
    payload_entry  TEXT,
    -- Path relative to the corpus root, so a pattern can be located on disk.
    source_path    TEXT,
    -- Lineage, per the knowledge-provenance requirement in docs/cognition-log:
    -- origin URI -> parent UUID -> mutation type.
    origin_uri     TEXT,
    parent_du_uuid UUID REFERENCES pattern(du_uuid),
    mutation_kind  TEXT,
    -- Bumped when the body changes, which yields a new du_uuid and therefore a
    -- new row rather than mutating this one.
    revision       INTEGER     NOT NULL DEFAULT 1,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at     TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT runnable_payload_is_complete CHECK (
        (payload_kind IS NULL AND payload_ref IS NULL)
        OR (payload_kind IS NOT NULL AND payload_ref IS NOT NULL)
    )
);

-- Observable state variables, per spec section 2.2.
CREATE TABLE state_var (
    id          BIGSERIAL PRIMARY KEY,
    du_uuid     UUID NOT NULL REFERENCES pattern(du_uuid) ON DELETE CASCADE,
    name        TEXT NOT NULL,
    type_name   TEXT NOT NULL,                 -- float | string | int | bool | enum
    range_lo    REAL,
    range_hi    REAL,
    unit        TEXT,
    enum_values TEXT[],
    UNIQUE (du_uuid, name)
);

-- Actions, per spec section 2.3. `resolved_primitive` records what the Bridge
-- actually dispatched, which is not always derivable from the id: see
-- docs/SPEC.md divergence D6, where substring resolution makes SetValue the
-- fallback for every unrecognised action.
CREATE TABLE action (
    id                 BIGSERIAL PRIMARY KEY,
    du_uuid            UUID NOT NULL REFERENCES pattern(du_uuid) ON DELETE CASCADE,
    action_id          TEXT NOT NULL,
    target_state       TEXT NOT NULL DEFAULT '',
    resolved_primitive primitive_group,
    -- Preconditions. The formal spec requires these to gate dispatch; the
    -- implementation does not evaluate them yet. Recorded so the gap is
    -- queryable rather than invisible.
    constraints        TEXT[] NOT NULL DEFAULT '{}',
    params             JSONB    NOT NULL DEFAULT '{}'::jsonb,
    UNIQUE (du_uuid, action_id)
);

-- Every phrase a pattern can be found by. This is the retrieval surface, and
-- keeping it normalised means the index does not have to be rebuilt when the
-- text of a single action changes.
CREATE TABLE alias (
    id         BIGSERIAL PRIMARY KEY,
    du_uuid    UUID NOT NULL REFERENCES pattern(du_uuid) ON DELETE CASCADE,
    action_id  TEXT REFERENCES action(action_id) ON DELETE CASCADE,
    phrase     TEXT NOT NULL,
    -- Whether this phrase came from the pattern itself or was learned from
    -- observed phrasings. Learned aliases are the induction output.
    learned    BOOLEAN NOT NULL DEFAULT FALSE
);

-- One harvested execution. This is the training signal: without traces there
-- is nothing to induce an actuator from. The in-process store writes these to
-- traces.jsonl, and `unia index --sync` folds them into this table.
CREATE TABLE trace (
    id           BIGSERIAL PRIMARY KEY,
    ts           TIMESTAMPTZ NOT NULL,
    intent       TEXT        NOT NULL,
    du_uuid      UUID REFERENCES pattern(du_uuid) ON DELETE SET NULL,
    -- hit  = a pattern served the call, no provider request was made
    -- miss = a provider was called and tokens were spent
    outcome      TEXT        NOT NULL CHECK (outcome IN ('hit','miss')),
    tokens_in    BIGINT      NOT NULL DEFAULT 0 CHECK (tokens_in >= 0),
    tokens_out   BIGINT      NOT NULL DEFAULT 0 CHECK (tokens_out >= 0),
    -- The routing decision at the time, retained so champion changes can be
    -- attributed rather than guessed at.
    resolved_by  TEXT
);

-- Quality measured over time. Never overwritten: the three metrics from
-- src/wmis are QualityMetrics, while the README's five-metric (A-E) framing is
-- recorded as unresolved divergence D1 and is not modelled here.
CREATE TABLE observation (
    id       BIGSERIAL PRIMARY KEY,
    du_uuid  UUID NOT NULL REFERENCES pattern(du_uuid) ON DELETE CASCADE,
    ts       TIMESTAMPTZ NOT NULL DEFAULT now(),
    qor      REAL,   -- quality of resource, intrinsic
    qos      REAL,   -- quality of service, real-time behaviour
    qop      REAL,   -- quality of product, output value
    success  BOOLEAN,
    ms       INTEGER
);

-- Promotion record. A capability has at most one champion; promotion is a state
-- change on the pattern plus a row here, never a file move.
CREATE TABLE champion (
    capability  TEXT PRIMARY KEY,
    du_uuid     UUID NOT NULL REFERENCES pattern(du_uuid) ON DELETE CASCADE,
    promoted_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    -- Who or what authorised the promotion. Promotion without evidence is how a
    -- wrong pattern becomes fast, silent and reused.
    promoted_by TEXT NOT NULL DEFAULT 'manual',
    evidence    TEXT
);

-- Provenance edge, kept separate from pattern.parent_du_uuid so a pattern can
-- have more than one derivation and the full derivation graph can be walked.
CREATE TABLE lineage (
    parent_du_uuid UUID NOT NULL REFERENCES pattern(du_uuid) ON DELETE CASCADE,
    child_du_uuid  UUID NOT NULL REFERENCES pattern(du_uuid) ON DELETE CASCADE,
    mutation_kind  TEXT    NOT NULL,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (parent_du_uuid, child_du_uuid, mutation_kind)
);

COMMIT;
