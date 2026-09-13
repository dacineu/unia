# amater - Agent Manager/Administrator Design

## Project Overview

The **amater** system is an agent manager/administrator that dynamically profiles and reprofiles subagents with appropriate identities based on task requirements, enabling optimal task execution across various job roles and goals.

### Core Capabilities

1. **Database Integration**: Combines custom job role database with internet research capabilities
2. **Agent Profiling**: Generates/profiles identity files (SOUL.md, USER.md, AGENTS.md, TOOLS.md, HEARTBEAT.md, MEMORY.md, SKILL.md) for any agent role
3. **Orchestration**: Analyzes/creates project plans from given goals
4. **Multi-threaded Analysis**: Preemptive analysis before task execution with dynamic database updates
5. **Job Search & Execution**: Supports "quick income" goals with simultaneous gig tries

### Database Structure (Updated - `.ure` Format)

The database has been redesigned from PostgreSQL to SQLite with 5 tables (agents, identities, resources, benchmark_combinations, job_marketplaces) and integrates `.ure` format for universal resource identification.

### `.ure` Integration

The `.ure` format (United Resource Exchange Format) is now the universal resource identification system for amater:

- **Identifier scheme**: `ure_RES_CAT_LOC_UNIQ` with UUID v4 format
- **7 resource types**: identity, skill, config, job, hardware, tool, doc
- **Quality metrics A-E**: quality_of_product, quality_of_service, quality_of_resource, access_policy, checksum
- **5 correlation patterns**: skill dependency, config-chain, hardware-software, benchmark-links, tool-resource
- **Top-10 ranking**: A-F tiers with correlation-enhanced scoring
- **Backward compatible**: Existing fmdL identifiers documented for reference, UUID chosen over fmdL per Approach A

Full `.ure` design saved in:
- `docs/amater-ure-design.md` - Core design (368 lines)
- `docs/amater-ure-analysis.md` - Analysis for omniLLM/WMIS (111 lines)
- `docs/amater-ure-proposals.md` - Saved proposals for future projects (117 lines)

### Identity File Formats

Each agent role has these well-categorized and benchmarked identity files:

| File | Purpose | Key Contents |
|------|---------|--------------|
| **SOUL.md** | Core identity & philosophy | Mission, values, decision philosophy |
| **USER.md** | User/profile metadata | Name, background, skills, experience level, CV data |
| **AGENTS.md** | Agent capabilities | What this agent can do, tools, strengths, weaknesses |
| **TOOLS.md** | Tool proficiency | Which tools this agent uses best, how to invoke them |
| **HEARTBEAT.md** | Health & status | Last execution, success rate, token usage, performance metrics |
| **MEMORY.md** | Memory & learning | What this agent has learned, patterns, preferences, past successes |
| **SKILL.md** | Skill definitions | Specific skills, when to trigger, test cases, evaluation metrics |

**Example - Programmer Role** (see full format in design doc)

### Orchestration Workflow

```
┌──────────────────────────────────────────────────────────────┐
│  User Goal: "make me a quick income"                         │
└──────────────────────────────────────────────────────────────┘
                │
                ▼
┌──────────────────────────────────────────────────────────────┐
│  1. Goal Analysis                                             │
│  • Parse goal intent                                         │
│  • Identify constraints (time, cost, skills available)       │
│  • Determine if "quick income" requires job search           │
└──────────────────────────────────────────────────────────────┘
                │
                ▼
┌──────────────────────────────────────────────────────────────┐
│  2. Database Query                                            │
│  • Query custom job roles database                           │
│  • Research internet for current job markets                 │
│  • Get human profile (from CV or user input)                 │
│  • Get available agent profiles                              │
└──────────────────────────────────────────────────────────────┘
                │
                ▼
┌──────────────────────────────────────────────────────────────┐
│  3. Agent Profiling & Assignment                              │
│  • For "quick income": profile amater for job researcher     │
│  • Simultaneously profile: idea makers, researchers,         │
│    gig workers, freelancers                                  │
│  • Match agent identities to tasks based on benchmark data   │
│  • Consider: human skills, agent strengths, task requirements│
└──────────────────────────────────────────────────────────────┘
                │
                ▼
┌──────────────────────────────────────────────────────────────┐
│  4. Multi-threaded Execution Plan                            │
│  • Decompose into concurrent subtasks                       │
│  • Assign agents based on profiling                         │
│  • Resolve dependencies                                    │
│  • Generate execution order with parallel groups            │
└──────────────────────────────────────────────────────────────┘
                │
                ▼
┌──────────────────────────────────────────────────────────────┐
│  5. Execute & Monitor                                        │
│  • Run agents in parallel where possible                    │
│  • Track: execution_time, tokens, success_rate              │
│  • Dynamically update database                               │
│  • Provide real-time feedback to user                       │
└──────────────────────────────────────────────────────────────┘
```

### Quick Income Example

When user says "make me a quick income":

