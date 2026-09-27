-- unia pattern index — DuckDB retrieval
--
-- Replaces the linear scan in src/mcp/store.rs and the SQL in
-- database/postgres/003_retrieval.sql.
--
-- Why the ranking looks the way it does. The measured cost of the in-process
-- scan is about 0.5 microseconds per pattern, which is 4.6 us at 12 manifests
-- and about 10 ms at 20,000. Retrieval, not induction, is the component that
-- breaks first.
--
-- DuckDB is columnar and vectorised, so even a full scan of the flattened
-- phrase view is fast enough that a separate inverted index is not yet
-- justified at this corpus size. That is a measurement-driven choice, not a
-- claim that indexing is unnecessary: if the corpus passes roughly 10^5
-- phrases, add one.
--
-- Trigram similarity is deliberately not used for scoring. It ignores term
-- frequency, so a rare token scores the same as a common one, which is worse
-- than inverse-document-frequency overlap for short technical intents where one
-- distinctive word carries most of the signal.

-- ===========================================================================
-- Corpus statistics
-- ===========================================================================

-- Document frequency per term, for inverse-document-frequency weighting.
--
-- Computed on the fly rather than stored: IDF must reflect the corpus as it is
-- now, and a stale statistic would silently distort every ranking. At this
-- corpus size the recomputation is cheaper than maintaining it correctly.
CREATE OR REPLACE VIEW term_idf AS
SELECT
    term,
    count(DISTINCT du_uuid) AS df,
    ln( ((SELECT count(DISTINCT du_uuid) FROM pattern) - count(DISTINCT du_uuid) + 0.5)
        / (count(DISTINCT du_uuid) + 0.5) + 1.0 ) AS idf
FROM pattern_phrase,
     LATERAL (SELECT unnest(regexp_split_to_array(phrase_lower, '[^a-z0-9_]+')) AS term)
WHERE term <> '' AND length(term) > 1
GROUP BY term;


-- ===========================================================================
-- Retrieval
-- ===========================================================================

-- The retrieval surface as a scored relation, so the hot path is a plain query
-- and the scoring logic is inspectable on its own.
--
-- Two matching paths, in priority order:
--
--   containment  An intent that contains a stored phrase, or a stored phrase
--                that equals the intent, with underscores normalised to spaces.
--                This mirrors the Bridge's fast path and is decisive when it
--                fires, because it means the caller phrased the request the way
--                the pattern was authored.
--
--   overlap      IDF-weighted token overlap, scored symmetrically so a long
--                intent is not penalised against a short phrase. This is the
--                case that matters in practice: "add a rust dependency to
--                Cargo.toml and update the lockfile" contains no substring of
--                any stored title, so containment never fires and everything
--                falls through here.
-- A macro, not a view: a view cannot take a parameter, and DuckDB rejects a
-- prepared placeholder that is not in the final statement of a multi-statement
-- execute. A macro is textual substitution, so the intent flows through cleanly.
CREATE OR REPLACE MACRO phrase_score(p_intent) AS TABLE
WITH params AS (
    SELECT lower(trim(p_intent)) AS intent
),
tokens AS (
    SELECT t.term
    FROM params,
         LATERAL (SELECT unnest(regexp_split_to_array(params.intent, '[^a-z0-9_]+')) AS term) t
    WHERE t.term <> '' AND length(t.term) > 1
),
-- The intent is carried onto every row rather than referenced from a CTE inside
-- the CASE: GROUP BY may only name columns present in the FROM, so a reference
-- to params.intent here is unresolvable.
rows AS (
    SELECT
        pp.du_uuid,
        pp.phrase,
        pp.phrase_lower,
        replace(pp.phrase_lower, '_', ' ') AS phrase_norm,
        pa.intent,
        pt.term
    FROM pattern_phrase pp
    CROSS JOIN params pa,
         LATERAL (
             SELECT unnest(regexp_split_to_array(pp.phrase_lower, '[^a-z0-9_]+')) AS term
         ) pt
    WHERE pt.term <> '' AND length(pt.term) > 1
)
SELECT
    r.du_uuid,
    r.phrase,
    r.phrase_lower,
    CASE
        WHEN r.phrase_norm = r.intent THEN 1.00
        WHEN length(r.phrase_norm) > 2
             AND r.intent LIKE '%' || r.phrase_norm || '%' THEN 0.85
        ELSE 0.75 * (
                 COALESCE(sum(COALESCE(i.idf, 0)) FILTER (WHERE tk.term IS NOT NULL), 0)
                 / NULLIF(sum(COALESCE(i.idf, 0)), 0)
             )
             + 0.25 * (
                 count(*) FILTER (WHERE tk.term IS NOT NULL)::DOUBLE
                 / NULLIF(count(*), 0)
             )
    END AS score
