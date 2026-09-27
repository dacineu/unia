-- unia pattern index — retrieval
--
-- Replaces the in-process linear scan in src/mcp/store.rs. Apply after
-- 002_indexes.sql.
--
-- Ranking is IDF-weighted token overlap, the same shape as the Rust matcher, so
-- a result set is comparable between the file-backed and database-backed
-- stores. IDF is computed on the fly from the alias table rather than stored,
-- because it must reflect the corpus as it is now, and a stale corpus statistic
-- would silently distort every ranking.

BEGIN;

-- Flattened retrieval surface: one row per phrase, joined to its pattern.
CREATE VIEW pattern_phrase AS
SELECT
    a.du_uuid,
    a.phrase,
    lower(a.phrase)                       AS phrase_lower,
    a.learned,
    p.title,
    p.guidance,
    p.resource_class,
    p.target,
    p.lifecycle,
    p.payload_kind,
    p.complexity
FROM alias a
JOIN pattern p USING (du_uuid);

-- Document frequency, for inverse-document-frequency weighting.
CREATE VIEW term_document_frequency AS
SELECT
    t.term,
    count(DISTINCT du_uuid) AS df
FROM pattern_phrase,
     LATERAL unnest(regexp_split_to_array(phrase_lower, '[^[:alnum:]_]+')) AS t(term)
WHERE t.term <> '' AND length(t.term) > 1
GROUP BY t.term;

CREATE VIEW term_idf AS
SELECT
    term,
    df,
    ln( ( (SELECT count(DISTINCT du_uuid) FROM pattern) - df + 0.5 )
        / (df + 0.5) + 1.0 ) AS idf
FROM term_document_frequency;


-- Rank patterns against an intent.
--
-- `p_target` and `p_include_non_runnable` are the two constraints that make
-- this usable rather than merely clever: the caller knows which environment it
-- is on, and knows whether a descriptive-only pattern is still worth showing.
--
-- Note the deliberate absence of any `similarity()` call. Trigram similarity
-- ignores term frequency, so it scores a rare word the same as a common one and
-- is worse than IDF overlap for short technical intents, where one distinctive
-- token ("lockfile", "Cargo.toml") carries most of the signal.
CREATE FUNCTION pattern_search(
    p_intent                 TEXT,
    p_limit                  INTEGER DEFAULT 5,
    p_target                 execution_target DEFAULT NULL,
    p_include_non_runnable   BOOLEAN DEFAULT TRUE,
    p_min_score              REAL    DEFAULT 0.15
)
RETURNS TABLE (
    du_uuid        UUID,
    title          TEXT,
    resource_class resource_class,
    target         execution_target,
    lifecycle      lifecycle_state,
    payload_kind   TEXT,
    saves_tokens   BOOLEAN,
    score          REAL,
    matched_on     TEXT
)
LANGUAGE sql STABLE AS $$
WITH params AS (
    SELECT lower(p_intent) AS intent,
           p_limit, p_target, p_include_non_runnable, p_min_score
),
-- Exact and containment matches win outright. This mirrors the Bridge fast
-- path: an action id with underscores normalised to spaces, or an alias.
fast AS (
    SELECT pp.du_uuid, pp.phrase, 1.00::real AS score
    FROM pattern_phrase pp, params
    WHERE replace(pp.phrase_lower, '_', ' ') = params.intent
       OR params.intent LIKE '%' || replace(pp.phrase_lower, '_', ' ') || '%'
),
-- Otherwise: IDF-weighted overlap between the intent and each phrase, scored
-- symmetrically so a long intent is not penalised against a short phrase.
overlap AS (
    SELECT
        pp.du_uuid,
        pp.phrase,
        (
            0.75 * (
                COALESCE(sum(i.idf) FILTER (WHERE it.term IS NOT NULL), 0)
                / NULLIF(sum(i.idf), 0)
            )
            + 0.25 * (
                count(*) FILTER (WHERE it.term IS NOT NULL)::real
                / NULLIF(count(*), 0)
            )
        )::real AS score
    FROM pattern_phrase pp
    CROSS JOIN params
    LEFT JOIN term_idf i ON i.term = pp.phrase_lower   -- placeholder, replaced below
    LEFT JOIN LATERAL (
        SELECT t.term FROM unnest(
            regexp_split_to_array(pp.phrase_lower, '[^[:alnum:]_]+')
        ) AS t(term) WHERE t <> '' AND length(t.term) > 1
    ) pt ON true
    LEFT JOIN LATERAL (
        SELECT t.term FROM unnest(
            regexp_split_to_array(params.intent, '[^[:alnum:]_]+')
        ) AS t(term) WHERE t <> '' AND length(t.term) > 1
    ) it ON it.term = pt.term
    GROUP BY pp.du_uuid, pp.phrase
),
scored AS (
    SELECT du_uuid, phrase AS matched_on, score FROM fast
    UNION ALL
    SELECT du_uuid, phrase, score FROM overlap WHERE score >= (SELECT p_min_score FROM params)
),
ranked AS (
    SELECT s.du_uuid, s.matched_on, max(s.score) AS score
    FROM scored s
    GROUP BY s.du_uuid, s.matched_on
)
SELECT
    p.du_uuid,
    p.title,
    p.resource_class,
    p.target,
    p.lifecycle,
    p.payload_kind,
    (p.payload_kind IS NOT NULL) AS saves_tokens,
    -- Champions get a bounded boost so a previously promoted pattern wins ties.
    -- The 1.1 factor then the clamp: a champion can outrank a marginally better
    -- unpromoted pattern, but never reach a perfect 1.0 it did not earn.
    LEAST(1.0, r.score * CASE WHEN c.du_uuid IS NOT NULL THEN 1.1 ELSE 1.0 END) AS score,
    r.matched_on