**Step 1: Goal Analysis**
- Intent: Generate income quickly
- Constraints: Time limit, available skills, preferred methods
- Requires: Job search + gig execution

**Step 2: Database Query**
- Custom roles: "freelancer", "gig worker", "dropshipper", "affiliate marketer"
- Internet research: Current quick income opportunities, trending gigs
- Human profile: User's skills, experience, availability (from CV)
- Agent profiles: Available Zeroclaw/ZeroCode agents with benchmarks

**Step 3: Agent Profiling**
- Profile amater as "job researcher" agent
- Simultaneously profile:
  - Idea maker (for business ideas)
  - Researcher (for market research)
  - Gig worker (for freelance tasks)
  - Freelancer (for skill-based income)
- Match identities to tasks based on benchmark data

**Step 4: Execution Plan**
- Subtask 1: Research quick income gigs (agent: researcher)
- Subtask 2: Apply to gigs (agent: freelancer)
- Subtask 3: Manage gigs (agent: amater/gig manager)
- Subtask 4: Track results (agent: researcher)

**Step 5: Execute & Monitor**
- Run all subtasks in parallel
- Track success rates, execution times
- Dynamically update database with results
- Provide feedback to user on what's working

### Performance Benchmarking

Tracked per agent role:

| Metric | Description | Target |
|--------|-------------|--------|
| `execution_time_ms` | How long agent takes to execute task | Minimize |
| `tokens_used` | LLM tokens consumed | Minimize (cost efficiency) |
| `success_rate` | % of executions without error | > 95% |
| `complexity_score` | 0.0-1.0, lower = easier | Lower is better |
| `popularity_score` | How many users/Projects use this role | Higher is better |
| `constraint_compliance` | How well agent meets project constraints | 100% target |

**Dynamic Updates**: After each execution, update the database with:
- New success rates
- Adjusted complexity scores
- Updated popularity based on actual usage
- Learned patterns from failures/successes

### Implementation Phases

**Phase 1: Database Foundation** (Weeks 1-2)
- Create custom job role database (CEO, CTO, Programmer, Linux Admin, quick income roles)
- Define identity file formats (SOUL.md, USER.md, AGENTS.md, TOOLS.md, HEARTBEAT.md, MEMORY.md, SKILL.md)
- Import existing skills from `.claude/skills/` with benchmark data
- Set up PostgreSQL schema

**Phase 2: Profiling Engine** (Weeks 3-4)
- Implement agent profiling logic
- Implement identity file generation/profiling
- Implement role matching algorithm
- Integrate with existing agent system

**Phase 3: Orchestration** (Weeks 5-6)
- Implement goal analysis
- Implement project plan creation/decomposition
- Implement multi-threaded execution planning
- Implement dependency resolution

**Phase 4: Job Search & Gig Coordination** (Weeks 7-8)
- Implement internet job market research
- Implement quick income gig proposals
- Implement simultaneous task execution
- Implement human profile integration

**Phase 5: Optimization & Polish** (Weeks 9-10)
- Implement dynamic database updates
- Implement performance optimization
- Implement user dashboard
- Write documentation and docs

### Technology Stack

| Layer | Technology |
|-------|-----------|
| Database | PostgreSQL with JSONB support for identity files |
| Language | Python 3.11+ (for orchestration logic) or Rust (for performance) |
| Orchestration | Custom engine or adapt existing Zeroclaw workflow |
| Internet Research | Custom web scraper + Skills CLI integration (`npx skills find`) |
| Identity Files | Markdown files with structured format |
| Caching | Redis for frequently accessed benchmark data |
| API | REST endpoints for agent profiling and orchestration |

### Risk Mitigation

| Risk | Mitigation |
|------|-----------|
| Database too small/incomplete | Start with well-known job roles, gradually add more from internet research |
| Poor role-task matching | Iteratively refine scoring formula based on feedback and execution results |
| Internet research limitations | Use as supplementary to custom role database, not solely dependent |
| Agent execution failures | Implement retry logic, fallback agents, user approval for risky tasks |
| Profile accuracy drift | Regular dynamic updates from execution results, user feedback loops |
| Multi-threaded complexity | Careful dependency resolution, isolated agent execution environments |

### Success Criteria

- [ ] Database contains 50+ well-categorized job roles with benchmark data
- [ ] Identity files (SOUL.md, USER.md, AGENTS.md, TOOLS.md, HEARTBEAT.md, MEMORY.md, SKILL.md) properly formatted for all roles
- [ ] Agent profiling correctly matches agents to tasks 80%+ of time
- [ ] Orchestration generates valid execution plans with dependency handling
- [ ] Quick income goal execution achieves results within user-defined time/cost constraints
- [ ] Dynamic database updates improve role matching over 3+ project cycles
- [ ] User can complete "make me a quick income" from start to finish in reasonable time
- [ ] Multi-threaded execution works reliably with proper dependency handling

---

**Design complete**. Ready for spec review and user approval before proceeding to implementation planning.