# askillify Database Schema

Defines the PostgreSQL database structure for skill aggregation and dispatching.

## Tables

### `skill_registry`
Central repository of all available skills from all sources.

| Column | Type | Description |
|--------|------|-------------|
| `id` | SERIAL PRIMARY KEY | Unique identifier |
| `name` | VARCHAR(255) | Skill name (e.g., "react-best-practices") |
| `description` | TEXT | What the skill does |
| `source` | VARCHAR(100) | Origin: 'local', 'skills.sh', 'github', 'custom' |
| `source_id` | VARCHAR(255) | skills.sh ID or GitHub repo/ref |
| `category` | VARCHAR(100) | Domain: 'web', 'testing', 'deploy', etc. |
| `tags` | TEXT[] | Keywords: ['performance', 'optimization', etc.] |
| `install_count` | INTEGER DEFAULT 0 | Popularity metric |
| `popularity_score` | DECIMAL(10,2) | Normalized popularity (0-100) |
| `complexity_score` | DECIMAL(10,2) | 0.0-1.0, lower = easier to use |
| `status` | VARCHAR(50) | 'active', 'deprecated', 'experimental' |
| `capabilities` | JSONB | What tasks this skill can handle |
| `created_at` | TIMESTAMP DEFAULT NOW() | When skill was added |
| `updated_at` | TIMESTAMP DEFAULT NOW() | Last update |

### `project_skills`
Skills assigned to specific projects.

| Column | Type | Description |
|--------|------|-------------|
| `project_id` | UUID PRIMARY KEY | Unique project identifier |
| `skill_id` | INTEGER REFERENCES skill_registry(id) | Assigned skill |
| `assignment_order` | INTEGER | Order of skill execution |
| `assigned_at` | TIMESTAMP DEFAULT NOW() | When skill was assigned |
| `status` | VARCHAR(50) | 'pending', 'active', 'completed', 'failed' |
| `performance_metrics` | JSONB | Timing, tokens, success rate |
| `constraints` | JSONB | Project-specific constraints |

### `task_skill_mapping`
Maps skills to tasks within projects.

| Column | Type | Description |
|--------|------|-------------|
| `id` | SERIAL PRIMARY KEY | Unique mapping ID |
| `project_id` | UUID REFERENCES project_skills(project_id) | Parent project |
| `task_description` | TEXT | Description of the task |
| `required_capabilities` | JSONB | Capabilities needed |
| `suggested_skill_id` | INTEGER REFERENCES skill_registry(id) | Recommended skill |
| `confidence_score` | DECIMAL(5,2) | 0.0-1.0, how well skill matches |
| `assigned_at` | TIMESTAMP DEFAULT NOW() | When mapping was created |

### `skill_execution_log`
Tracks skill execution performance.

| Column | Type | Description |
|--------|------|-------------|
| `id` | SERIAL PRIMARY KEY | Unique log ID |
| `skill_id` | INTEGER REFERENCES skill_registry(id) | Executed skill |
| `project_id` | UUID | Parent project |
| `task_id` | UUID | Task identifier |
| `execution_time_ms` | INTEGER | How long skill took |
| `tokens_used` | INTEGER | LLM tokens consumed |
| `success` | BOOLEAN | Whether execution succeeded |
| `error_message` | TEXT | Error details if failed |
| `executed_at` | TIMESTAMP DEFAULT NOW() | When execution happened |

## Indexes

```sql
-- Optimize skill lookups by category and tags
CREATE INDEX idx_skill_registry_category ON skill_registry(category);
CREATE INDEX idx_skill_registry_tags ON skill_registry using gist(tags gist__text_pattern_ops);
CREATE INDEX idx_skill_registry_popularity ON skill_registry(popularity_score DESC);
CREATE INDEX idx_skill_registry_complexity ON skill_registry(complexity_score ASC);

-- Optimize project-skill queries
CREATE INDEX idx_project_skills_project ON project_skills(project_id);
CREATE INDEX idx_task_mapping_project ON task_skill_mapping(project_id);

-- Optimize execution tracking
CREATE INDEX idx_execution_skill ON skill_execution_log(skill_id);
CREATE INDEX idx_execution_project ON skill_execution_log(project_id);
CREATE INDEX idx_execution_time ON skill_execution_log(execution_time_ms);
```