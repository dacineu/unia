# amater-ure-design.md

## `.ure` Format Overview

**Universal Resource Exchange Format** designed only for amater agent manager.
- **Not modified for omniLLM or WMIS projects** (Approach A confirmed)
- Saves analysis/proposals for future use in other projects

## Identifier Scheme

- **Format**: `ure_RES_CAT_LOC_UNIQ`
- **RES**: Resource type flag
  - `identity` - Agent identity (.ure manifest)
  - `skill` - Agent skill/execution capability
  - `config` - Agent configuration settings
  - `job` - Task/job resource (external URLs)
  - `hardware` - CPU/gpu/memory hardware specs
  - `tool` - Agent tool/resource
  - `doc` - Documentation/resource description
- **CAT**: Category / location type
  - `L` = Local (`.md` file + SQLite database)
  - `E` = External (URL, cloud resource, P2P node)
- **LOC**: Location flag (same as CAT for clarity)
  - `L` = Local storage
  - `E` = External/remote
- **UNIQ**: Unique identifier
  - **UUID v4 format**: `a1b2c3d4-e5f6-7g8h-9i10-jk11l12l12`
  - Standard ISO/IEC 9834-1 format
  - Universally unique, no central coordination needed

### Identifier Examples

| Identifier | Resource Type | Category | Location | Unique ID |
|------------|--------------|----------|----------|-----------|
| `ure_identity_L_a1b2c3d4-...` | identity | Local | Local | UUID v4 |
| `ure_skill_L_a1b2c3d4-...` | skill | Local | Local | UUID v4 |
| `ure_config_L_a1b2c3d4-...` | config | Local | Local | UUID v4 |
| `ure_job_E_a1b2c3d4-...` | job | External | External/URL | UUID v4 |
| `ure_hardware_C_a1b2c3d4-...` | hardware | Cloud | Cloud/Remote | UUID v4 |

### Backward Compatibility Note

- **UUID format chosen** over `fmdL` format (user confirmed)
- Existing `fmdL` identifiers work as reference but `.ure` uses UUIDs
- Migration layer can be created later if needed
- Analysis/proposals saved for other projects understand this choice

## Quality Metrics (All Accepted: A-E)

### A) `quality_of_product` - Single 0.0-1.0 Score

- **Purpose**: How well the resource fulfills its purpose/goal
- **Range**: 0.0 (fulfills none of its purpose) to 1.0 (fulfills all of its purpose)
- **Example**: `0.95` = 95% fulfills its intended purpose
- **Used in**: Top-10 ranking (first ranking criterion, highest weight)

### B) `quality_of_service` - Sub-metrics

- **Purpose**: Execution performance and reliability metrics
- **Sub-metrics** (all optional, default values apply if absent):

| Sub-metric | Format | Default | Description |
|------------|--------|---------|-------------|
| `avg_execution_time_ms` | Integer (ms) | `5000` | Average execution time in milliseconds |
| `success_rate` | Float (0.0-1.0) | `1.0` | Percentage of successful executions |
| `tokens_per_task` | Integer | `1000` | LLM tokens consumed per task |

- **Used in**: Top-10 ranking (second criterion, after quality_of_product)

### C) `quality_of_resource` - Sub-metrics

- **Purpose**: Hardware/resource requirements and preferences
- **Sub-metrics** (all optional):

| Sub-metric | Format | Default | Description |
|------------|--------|---------|-------------|
| `min_cpu_percent` | Float (percentage) | `10.0` | Minimum CPU percentage required |
| `min_memory_mb` | Integer (MB) | `512` | Minimum memory in MB required |
| `preferred_agent` | String | `'any'` | Which agent console prefers this resource |

- **Example**: `preferred_agent: "openclaw"` - This resource works best with openclaw agent
- **Used in**: Agent profiling, resource selection filtering

### D) `access_policy` - Sub-metrics

