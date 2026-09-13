# askillify - Performance & Easiness Optimization Metrics

The metrics tracked and optimized for skill execution effectiveness.

## Tracked Metrics

### Execution Performance

| Metric | Type | Description | Target |
|--------|------|-------------|--------|
| `execution_time_ms` | INTEGER | How long the skill takes to execute from start to finish | Minimize (project-dependent) |
| `tokens_used` | INTEGER | Number of LLM tokens consumed during execution | Minimize (cost efficiency) |
| `success_rate` | DECIMAL(5,2) | Percentage of executions that complete without error | > 95% |
| `error_count` | INTEGER | Number of failed executions | Minimize |
| `retry_count` | INTEGER | Number of automatic retries attempted | Minimize |

### Skill Quality Metrics

| Metric | Type | Description | Target |
|--------|------|-------------|--------|
| `popularity_score` | DECIMAL(10,2) | Normalized popularity (0-100) based on install_count | Higher is better (more battle-tested) |
| `complexity_score` | DECIMAL(10,2) | 0.0-1.0 rating of how easy the skill is to use (lower = easier) | Lower is better |
| `capability_coverage` | DECIMAL(5,2) | Percentage of task requirements the skill covers | > 80% |
| `constraint_compliance` | DECIMAL(5,2) | How well skill meets project constraints (0-100%) | 100% target |

### Project-Level Metrics

| Metric | Type | Description | Target |
|--------|------|-------------|--------|
| `total_execution_time` | SUM of all execution_time_ms | Overall project skill execution time | Minimize |
| `total_tokens_consumed` | SUM of all tokens_used | Total cost of skill executions | Minimize |
| `skill_success_rate` | Percentage of successful vs failed | Overall project health | > 95% |
| `average_skill_confidence` | Average confidence_score across all tasks | How well skills match tasks | Higher is better |
| `dispatch_efficiency` | tasks_completed / total_tasks | How many tasks got skill recommendations | 100% target |

## Optimization Goals

### 1. Minimize Total Execution Time
- Track which skills are fastest for each task type
- Recommend fastest skills for recurring tasks
- Identify slow skills that may need replacement

### 2. Minimize Token/Cost Usage
- Log tokens per skill per task
- Identify token-hungry skills
- Suggest more efficient alternatives

### 3. Maximize Success Rate
- Track which skills most often fail
- Update complexity_score based on actual difficulty
- Flag skills with low success rates for review

### 4. Balance Complexity vs Capability
- When two skills have similar capability_match scores:
  - Prefer lower complexity_score (easier to use)
- When capability_match differs significantly:
  - Higher capability_match may justify higher complexity

### 5. Constraint Compliance
- Project-specific limits to track:
  - Max cost per project (tokens_used × price_per_token)
  - Max execution time per task
  - Security restrictions (which skills are allowed)
  - Resource limits (CPU, memory per skill)

## Scoring Formula Updates

After each execution, update skill_registry scores:

```
new_popularity_score = 0.9 * old_popularity_score + 0.1 * normalized_success_rate
new_complexity_score = 0.9 * old_complexity_score + 0.1 * actual_complexity_perception
```

Where `actual_complexity_perception` is derived from:
- Actual execution time vs expected
- User feedback on difficulty
- Token usage vs expected

## Dashboard Visualization

Recommended metrics for user dashboard:

1. **Skill effectiveness chart**: Popularity vs Complexity scatter plot
2. **Task success rate**: Which task types succeed most/least
3. **Execution time trends**: How performance changes over time
4. **Constraint compliance**: Project-specific constraint metrics
5. **Recommendation confidence**: Average confidence_score per task type