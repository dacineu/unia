# amater Implementation Plan

## Overview

Detailed phased implementation plan for the amater agent manager/administrator system, based on the approved design document at `/home/dacineu/dev/dev-skills/askillify/amater-agent-manager-design.md`.

## Phase 1: Database Foundation (Weeks 1-2)

### Milestone 1.1: Custom Job Role Database
- [ ] Create JSON/CSV database with predefined job roles
- [ ] Roles to include: CEO, CTO, Programmer, Linux Administrator, Quick Income Roles
- [ ] Quick Income Roles: freelancer, gig worker, dropshipper, affiliate marketer, virtual assistant, content creator, online tutor, survey taker, task broker
- [ ] Add benchmark data per role: execution_time_ms, tokens_used, success_rate, complexity_score
- [ ] Set up PostgreSQL database schema
- [ ] Import existing skills from `.claude/skills/` with benchmark data

### Milestone 1.2: Identity File Formats
- [ ] Define 7 identity file formats: SOUL.md, USER.md, AGENTS.md, TOOLS.md, HEARTBEAT.md, MEMORY.md, SKILL.md
- [ ] Create template files for each role type
- [ ] Establish markdown structure conventions
- [ ] Validate templates with example roles

### Milestone 1.3: Database Integration
- [ ] Set up PostgreSQL with JSONB columns for identity files
- [ ] Create API endpoints for role querying
- [ ] Implement role matching algorithm
- [ ] Set up caching layer (Redis) for benchmark data
- [ ] Test database queries and performance

### Deliverables - Phase 1
- PostgreSQL database with job roles and benchmark data
- 7 identity file templates (SOUL.md, USER.md, AGENTS.md, TOOLS.md, HEARTBEAT.md, MEMORY.md, SKILL.md)
- Role matching API
- Database performance benchmarks

---

## Phase 2: Profiling Engine (Weeks 3-4)

### Milestone 2.1: Agent Profiling Logic
- [ ] Implement agent profiling logic
- [ ] Parse task description to extract requirements
- [ ] Query database for optimal role matching
- [ ] Generate/profiles identity files based on matching
- [ ] Consider both human profile and agent profile factors

### Milestone 2.2: Identity File Generation
- [ ] Implement identity file generation from templates
- [ ] Populate SOUL.md with mission/philosophy based on role
- [ ] Populate USER.md with skills/background based on role
- [ ] Populate AGENTS.md with capabilities/strengths/weaknesses
- [ ] Populate TOOLS.md with tool proficiencies
- [ ] Populate HEARTBEAT.md with performance metrics
- [ ] Populate MEMORY.md with learned patterns
- [ ] Populate SKILL.md with skill definitions and trigger phrases

### Milestone 2.3: Role Matching Algorithm
- [ ] Implement Jaccard similarity for tag/capability matching
- [ ] Implement category-based matching bonuses
- [ ] Calculate composite scoring: capability_match × 0.4 + popularity × 0.25 + (1/complexity) × 0.2 + success_rate × 0.15
- [ ] Handle constraint checking (cost, time, security)
- [ ] Return top-N role matches with confidence scores

### Deliverables - Phase 2
- Agent profiling engine with task analysis
- Identity file generation from templates
- Role matching algorithm with scoring
- 20+ test cases with expected outputs

---

## Phase 3: Orchestration (Weeks 5-6)

### Milestone 3.1: Goal Analysis
- [ ] Implement goal parsing from natural language
- [ ] Identify intent, constraints, success criteria
- [ ] Determine if goal requires job search, task execution, or both
- [ ] Decompose complex goals into subtasks

### Milestone 3.2: Project Plan Creation
- [ ] Implement project plan creation from goals
- [ ] Decompose into concurrent subtasks
- [ ] Determine execution order (topological sort)
- [ ] Identify parallelizable skills
- [ ] Generate execution plan with ordering

### Milestone 3.3: Multi-threaded Execution Planning
- [ ] Implement dependency resolution between agents
- [ ] Generate execution order with parallel groups
- [ ] Handle failure and retry logic
- [ ] Manage agent dependencies and resource allocation
- [ ] Provide real-time execution tracking

### Deliverables - Phase 3
- Goal analysis engine
- Project plan creator/decomposer
- Multi-threaded execution planner
- 15+ test goals with execution plans

---

## Phase 4: Job Search & Gig Coordination (Weeks 7-8)

### Milestone 4.1: Internet Job Market Research
- [ ] Implement internet research capability
- [ ] Query job APIs (LinkedIn, Indeed, Glassdoor as fallback)
- [ ] Scrape current quick income opportunities
- [ ] Filter by user skills, availability, constraints
- [ ] Rank opportunities by potential success

