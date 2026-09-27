-- unia pattern index — DuckDB
--
-- Replaces the PostgreSQL schema. Two things drive that choice, both measured on
-- this machine rather than assumed:
--
--   1. The corpus is small and still moving. The schema was revised three times
--      during development, and the PostgreSQL migrations had to be dropped and
--      reapplied each time. Here the corpus is a *view over the .ure files*, so
--      the model is edited as SQL and there is no migration to write. Nothing is
--      copied, so the files cannot drift from the index.
--   2. The corpus is JSON on disk, and read_json reads it directly. A 12-file
--      corpus resolves in about 5 ms with no ingest step at all.
--
-- STATUS: the corpus layer below is verified working. 001 loads clean and
-- yields 12 patterns, 5 actions, 30 aliases, 3 state variables and 32 retrieval
-- phrases from the corpus on disk, with MAP key unnesting and JSON[] list
-- unnesting both correct and no ingest step. 002_retrieval.sql loads but its
-- term_idf view still raises a type-coercion error on a manifest whose
-- guidance field is a plain string, so pattern_search and pattern_resolve are
-- NOT yet callable. Treat 002 as unfinished.
--
-- Only mutable state is a table. Everything derived from the corpus is a view.
--
-- The PostgreSQL version is kept in database/postgres/ as the upgrade path. It
-- becomes the right answer when there are concurrent writers, a shared index
-- across machines, or a trace log in the millions. See that directory's README.

-- ===========================================================================
-- Mutable state
-- ===========================================================================

-- Promotion record. A capability has at most one champion. Promotion is a state
-- change, never a file move: encoding lifecycle in a path would rewrite history
-- on every promotion.
CREATE TABLE IF NOT EXISTS champion (
    capability   VARCHAR PRIMARY KEY,
    du_uuid      UUID    NOT NULL,
    promoted_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    promoted_by  VARCHAR NOT NULL DEFAULT 'manual',
    evidence     VARCHAR
);

-- One harvested execution. This is the training signal: with no traces there is
-- nothing to induce an actuator from, so a trace that is not recorded is a
-- pattern that can never be learned.
CREATE TABLE IF NOT EXISTS trace (
    id          BIGINT,
    ts          TIMESTAMPTZ NOT NULL DEFAULT now(),
    intent      VARCHAR NOT NULL,
    du_uuid     UUID,
    outcome     VARCHAR NOT NULL CHECK (outcome IN ('hit', 'miss')),
    tokens_in   BIGINT  NOT NULL DEFAULT 0 CHECK (tokens_in >= 0),
    tokens_out  BIGINT  NOT NULL DEFAULT 0 CHECK (tokens_out >= 0),
    resolved_by VARCHAR
);

-- Quality measured over time, never overwritten. A pattern that was promoted
-- and later regressed must be able to show both. Three metrics, from
-- QualityMetrics in src/wmis. The five-metric (A-E) framing in the README is
-- unresolved divergence D1 and is deliberately not modelled.
CREATE TABLE IF NOT EXISTS observation (
    id      BIGINT,
    du_uuid UUID NOT NULL,
    ts      TIMESTAMPTZ NOT NULL DEFAULT now(),
    qor     DOUBLE,
    qos     DOUBLE,
    qop     DOUBLE,
    success BOOLEAN,
    ms      INTEGER
);