FROM rows r
LEFT JOIN term_idf i ON i.term = r.term
LEFT JOIN tokens tk  ON tk.term = r.term
GROUP BY r.du_uuid, r.phrase, r.phrase_lower, r.phrase_norm, r.intent;


-- Rank patterns for an intent.
--
-- Champion handling is deliberately restrictive: a champion is served only when
-- it is the best match for the intent and clears a higher bar than the general
-- floor. Joining the champion table to everything above the floor is the bug
-- that made a valve champion answer "rewrite the docstring parser in rust",
-- which is exactly the failure where a promoted pattern becomes fast, silent
-- and wrong.
CREATE OR REPLACE MACRO pattern_search(intent, p_limit := 5, p_min_score := 0.15) AS TABLE
WITH scored AS (
    SELECT du_uuid, phrase AS matched_on, max(score) AS score
    FROM phrase_score(intent)
    WHERE score >= p_min_score
    GROUP BY du_uuid, phrase
),
best AS (SELECT max(score) AS top FROM scored),
champ AS (
    SELECT s.du_uuid, s.score
    FROM scored s
    JOIN champion c ON c.du_uuid = s.du_uuid
    CROSS JOIN best
    WHERE s.score >= 0.5 AND s.score = best.top
)
SELECT
    p.du_uuid,
    p.resource_id,
    p.resource_class,
    p.target,
    p.lifecycle,
    p.payload_kind,
    (p.payload_kind IS NOT NULL) AS saves_tokens,
    LEAST(1.0, s.score * CASE WHEN ch.du_uuid IS NOT NULL THEN 1.1 ELSE 1.0 END) AS score,
    s.matched_on
FROM scored s
JOIN pattern p USING (du_uuid)
LEFT JOIN champ ch ON ch.du_uuid = s.du_uuid
ORDER BY score DESC, p.du_uuid
LIMIT p_limit;


-- The three-tier routing ladder, in one place so the decision is auditable.
--
--   tier 1  champion that is the best match  -> serve locally, 0 tokens
--   tier 2  any runnable pattern             -> serve locally, 0 tokens
--   tier 3  nothing runnable                -> escalate to a model
--
-- `saves_tokens` is false whenever a pattern has no payload, because a
-- declaration with no executable body re-describes the problem and still costs a
-- provider request. Reporting it as true would be the whole failure this
-- project is meant to avoid.
CREATE OR REPLACE MACRO pattern_resolve(intent) AS TABLE
WITH scored AS (
    SELECT du_uuid, phrase AS matched_on, max(score) AS score
    FROM phrase_score(intent)
    WHERE score >= 0.15
    GROUP BY du_uuid, phrase
),
top AS (SELECT max(score) AS s FROM scored),
champ AS (
    SELECT sc.du_uuid, sc.score
    FROM scored sc JOIN champion c ON c.du_uuid = sc.du_uuid
    CROSS JOIN top
    WHERE sc.score >= 0.5 AND sc.score = top.s
),
runnable AS (
    SELECT sc.du_uuid, sc.score FROM scored sc
    JOIN pattern p USING (du_uuid)
    WHERE p.payload_kind IS NOT NULL
    ORDER BY sc.score DESC LIMIT 1
),
any_best AS (SELECT du_uuid, score FROM scored ORDER BY score DESC LIMIT 1)
SELECT
    CASE
        WHEN (SELECT count(*) FROM champ) > 0 THEN 1
        WHEN (SELECT count(*) FROM runnable) > 0 THEN 2
        ELSE 3
    END                                                          AS tier,
    COALESCE((SELECT du_uuid FROM champ),
             (SELECT du_uuid FROM runnable),
             (SELECT du_uuid FROM any_best))                    AS du_uuid,
    COALESCE((SELECT reason FROM (
                 SELECT 'champion' AS reason, 1 AS o
                 UNION ALL SELECT 'runnable-pattern', 2
                 UNION ALL SELECT 'escalate', 3) WHERE
                 CASE
                     WHEN (SELECT count(*) FROM champ) > 0 THEN 1
                     WHEN (SELECT count(*) FROM runnable) > 0 THEN 2
                     ELSE 3
                 END = o), 'escalate')                          AS reason;


-- Token accounting per capability, which is the only evidence that can argue a
-- promotion was worth making.
CREATE OR REPLACE VIEW capability_token_savings AS
SELECT
    ch.capability,
    ch.du_uuid,
    p.resource_id,
    count(*) FILTER (WHERE t.outcome = 'hit')      AS served_locally,
    count(*) FILTER (WHERE t.outcome = 'miss')     AS escalated,
    COALESCE(sum(t.tokens_in + t.tokens_out)
             FILTER (WHERE t.outcome = 'hit'), 0)  AS tokens_saved,
    max(t.ts)                                       AS last_served
FROM champion ch
JOIN pattern p USING (du_uuid)
LEFT JOIN trace t ON t.du_uuid = ch.du_uuid
GROUP BY ch.capability, ch.du_uuid, p.resource_id;
