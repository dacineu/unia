-- Development seed. Not part of the schema: this exists so the retrieval path
-- can be exercised against known patterns, and so a reviewer can see the
-- ranking behave as the specification describes.
--
-- The three patterns are chosen to cover the two matching paths: an intent that
-- contains a stored phrase verbatim (the containment fast path) and one that
-- shares only vocabulary (the IDF overlap path). The rust pattern is what a
-- coding agent would actually ask for, and it is the case the linear scan in
-- src/mcp/store.rs handles worst, since it contains no substring of the title.

TRUNCATE pattern CASCADE;

INSERT INTO pattern
    (du_uuid, ure_version, resource_class, title, guidance, target, source_path, lifecycle)
VALUES
    ('11111111-1111-4111-8111-111111111111', '1.0', 'tool', 'smart_valve',
     'Modulate flow on a smart valve actuator', 'native-linux',
     'patterns/actuator/native-linux/smart_valve.ure', 'champion'),
    ('22222222-2222-4222-8222-222222222222', '1.0', 'tool', 'filesystem',
     'Read, write and clear files on a mounted volume', 'native-linux',
     'patterns/actuator/native-linux/filesystem.ure', 'candidate'),
    ('33333333-3333-4333-8333-333333333333', '1.0', 'skill', 'cargo_dep',
     'Add a rust dependency to Cargo.toml and update the lockfile', 'pure',
     'patterns/skill/cargo_dep.ure', 'candidate');

INSERT INTO state_var (du_uuid, name, type_name, range_lo, range_hi, unit, enum_values)
VALUES
    ('11111111-1111-4111-8111-111111111111', 'flow_rate', 'float', 0.0, 1.0, 'percentage', NULL),
    ('11111111-1111-4111-8111-111111111111', 'status',   'enum',  NULL, NULL, NULL,
     ARRAY['open', 'closed', 'fault']),
    ('22222222-2222-4222-8222-222222222222', 'last_write', 'string', NULL, NULL, 'path', NULL);

INSERT INTO action (du_uuid, action_id, target_state, resolved_primitive, constraints, params)
VALUES
    ('11111111-1111-4111-8111-111111111111', 'emergency_shutdown', 'flow_rate = 0.0',
     'state_transition', ARRAY['status != ''fault'''], '{}'::jsonb),
    ('11111111-1111-4111-8111-111111111111', 'adjust_flow', 'flow_rate',
     'state_transition', '{}', '{"target":"float"}'::jsonb),
    ('22222222-2222-4222-8222-222222222222', 'read_file', 'output = file_content',
     'state_query', '{}', '{"path":"REQUIRED"}'::jsonb),
    ('22222222-2222-4222-8222-222222222222', 'write_file', 'last_write = current_path',
     'state_transition', ARRAY['disk_space > 0'], '{}'::jsonb),
    ('33333333-3333-4333-8333-333333333333', 'add_dependency', 'lockfile updated',
     'state_transition', '{}', '{}'::jsonb);

-- Pattern-level aliases have a NULL action_id; action-level aliases point at
-- their action. Both forms exist because the corpus is meant to hold both.
INSERT INTO alias (du_uuid, action_id, phrase, learned)
VALUES
    ('11111111-1111-4111-8111-111111111111', 'emergency_shutdown', 'emergency_shutdown', FALSE),
    ('11111111-1111-4111-8111-111111111111', 'emergency_shutdown', 'emergency shutdown', FALSE),
    ('11111111-1111-4111-8111-111111111111', 'emergency_shutdown', 'halt',             FALSE),
    ('11111111-1111-4111-8111-111111111111', 'adjust_flow',       'adjust flow',       FALSE),
    ('11111111-1111-4111-8111-111111111111', NULL,                'close the valve',   FALSE),
    ('22222222-2222-4222-8222-222222222222', 'read_file',         'open file',         FALSE),
    ('22222222-2222-4222-8222-222222222222', 'read_file',         'get content',       FALSE),
    ('22222222-2222-4222-8222-222222222222', 'write_file',        'save file',         FALSE),
    ('33333333-3333-4333-8333-333333333333', 'add_dependency',    'add rust dependency', FALSE),
    ('33333333-3333-4333-8333-333333333333', 'add_dependency',    'update lockfile',   FALSE),
    ('33333333-3333-4333-8333-333333333333', NULL,                'cargo toml',        FALSE);

-- An alias learned from observed phrasing, as induction would produce. Recorded
-- separately from the authored ones so the distinction is queryable.
INSERT INTO alias (du_uuid, action_id, phrase, learned)
VALUES ('33333333-3333-4333-8333-333333333333', 'add_dependency',
        'bump a crate and refresh the lock', TRUE);

INSERT INTO champion (capability, du_uuid, promoted_by, evidence)
VALUES ('valve.actuator', '11111111-1111-4111-8111-111111111111', 'seed',
        'development fixture, not a measured promotion');

INSERT INTO trace (ts, intent, du_uuid, outcome, tokens_in, tokens_out, resolved_by)
VALUES
    (now(), 'emergency shutdown the valve', '11111111-1111-4111-8111-111111111111', 'hit', 0, 0, 'champion'),
    (now(), 'read the config file',        '22222222-2222-4222-8222-222222222222', 'hit', 0, 0, 'runnable-pattern'),
    (now(), 'refactor the parser',         NULL,                                   'miss', 1840, 620, 'escalate'),
    (now(), 'write a migration',           NULL,                                   'miss', 2210, 910, 'escalate');