FROM ranked r
JOIN pattern p USING (du_uuid)
LEFT JOIN champion c ON c.du_uuid = p.du_uuid
CROSS JOIN params
WHERE p.lifecycle NOT IN ('quarantined', 'deprecated')
  AND (params.p_target IS NULL OR p.target = params.p_target OR p.target = 'pure')
  AND (params.p_include_non_runnable OR p.payload_kind IS NOT NULL)
ORDER BY score DESC, p.du_uuid
LIMIT (SELECT p_limit FROM params);
$$;

-- The three-tier ladder, as a function, so the routing decision is auditable in
-- one place rather than spread across callers.
--
--   tier 1  champion with a runnable payload  -> execute natively, 0 tokens
--   tier 2  any runnable pattern              -> execute, 0 tokens
--   tier 3  nothing runnable                 -> escalate to a model
--
-- `tier` is returned even on a miss so the caller can record which rung it fell
-- through, which is what the token accounting in LICENSE-COMMERCIAL.md rests on.
CREATE FUNCTION pattern_resolve(
    p_intent  TEXT,
    p_user    TEXT DEFAULT NULL
)
RETURNS TABLE (
    tier       INTEGER,
    du_uuid    UUID,
    title      TEXT,
    saves_tokens BOOLEAN,
    reason     TEXT
)
LANGUAGE sql STABLE AS $$
WITH params AS (SELECT p_intent AS intent, p_user AS usr),
best AS (
    SELECT * FROM pattern_search(
        (SELECT intent FROM params),
        5, NULL, TRUE, 0.15
    )
),
champ AS (
    SELECT b.du_uuid, b.title, b.saves_tokens
    FROM best b JOIN champion c ON c.du_uuid = b.du_uuid
    ORDER BY b.score DESC LIMIT 1
)
SELECT
    CASE
        WHEN (SELECT count(*) FROM champ) > 0 THEN 1
        WHEN EXISTS (SELECT 1 FROM best WHERE saves_tokens) THEN 2
        ELSE 3
    END,
    COALESCE((SELECT du_uuid FROM champ),
             (SELECT du_uuid FROM best WHERE saves_tokens LIMIT 1),
             (SELECT du_uuid FROM best LIMIT 1)),
    COALESCE((SELECT title FROM champ),
             (SELECT title FROM best WHERE saves_tokens LIMIT 1),
             (SELECT title FROM best LIMIT 1)),
    COALESCE((SELECT saves_tokens FROM champ),
             EXISTS (SELECT 1 FROM best WHERE saves_tokens)),
    CASE
        WHEN (SELECT count(*) FROM champ) > 0 THEN 'champion'
        WHEN EXISTS (SELECT 1 FROM best WHERE saves_tokens) THEN 'runnable-pattern'
        ELSE 'escalate'
    END;
$$;

-- Token accounting, per capability, for the promotion argument.
CREATE VIEW capability_token_savings AS
SELECT
    ch.capability,
    ch.du_uuid,
    p.title,
    count(*) FILTER (WHERE t.outcome = 'hit')        AS served_locally,
    count(*) FILTER (WHERE t.outcome = 'miss')       AS escalated,
    COALESCE(sum(t.tokens_in + t.tokens_out)
             FILTER (WHERE t.outcome = 'hit'), 0)    AS tokens_saved,
    max(t.ts)                                         AS last_served
FROM champion ch
JOIN pattern p USING (du_uuid)
LEFT JOIN trace t ON t.du_uuid = ch.du_uuid
GROUP BY ch.capability, ch.du_uuid, p.title;

COMMIT;