- **Purpose**: Distribution and access preferences
- **Sub-metrics** (all optional):

| Sub-metric | Format | Default | Description |
|------------|--------|---------|-------------|
| `priority` | String | `'custom'` | Access order: `local_first`, `edge`, `cloud`, `custom` |
| `ttl_hours` | Integer (hours) | `168` | Time to live in cache/hours |
| `replication` | Integer | `1` | Number of copies across locations (local:1, edge:2, cloud:0) |

- **Example**: `priority: "local_first"` - Prefer local access before edge/cloud
- **Used in**: Distributed resource access, P2P networking

### E) `checksum` - SHA-256 Hash

- **Purpose**: Integrity verification of the `.ure` manifest
- **Format**: 64-character hexadecimal string (SHA-256)
- **Example**: `a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2`
- **Used in**: Resource integrity checks, corruption detection

### manifest.json Structure Example

```json
{
  "resource_id": "ure_identity_L_a1b2c3d4-e5f6-7g8h-9i10-jk11l12l12",
  "quality_of_product": 0.95,
  "quality_of_service": {
    "avg_execution_time_ms": 2300,
    "success_rate": 0.94,
    "tokens_per_task": 1200
  },
  "quality_of_resource": {
    "min_cpu_percent": 10,
    "min_memory_mb": 512,
    "preferred_agent": "openclaw"
  },
  "access_policy": {
    "priority": "local_first",
    "ttl_hours": 168,
    "replication": 1
  },
  "checksum": "a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2",
  "correlations": [
    {
      "related_resource_id": "ure_skill_L_b2c3d4e5-f6g7-7h8i-9j10-kl11m12n13o1",
      "relationship_type": "requires",
      "strength": 0.98,
      "category": "skill_to_identity"
    }
  ],
  "history": [
    {
      "timestamp": "2024-01-15T10:30:00Z",
      "old_resource_id": null,
      "new_resource_id": "ure_identity_L_a1b2c3d4-e5f6-7g8h-9i10-jk11l12l12",
      "reason": "initial_creation",
      "strength_delta": 0.0
    }
  ]
}
```

## Correlation Mechanism (5 Patterns)

### Pattern 1: Agent Skill Dependency
- **Relationship**: `requires`
- **Description**: Skill requires identity to function
- **Example**: `ure_skill` → `ure_identity` (skill needs identity)

### Pattern 2: Configuration-Skill Chain
- **Relationships**: `provides`, `complements`
- **Description**: Identity provides capabilities, skill executes them
- **Example**: `ure_identity` → `ure_config` → `ure_skill`

### Pattern 3: Hardware-Software Compatibility
- **Relationship**: `complements`
- **Description**: Hardware and software work well together
- **Example**: `ure_hardware` → `ure_software` (CPU ↔ LLM software)

### Pattern 4: Benchmark Combination Links
- **Relationship**: `derived_from`
- **Description**: Resource evolved from or used in benchmark
- **Example**: `benchmark_combinations` → `ure_resources`

### Pattern 5: Tool-Resource Association
- **Relationship**: `requires`
- **Description**: Tool requires resource for task execution
- **Example**: `ure_tool` → execution resources

## Top 10 Complications Enhancement

### Enhanced Ranking Formula

```
rank_score = (usage_count * 0.5) + 
             (success_rate * 100 * 0.3) + 
             (AVG(correlation_strength) * COUNT(*) * 0.2) +  -- Dynamic correlation bonus
             (quality_of_product * 0)  -- Reserved for future
```

### A-F Ranking Tiers

| Rank | Tier | Description |
|------|------|-------------|
| **A** | 1st | Highest rank_score - Best overall combination |
| **B** | 2nd | 2nd highest rank_score - Strong correlation network |
| **C** | 3rd | 3rd highest rank_score - Good product quality |
| **D** | 4th-5th | Next rank_scores - Serviceable combinations |
| **E** | 6th-8th | Lower rank_scores - Limited correlations |
| **F** | 9th-10th | Lowest rank_scores - Fallback options |

