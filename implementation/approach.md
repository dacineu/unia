# askillify - Implementation Approach

## Phase 1: Foundation (Weeks 1-2)

### 1.1 Set Up Project Structure
- [x] Create directory structure at `/home/dacineu/dev/dev-skills/askillify/`
- [ ] Initialize Git repository
- [ ] Set up PostgreSQL database
- [ ] Apply schema.sql to create tables

### 1.2 Import Existing Skills
- [ ] Migrate 33 skills from `/home/dacineu/dev/dev-rust/zeroclaw/.claude/skills/`
- [ ] Parse each skill's SKILL.md to extract:
  - Name, description, categories, tags
  - Capabilities and requirements
- [ ] Store in `skills/local/` directory
- [ ] Populate `skill_registry` table

### 1.3 Configure Skill Sources
- [ ] Set up Skills CLI integration (`npx skills find`)
- [ ] Configure GitHub skill discovery
- [ ] Define API endpoints for skill search
- [ ] Create fallback mechanism if external sources unavailable

### 1.4 Basic CLI Setup
- [ ] Create `askillify CLI entry point
- [ ] Implement `askillify list` - show available skills
- [ ] Implement `askillify search [query]` - search skills
- [ ] Implement `askillify dispatch --task "description"` - main feature

## Phase 2: Core Engine (Weeks 3-4)

### 2.1 Task Analyzer
- [ ] Implement NLP-based task decomposition
- [ ] Extract technical domains, capabilities, constraints
- [ ] Parse success criteria from task descriptions
- [ ] Output: structured task requirements JSON

### 2.2 Skill Matching Algorithm
- [ ] Implement capability matching (Jaccard similarity on tags)
- [ ] Implement category matching logic
- [ ] Calculate composite scoring formula
- [ ] Rank and select top-N skills per task

### 2.3 Dispatcher Service
- [ ] Implement dependency resolution (topological sort)
- [ ] Implement parallel execution group detection
- [ ] Handle failure and retry logic
- [ ] Generate execution plans with ordering

### 2.4 Performance Tracking
- [ ] Log every skill execution to `skill_execution_log`
- [ ] Track execution_time_ms, tokens_used, success/failure
- [ ] Update skill_registry scores after each execution
- [ ] Generate performance reports

## Phase 3: Optimization (Weeks 5-6)

### 3.1 Optimization Loop
- [ ] After each project: analyze execution_log
- [ ] Update skill popularity_score and complexity_score
- [ ] Identify underperforming skills
- [ ] Recommend skill improvements or replacements

### 3.2 User Feedback Integration
- [ ] Allow users to rate skill effectiveness
- [ ] Capture user feedback on skill quality
- [ ] Manual complexity_score adjustments
- [ ] A/B test different skill selections

### 3.3 Constraint Management
- [ ] Project-level constraint configuration
- [ ] Max cost, time, security boundaries
- [ ] Automatic constraint checking before dispatch
- [ ] Constraint violation warnings and overrides

## Phase 4: Polish (Weeks 7-8)

### 4.1 User Interface
- [ ] Dashboard to view skills and metrics
- [ ] Task planning interface with skill recommendations
- [ ] Execution history and analytics
- [ ] Skill installation/management commands

### 4.2 Documentation
- [ ] Write user guide
- [ ] API documentation
- [ ] Design specs in `docs/superpowers/specs/`
- [ ] Example projects and use cases

### 4.3 Testing & Quality
- [ ] Unit tests for all components
- [ ] Integration tests for end-to-end flow
- [ ] Performance benchmarks
- [ ] Edge case handling

## Technology Stack

| Layer | Technology |
|-------|-----------|
| Database | PostgreSQL with JSONB support |
| Language | Python 3.11+ (for CLI and engine) or Rust (for performance) |
| ORM | SQLAlchemy or Diesel |
| CLI | Click (Python) or clap (Rust) |
| AI/NLP | Transformers or custom regex-based parser |
| Docker | For skill execution isolation |
| Logging | structlog or pino |

## Risk Mitigation

| Risk | Mitigation |
|------|-----------|
| Skill database too small | Start with local skills, gradually integrate external sources |
| Poor skill-task matching | Iteratively refine scoring formula based on feedback |
| Skill execution failures | Implement retry logic, fallback skills, user approval for risky skills |
| Performance degradation over time | Regular optimization loop to update skill scores |
| Constraint violations | Hard bounds checking, user approval for constraint overrides |

## Success Criteria

- [ ] Database contains 100+ skills from all sources
- [ ] Task analysis correctly extracts requirements 80%+ of time
- [ ] Skill matching selects correct skill in top-3 results 70%+ of time
- [ ] Dispatcher generates valid execution plans with dependency handling
- [ ] Performance metrics tracked and visible in dashboard
- [ ] Optimization loop improves skill selection over 3+ project cycles
- [ ] User can complete "dispatch a task" from start to finish in < 5 minutes