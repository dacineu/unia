# amater Rust + SQLite Redesign Design

## Project Redescope

Redesigning the amater agent manager system with **Rust** language and **SQLite** database, focusing on easy aggregation with easy patterns for easy decisions and actions (Option A priority).

## Core Database Philosophy

The SQLite database serves as a **resources easy unique identification schema** with:

### Storage Model
- **Local files stored**: SOUL.md, USER.md, AGENTS.md, TOOLS.md, HEARTBEAT.md, MEMORY.md, SKILL.md in local directory
- **Database stores**: Unique identifiers (UUIDs) referencing those files, plus benchmarking data and correlations
- **Job descriptions**: Stored in .md files locally, only unique IDs in database
- **Resources**: Agent names, file paths, CPU/memory specs, local jobs, executed jobs, plus external URLs (job markets, APIs, skill markets)

### Resource Types

| Category | Local Examples | External Examples |
|----------|---------------|-------------------|
| **Agent Names** | openclaw, zeroclaw, claude, any available | - |
| **Identity Files** | SOUL.md, USER.md, AGENTS.md, etc. (file paths) | - |
| **Resource Metadata** | CPU fill, memory fill, other defined later | - |
| **Local Jobs** | Created jobs, executed finalised jobs | - |
| **External URLs** | - | Job marketplaces, job URLs, API URLs, skill market URLs |

### Correlation Model
- Database entries **reference** local files by unique ID
- Enables easy aggregation and pattern analysis
- Supports top-10 most used combinations ranking
- Facilitates later refinement with details on each combination

## Database Schema (SQLite + Rust)

### 1. `agents` Table
Stores agent console names and basic metadata:

```sql
CREATE TABLE agents (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,          -- e.g., "openclaw", "zeroclaw", "claude"
    description TEXT,                   -- Brief description
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

### 2. `identities` Table
References SOUL.md etc. stored locally:

```sql
CREATE TABLE identities (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    agent_id INTEGER,                   -- FK to agents.id (nullable)
    file_path TEXT NOT NULL UNIQUE,     -- Path to SOUL.md, USER.md, etc.
    identity_type TEXT,                 -- "soul", "user", "agents", "tools", "heartbeat", "memory", "skill"
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(agent_id, file_path)
);
```

### 3. `resources` Table
Resource metadata (CPU, memory, local/remote jobs):

```sql
CREATE TABLE resources (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    agent_id INTEGER,                   -- FK to agents.id (nullable)
    name TEXT NOT NULL,                 -- Resource name/description
    resource_type TEXT,                 -- "cpu", "memory", "local_job", "external_url"
    local_path TEXT,                    -- Path if local (nullable)
    url_value TEXT,                     -- URL if external (nullable)
    cpu_spec TEXT,                      -- CPU specification (nullable)
    memory_spec TEXT,                   -- Memory specification (nullable)
    status TEXT DEFAULT "active",       -- "active", "executed", "archived"
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(agent_id, name, resource_type)
);
```

### 4. `benchmark_combinations` Table
Top combinations tracking:

```sql
CREATE TABLE benchmark_combinations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    combination_hash TEXT NOT NULL,     -- Hash of resource IDs combined
    resource_count INTEGER,             -- Number of resources in combination
    usage_count INTEGER DEFAULT 0,      -- How many times used
    success_rate REAL DEFAULT 0.0,      -- Success rate (0.0-1.0)
    avg_execution_time_ms INTEGER,      -- Average execution time
    last_used TIMESTAMP,                -- When last used
    rank_score REAL,                    -- Composite ranking score
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(combination_hash)
);
```

### 5. `job_marketplaces` Table
External URL resources:

```sql
CREATE TABLE job_marketplaces (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,                 -- e.g., "LinkedIn", "Indeed", "Upwork"
    url_pattern TEXT NOT NULL,          -- URL pattern with placeholders
    category TEXT,                      -- "full-time", "freelance", "gig"
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

## Key Design Decisions

### 1. File-Based Identities + DB References
- SOUL.md, USER.md etc. stored as `.md` files in local directory
- Database only stores UUIDs/IDs referencing those files
- Enables easy file management + fast DB queries
- Later: can add file hashes for change detection

### 2. Resource Correlation via IDs
- Each resource (agent + type) gets unique DB ID
- Combinations tracked via `combination_hash` 
- Top 10 most used combinations easily queried
- Pattern analysis supported for future refinement

### 3. URL Resources for External Integration
- `job_marketplaces` table stores URL patterns
- Can later fetch and correlate with internal resources
- URL patterns support placeholders for dynamic filling
- API endpoints can be added similarly

### 4. Simple Ranking for Top 10
- `rank_score` composite: `usage_count * 0.7 + success_rate * 100 * 0.3`
- Auto-updated on each use
- Top 10 query: `ORDER BY rank_score DESC LIMIT 10`
- Easy patterns for easy decisions

## Rust Implementation Structure

```
/amater-rust/
├── Cargo.toml              -- Rust project config
├── src/
│   ├── main.rs           -- Entry point, CLI interface
│   ├── database.rs       -- SQLite operations via rusqlite
│   ├── identity.rs       -- Identity file management
│   ├── resource.rs       -- Resource tracking (local + external)
│   ├── benchmark.rs      -- Combination benchmarking + top-10
│   ├── job_market.rs     -- URL marketplace integration
│   └── models.rs         -- Data structures matching SQL tables
├── data/                 -- Local .md files directory
│   ├── souls/
│   ├── users/
│   ├── agents/
│   └── ...
└── README.md
```

## CLI Interface (Rust)

```
amater --help                          Show help
amater list agents                     List agent console names
amater add agent --name openclaw       Add agent to database
amater add identity --type soul --file souls/programmer.md
amater add resource --name "quick job" --type local_job --agent openclaw
amater add url --marketplace LinkedIn --pattern "https://linkedin.com/jobs/"
amater benchmark --combination "openclaw+programmer" --success 1 --time 2300
amater top10                           Show top 10 most used combinations
amater search --query "programmer"     Search resources/agents
amater report --format json             Generate benchmark report
```

## Top 10 Combinations Query

```sql
-- Auto-updated rank_score: usage_count * 0.7 + success_rate * 100 * 0.3
SELECT 
    bc.id,
    a1.name || '+' || a2.name as resource_pair,
    bc.usage_count,
    bc.success_rate,
    bc.avg_execution_time_ms,
    (bc.usage_count * 0.7 + bc.success_rate * 100 * 0.3) as rank_score
FROM benchmark_combinations bc
JOIN resources r1 ON bc.resource_id_1 = r1.id
JOIN resources r2 ON bc.resource_id_2 = r2.id
JOIN agents a1 ON r1.agent_id = a1.id
JOIN agents a2 ON r2.agent_id = a2.id
ORDER BY rank_score DESC
LIMIT 10;
```

*(Simplified - actual implementation may use hash-based combination tracking)*

## Decision-Making Support

### Easy Aggregation Patterns
1. **Most Used**: Top 10 combinations by rank_score
2. **Best Success**: Highest success_rate combinations
3. **Fastest Execution**: Lowest avg_execution_time_ms
4. **Balanced**: Combined score considering both usage and success

### Easy Patterns for Decisions
- "Use openclaw+programmer identity 70% of time (success rate 94%)"
- "zeroclaw + linux-admin resources executes in 2.3s avg"
- "Claude + affiliate-marketer URLs generates most gig opportunities"

### Later Refinement Path
- Add resource performance details per combination
- Track user feedback on combinations
- Add seasonal/periodic ranking variations
- Integrate with actual agent execution results

## Risk Mitigation

| Risk | Mitigation |
|------|-----------|
| Schema changes later | Use SQLite's ALTER TABLE carefully; version the schema |
| File/db desync | Implement sync routine; file hash checking |
| URL patterns become stale | Mark is_active=FALSE, add new patterns |
| Top-10 too narrow | Configurable limit, track top-N with N adjustable |
| Rust compilation issues | Use stable Rust features; thorough testing |

## Success Criteria

- [ ] SQLite database created with 5 tables as specified
- [ ] 7 identity file formats tracked (SOUL.md, USER.md, AGENTS.md, TOOLS.md, HEARTBEAT.md, MEMORY.md, SKILL.md)
- [ ] Agent console names: openclaw, zeroclaw, claude, any available stored
- [ ] URL resources for job marketplaces stored
- [ ] Top 10 combinations ranking works correctly
- [ ] CLI interface functional for all core operations
- [ ] Local .md file integration works (store/retrieve)
- [ ] Benchmark data tracked and reportable
- [ ] Top-10 combinations query returns correct results

## Next Steps

1. Initialize Rust project with SQLite dependency (rusqlite)
2. Create database schema and migrate existing data if any
3. Implement CLI interface for all operations
4. Integrate local .md file management
5. Implement top-10 combinations ranking
6. Test with sample data and iterate

**This redesigned amater system with Rust + SQLite provides easy aggregation, easy patterns, and easy decisions for resource management and benchmarking, with the flexibility to complexify later with details on each combination of resources used.**