### Milestone 4.2: Quick Income Gig Proposals
- [ ] Generate plans for simultaneous job tries
- [ ] Consider human profile (CV skills, experience)
- [ ] Consider agent profiles (artificial logic capabilities)
- [ ] Profile amater (new "employee") for the task
- [ ] Simultaneously profile idea makers, researchers, gig workers, freelancers
- [ ] Execute to reach initial goal

### Milestone 4.2: Human/Agent Profile Integration
- [ ] Integrate human profile from CV input
- [ ] Integrate online agent profiles (artificial logic)
- [ ] Match profiles to task requirements
- [ ] Update profiles dynamically based on execution results

### Deliverables - Phase 4
- Internet job research capability
- Quick income gig proposal generator
- Simultaneous task execution orchestrator
- Profile integration system

---

## Phase 5: Optimization & Polish (Weeks 9-10)

### Milestone 5.1: Dynamic Database Updates
- [ ] Implement post-execution database updates
- [ ] Update success rates based on actual results
- [ ] Adjust complexity scores from execution data
- [ ] Update popularity based on usage metrics
- [ ] Learn patterns from failures/successes

### Milestone 5.2: Performance Optimization
- [ ] Implement token usage tracking and optimization
- [ ] Execute time minimization strategies
- [ ] Success rate improvement loops
- [ ] Constraint compliance monitoring
- [ ] Generate performance reports

### Milestone 5.3: User Dashboard & Documentation
- [ ] Create dashboard to view skills and metrics
- [ ] Implement task planning interface with recommendations
- [ ] Execution history and analytics
- [ ] Skill installation/management commands
- [ ] Write user guide and design docs

### Milestone 5.4: Testing & Quality Assurance
- [ ] Unit tests for all components
- [ ] Integration tests for end-to-end flow
- [ ] Performance benchmarks
- [ ] Edge case handling
- [ ] User acceptance testing

### Deliverables - Phase 5
- Dynamic optimization system
- User dashboard with analytics
- Comprehensive documentation
- Test suite with coverage goals

---

## Implementation Timeline

| Phase | Weeks | Key Deliverables |
|-------|-------|-----------------|
| 1 - Database Foundation | 1-2 | Job role DB, identity templates, role matching API |
| 2 - Profiling Engine | 3-4 | Profiling logic, identity generation, matching algorithm |
| 3 - Orchestration | 5-6 | Goal analysis, project planning, multi-threaded execution |
| 4 - Job Search & Gig Coordination | 7-8 | Internet research, quick income proposals, profile integration |
| 5 - Optimization & Polish | 9-10 | Dynamic updates, performance optimization, dashboard, testing |

## Technology Stack

| Layer | Technology |
|-------|-----------|
| Database | PostgreSQL with JSONB support |
| Language | Python 3.11+ (orchestration) or Rust (performance-critical parts) |
| Orchestration | Custom engine adapting Zeroclaw workflow |
| Internet Research | Custom web scraper + Skills CLI (`npx skills find`) |
| Identity Files | Markdown files with structured format |
| Caching | Redis for benchmark data |
| API | REST endpoints for profiling and orchestration |
| Testing | pytest + custom test harness |

## Risk Mitigation

| Risk | Mitigation |
|------|-----------|
| Database too small/incomplete | Start with 20+ well-known job roles, gradually add from internet research |
| Poor role-task matching | Iteratively refine scoring formula based on feedback and execution results |
| Internet research limitations | Use as supplementary to custom role database |
| Agent execution failures | Retry logic, fallback agents, user approval for risky tasks |
| Profile accuracy drift | Regular dynamic updates from execution results, user feedback |
| Multi-threaded complexity | Careful dependency resolution, isolated execution environments |

## Success Criteria

- [ ] Database contains 50+ well-categorized job roles with benchmark data
- [ ] 7 identity files properly formatted for all roles
- [ ] Agent profiling matches agents to tasks 80%+ of time
- [ ] Orchestration generates valid execution plans with dependency handling
- [ ] Quick income goal execution achieves results within constraints
- [ ] Dynamic database updates improve matching over 3+ project cycles
- [ ] User completes "make me a quick income" in reasonable time
- [ ] Multi-threaded execution works reliably

## Next Steps After This Plan

Once this implementation plan is approved, proceed with:
1. Setting up the PostgreSQL database and schema
2. Creating the custom job role database
3. Building the profiling engine (Phase 1)
4. Iterative development through all 5 phases

**This implementation plan is ready for user review and approval before proceeding to actual development.**