### SQL Query for Top 10

```sql
SELECT 
  bc.id,
  bc.resource_ids,
  bc.usage_count,
  bc.success_rate,
  COALESCE(SUM(c.strength), 0) as total_correlation_strength,
  (bc.usage_count * 0.5 + bc.success_rate * 100 * 0.3 + 
   COALESCE(SUM(c.strength), 0) * 0.2 + 
   bc.quality_of_product * 100 * 0.0) as rank_score
FROM benchmark_combinations bc
LEFT JOIN ure_resources ur ON bc.primary_resource_id = ur.resource_id
LEFT JOIN correlations c ON ur.resource_id = c.related_resource_id 
  AND c.relationship_type IN ('requires', 'provides', 'complements')
  AND c.strength > 0.7
GROUP BY bc.id, bc.usage_count, bc.success_rate, bc.quality_of_product
ORDER BY rank_score DESC
LIMIT 10;
```

## Agent Profiling Integration

### 7 Identity Types + Profiling

| Identity Type | `.ure` Manifest | Key Profiling Fields |
|--------------|----------------|---------------------|
| `identity` | `ure_identity_L_UUID.json` | `quality_of_product`, `role`, `specialization` |
| `skill` | `ure_skill_L_UUID.json` | `quality_of_service` (exec_time, success_rate), `difficulty_level` |
| `config` | `ure_config_L_UUID.json` | `quality_of_product`, `preferred_agent`, `settings` |
| `job` | `ure_job_E_UUID.json` | `quality_of_resource` (cpu, memory), `deadline`, `priority` |
| `hardware` | `ure_hardware_C_UUID.json` | `quality_of_resource` (cpu%, memory_mb), `capabilities` |
| `tool` | `ure_tool_L_UUID.json` | `quality_of_service` (execution_time, success), `tool_type` |
| `doc` | `ure_doc_L_UUID.json` | `quality_of_product` (relevance, accuracy), `category`, `tags` |

### Agent SQL Schema (Enhanced)

```sql
CREATE TABLE agents (
  id TEXT PRIMARY KEY,
  identity_id TEXT,           -- FK to ure_identities.resource_id
  skill_id TEXT,              -- FK to ure_skills.resource_id
  config_id TEXT,             -- FK to ure_configs.resource_id
  current_job_id TEXT,        -- FK to ure_jobs.resource_id
  hardware_id TEXT,           -- FK to ure_hardware.resource_id
  tool_id TEXT,               -- FK to ure_tools.resource_id
  doc_id TEXT,                -- FK to ure_docs.resource_id
  
  -- Profiling scores (from .ure metrics)
  quality_of_product REAL DEFAULT 0.5,
  quality_of_service_success_rate REAL DEFAULT 1.0,
  quality_of_service_avg_exec_ms INTEGER DEFAULT 5000,
  quality_of_service_tokens_per_task INTEGER DEFAULT 1000,
  quality_of_resource_min_cpu_percent REAL DEFAULT 10.0,
  quality_of_resource_min_memory_mb REAL DEFAULT 512,
  preferred_agent TEXT DEFAULT 'any',
  
  -- Correlation-aware fields
  correlation_count INTEGER DEFAULT 0,
  avg_correlation_strength REAL DEFAULT 0.0,
  last_correlation_update TIMESTAMP DEFAULT NOW(),
  
  -- Status
  is_active BOOLEAN DEFAULT true,
  last_heartbeat TIMESTAMP,
  total_tasks_completed INTEGER DEFAULT 0,
  total_tasks_successful INTEGER DEFAULT 0
);
```

### Profiling-Driven Agent Selection

