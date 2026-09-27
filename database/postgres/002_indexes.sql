-- unia pattern index — retrieval indexes
--
-- The reason this file exists: Store::search in src/mcp/store.rs scores every
-- pattern in the corpus on every query. Measured on this machine at roughly
-- 0.5 microseconds per pattern, which is fine at 12 manifests and 10
-- milliseconds at 20,000. Retrieval, not induction, is the component that
-- breaks first — the same conclusion the agent-skills literature reaches about
-- graph-structured skill libraries at scale.
--
-- These indexes replace that scan with a GIN lookup. Apply after 001_schema.sql.

BEGIN;

-- Trigram indexes are conditional on pg_trgm, which 001_schema.sql probes.
-- Everything else in this file uses core features and is always created, so a
-- host without contrib still gets the GIN full-text path and the whole
-- dimension index set.
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM unia_feature WHERE name = 'pg_trgm' AND available) THEN
        CREATE INDEX pattern_title_trgm ON pattern USING GIN (title gin_trgm_ops);
        CREATE INDEX pattern_guidance_trgm ON pattern USING GIN (guidance gin_trgm_ops);
        CREATE INDEX alias_phrase_trgm ON alias USING GIN (phrase gin_trgm_ops);
        CREATE INDEX trace_intent_trgm ON trace USING GIN (intent gin_trgm_ops);
        RAISE NOTICE 'trigram indexes created';
    ELSE
        RAISE NOTICE 'pg_trgm absent: skipping 4 trigram indexes';
    END IF;
END
$$;

-- ---------------------------------------------------------------------------
-- Full-text search over the retrieval surface.
--
-- The tsvector is built from guidance plus every alias, so a single query
-- covers both a description match and a phrasing match. It is stored rather
-- than generated because a generated column would be recomputed on every write
-- to alias, and alias is the highest-churn table in the schema.
-- ---------------------------------------------------------------------------

ALTER TABLE pattern ADD COLUMN search_tsv tsvector;
ALTER TABLE action   ADD COLUMN search_tsv tsvector;

CREATE FUNCTION pattern_search_tsv() RETURNS trigger AS $$
BEGIN
    NEW.search_tsv :=
        setweight(to_tsvector('english', coalesce(NEW.guidance, '')), 'A') ||
        setweight(to_tsvector('english', coalesce(NEW.title, '')),    'B');
    RETURN NEW;
END
$$ LANGUAGE plpgsql;

CREATE TRIGGER pattern_tsv BEFORE INSERT OR UPDATE OF guidance, title
    ON pattern FOR EACH ROW EXECUTE FUNCTION pattern_search_tsv();

-- Alias text is folded in by the indexer rather than a trigger, because it
-- needs a join back to pattern. See database/003_views.sql.
CREATE INDEX pattern_tsv_idx ON pattern USING GIN (search_tsv);

-- Trigram on title and guidance. Intent never matches a stored phrase
-- exactly — "add a rust dependency to Cargo.toml and update the lockfile"
-- contains no substring of the pattern title — which is why the Bridge's
-- containment check (docs/SPEC.md divergence D4) falls through to scoring far
-- more often than it should. Conditional on pg_trgm; see the DO block above.
CREATE INDEX alias_phrase_lower ON alias (lower(phrase));
CREATE INDEX alias_du_uuid ON alias (du_uuid);

-- ---------------------------------------------------------------------------
-- Dimension indexes: one per axis the corpus is organised along.
-- ---------------------------------------------------------------------------

CREATE INDEX pattern_class_lifecycle ON pattern (resource_class, lifecycle);
CREATE INDEX pattern_target_lifecycle ON pattern (target, lifecycle);
CREATE INDEX pattern_lifecycle ON pattern (lifecycle);
CREATE INDEX pattern_class_target ON pattern (resource_class, target);
CREATE INDEX pattern_complexity ON pattern (complexity);

-- Partial index for the only query the hot path actually runs: find
-- runnable, live patterns. Serves the three-tier ladder's first and second rungs
-- and excludes quarantined, deprecated and non-runnable rows entirely.
CREATE INDEX pattern_runnable_live ON pattern (resource_class, target)
    WHERE lifecycle IN ('candidate', 'champion') AND payload_kind IS NOT NULL;

-- Actions
CREATE INDEX action_du_uuid ON action (du_uuid);
CREATE INDEX action_primitive ON action (resolved_primitive);
CREATE INDEX action_constraints_gin ON action USING GIN (constraints);

-- State variables
CREATE INDEX state_var_du_uuid ON state_var (du_uuid);

-- ---------------------------------------------------------------------------
-- Trace and observation indexes.
--
-- token totals are what the promotion decision is argued from, so they get a
-- covering index rather than a plain one.
-- ---------------------------------------------------------------------------

CREATE INDEX trace_ts ON trace (ts DESC);
CREATE INDEX trace_outcome_ts ON trace (outcome, ts DESC);
CREATE INDEX trace_du_uuid ON trace (du_uuid);
-- An expression index: Postgres does not accept a bare expression in an index
-- column list, so the total must be wrapped. This is the index the promotion
-- decision is argued from, so it is covering rather than plain.
CREATE INDEX trace_token_totals ON trace (outcome, ((tokens_in + tokens_out)));

CREATE INDEX observation_du_uuid_ts ON observation (du_uuid, ts DESC);

CREATE INDEX lineage_child ON lineage (child_du_uuid);

-- ---------------------------------------------------------------------------
-- Capability lookup for promotion.
-- ---------------------------------------------------------------------------

CREATE INDEX champion_du_uuid ON champion (du_uuid);

COMMIT;