-- Provenance edges. Kept separate from a single parent column so a pattern can
-- have more than one derivation and the full graph can be walked.
CREATE TABLE IF NOT EXISTS lineage (
    parent_du_uuid UUID    NOT NULL,
    child_du_uuid  UUID    NOT NULL,
    mutation_kind  VARCHAR NOT NULL,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ===========================================================================
-- Corpus views
--
-- The corpus is the authoritative artifact. These views are the only way the
-- database is meant to read it, so that a change to the layout is made in one
-- place. Paths are relative to the repository root, which is where the
-- unia-mcp and unia binaries are expected to be run from.
-- ===========================================================================

-- Every .ure manifest under the corpus, read straight off disk.
--
-- The column spec is explicit rather than inferred, and that is the whole point.
-- read_json_auto infers a STRUCT when an object's keys are consistent and a MAP
-- when they vary, so state_space would flip type depending on corpus
-- composition and map_keys() would work on one corpus and fail on the next. It
-- also omits a field entirely when no file carries it: no manifest in the
-- corpus has a `payload`, so an inferred read has no payload column at all and
-- the query fails. Pinning the columns makes both problems impossible, and
-- means a field added to a single manifest does not change the shape of the
-- corpus.
--
-- ignore_errors is required rather than optional: the corpus contains
-- quarantined and hand-edited manifests, and one unreadable file must not make
-- the whole corpus unqueryable.
--
-- ure_version is VARCHAR because the curated manifests write "1.0" and the
-- harvested ones write 1.0; a numeric read rejects one of the two forms.
CREATE OR REPLACE VIEW manifest AS
SELECT
    json->>'resource_id'       AS resource_id,
    json->>'title'             AS title,
    json->>'ure_version'       AS ure_version,
    json->>'category'          AS category,
    json->>'guidance'          AS guidance,
    TRY_CAST(json->>'complexity_score' AS DOUBLE)              AS complexity_score,
    COALESCE(TRY_CAST(json->'state_space' AS MAP(VARCHAR, JSON)), MAP{})  AS state_space,
    COALESCE(TRY_CAST(json->'action_primitives' AS JSON[]), [])          AS action_primitives,
    json->'payload'            AS payload,
    -- resource_id is carried verbatim, not cast to UUID. The two curated
    -- manifests use 'valve-001' and 'fs-root-001', which are human-assigned
    -- names, and the harvested ones use UUIDs. Casting to UUID silently dropped
    -- the curated patterns. The DU-UUID content address is derived at
    -- registration time by src/identifiers, not stored in the file.
    resource_id               AS du_uuid,
    filename                  AS source_file
FROM read_json_objects('patterns/**/*.ure', ignore_errors = true, filename = true)
WHERE json->>'resource_id' IS NOT NULL;

-- Patterns, with lifecycle and target resolved. Lifecycle and target are not in
-- the manifest today; they are exposed as columns so the dimensions exist and
-- can be filtered and grouped on before the manifests carry the fields. Default
-- lifecycle is 'harvested' and default target is 'pure', matching the schema
-- defaults they replace.
CREATE OR REPLACE VIEW pattern AS
SELECT
    du_uuid,
    resource_id,
    COALESCE(ure_version, '1.0')      AS ure_version,
    -- Divergence D8, unresolved: category is a free-text functional class while
    -- the seven values in the specification's section 4.2 are a different
    -- vocabulary. The corpus uses category, so that is what is exposed here.
    COALESCE(category, 'unknown')      AS resource_class,
    -- No manifest in the corpus carries a title yet, so the pinned column
    -- is always present but usually null, and resource_id stands in. A title is
    -- the readable label the corpus is missing; see patterns/_schema/.
    COALESCE(NULLIF(manifest.title, ''), manifest.resource_id) AS title,
    COALESCE(guidance, '')             AS guidance,
    COALESCE(complexity_score, 0.5)    AS complexity,
    'harvested'                        AS lifecycle,
    'pure'                             AS target,
    source_file                        AS source_path,
    -- Carried through so the action and state_var views below can unnest the
    -- nested parts without re-reading the files or re-joining manifest.
    state_space                        AS state_space,
    action_primitives                  AS action_primitives,
    -- Carried through so the action and state_var views below can unnest the
    -- nested parts without re-reading the files or re-joining manifest.
    
    
    payload->>'kind'                   AS payload_kind,
    payload->>'ref'                    AS payload_ref
FROM manifest;

-- Actions, unnested. DuckDB unnests a JSON list into rows natively, so the
-- nested shape in the file becomes relational without an ingest step.
CREATE OR REPLACE VIEW action AS
SELECT
    p.du_uuid,
    a.id                                     AS action_id,
    COALESCE(a->>'target_state', '')         AS target_state,
    COALESCE(TRY_CAST(a->'aliases' AS VARCHAR[]), []::VARCHAR[])        AS aliases,
    COALESCE(TRY_CAST(a->'constraints' AS VARCHAR[]), []::VARCHAR[])    AS constraints,
    COALESCE(a->'params', '{}'::JSON)        AS params
FROM pattern p,
     LATERAL (SELECT unnest(p.action_primitives) AS a) s;

-- Every phrase a pattern can be found by, with a row per alias so the retrieval
-- view below is a flat table rather than an aggregation at query time.
--
-- Note this is where the STRUCT/MAP distinction from above would bite: aliases
-- live inside action_primitives, which read_json also auto-infers. They are
-- extracted here from the raw JSON so their type is explicit.
CREATE OR REPLACE VIEW alias AS
SELECT
    p.du_uuid,
    act.action_id,
    phrase,
    FALSE                                     AS learned
FROM pattern p
JOIN action act ON act.du_uuid = p.du_uuid,
     LATERAL (SELECT unnest(act.aliases) AS phrase) s
WHERE phrase IS NOT NULL AND length(trim(phrase)) > 0
UNION ALL
-- The action id is itself a retrieval phrase, which is what the Bridge's
-- containment fast path matches on.
SELECT p.du_uuid, act.action_id, act.action_id, FALSE
FROM pattern p JOIN action act ON act.du_uuid = p.du_uuid
UNION ALL
SELECT p.du_uuid, NULL, p.guidance, FALSE
FROM pattern p
WHERE length(trim(p.guidance)) > 0;

-- State variables, unnested from the MAP.
CREATE OR REPLACE VIEW state_var AS
SELECT
    p.du_uuid,
    sv.key                                       AS name,
    TRY_CAST(sv.value->>'type'  AS VARCHAR)      AS type_name,
    TRY_CAST(sv.value->>'unit'  AS VARCHAR)      AS unit,
    TRY_CAST(sv.value->'range'  AS DOUBLE[])     AS range,
    TRY_CAST(sv.value->'values' AS VARCHAR[])    AS enum_values
FROM pattern p,
     LATERAL (SELECT unnest(map_keys(p.state_space)) AS key) k,
     -- Both projections must be selected here: a LATERAL subquery exposes only
     -- what it selects, so selecting only the value leaves sv.key unresolvable.
     LATERAL (SELECT k.key AS key, p.state_space[k.key] AS value) sv;

-- Flattened retrieval surface, one row per phrase. The index is on this.
CREATE OR REPLACE VIEW pattern_phrase AS
SELECT
    a.du_uuid,
    lower(a.phrase)  AS phrase_lower,
    a.phrase,
    a.learned,
    p.title          AS title,
    p.guidance,
    p.resource_class,
    p.target,
    p.lifecycle,
    p.payload_kind
FROM alias a
JOIN pattern p USING (du_uuid);