```sql
-- Select top agents based on profile + correlations
SELECT 
  a.id,
  a.quality_of_product,
  a.quality_of_service_success_rate,
  COALESCE(SUM(c.strength), 0) as correlation_bonus,
  (a.quality_of_product * 50 + a.quality_of_service_success_rate * 30 + 
   COALESCE(SUM(c.strength), 0) * 20) as profile_score
FROM agents a
LEFT JOIN ure_skills s ON a.skill_id = s.resource_id
LEFT JOIN ure_hardware h ON a.hardware_id = h.resource_id
LEFT JOIN correlations c ON s.resource_id = c.related_resource_id 
  AND c.relationship_type = 'provides'
  AND c.strength > 0.7
WHERE a.is_active = true
  AND h.min_cpu_percent <= (SELECT current_cpu FROM system_status)
  AND h.min_memory_mb <= (SELECT current_memory FROM system_status)
GROUP BY a.id
ORDER BY profile_score DESC
LIMIT 5;
```

## Updatable Correlation Tracking

### History Tracking

Each correlation change is logged:

```json
"history": [
  {
    "timestamp": "2024-01-15T10:30:00Z",
    "old_resource_id": "ure_skill_L_old-uuid-1234",
    "new_resource_id": "ure_skill_L_new-uuid-5678",
    "reason": "skill_updated",
    "strength_delta": +0.05,
    "usage_after_change": 23
  }
]
```

### Strength Decay & Expiry

| Days Since Last Use | Penalty | Effective Strength |
|---------------------|---------|-------------------|
| 0-30 | 0.00 | 1.00 → 1.00 (full strength) |
| 31-60 | 0.035 | 1.00 → 0.965 |
| 61-90 | 0.070 | 1.00 → 0.930 |
| 91-180 | 0.150 | 1.00 → 0.850 |
| 181-365 | 0.280 | 1.00 → 0.720 |
| 365+ | 1.000 | 1.00 → 0.00 (expired) |

### SQLite Tracking Table

```sql
CREATE TABLE ure_correlation_history (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  manifest_id TEXT NOT NULL,
  resource_id TEXT NOT NULL,
  related_resource_id TEXT NOT NULL,
  relationship_type TEXT NOT NULL,
  strength REAL NOT NULL DEFAULT 1.0,
  usage_count INTEGER NOT NULL DEFAULT 0,
  last_used TIMESTAMP DEFAULT NOW(),
  created_at TIMESTAMP DEFAULT NOW(),
  changed_at TIMESTAMP DEFAULT NOW(),
  reason TEXT,
  old_resource_id TEXT,
  expiry_days INTEGER DEFAULT 365
);
```

### Correlation Triggers

| Trigger | Effect |
|---------|--------|
| `task_completion` | `usage_count++`, `strength += 0.01`, `last_used=NOW()` |
| `resource_update` | `changed_at=NOW()`, strength recalculated |
| `correlation_formed` | New row inserted, `strength=1.0`, `usage_count=1` |
| `correlation_broken` | `status=expired`, `reason=user_removed` |
| `system_cleanup` | Remove expired correlations (>365 days) |

## Design Summary

All 6 project plan items addressed:

1. ✅ **Correlation Mechanism Design** - UUID-based resource correlations
2. ✅ **Reusable Correlation Patterns** - 5 patterns for amater's 7 identity types
3. ✅ **Top 10 Complications Enhancement** - Enhanced ranking with correlation bonuses, A-F tiers
4. ✅ **Agent Profiling Integration** - 7 identity types with correlation-enhanced profiling
5. ✅ **Updatable Correlation Tracking** - History, decay, expiry, renewal
6. ✅ **Analysis/Proposals Files** - Saved for omniLLM/WMIS (no project modifications)

**User Preferences Respected**:
- Approach A only (`.ure` for amater only)
- UUID format for identifiers
- All quality metrics A-E accepted
- Top-10 rankings with A-F tiers
- Analysis/proposals saved for future use
- No modifications to omniLLM or WMIS projects