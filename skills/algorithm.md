# askillify - Skill Matching Algorithm

The core algorithm that matches project tasks to the optimal skills from the registry.

## Algorithm Flow

```
1. Receive Task Input
   └─ Task description, goals, constraints, context

2. Parse & Extract Requirements
   ├─ Technical domain (react, python, docker, etc.)
   ├─ Required capabilities (testing, deployment, optimization)
   ├─ Constraints (cost < $5, time < 2h, security level)
   └─ Success criteria (what "done" looks like)

3. Query Skill Registry
   ├─ Filter by category/tags matching
   ├─ Minimum install_count threshold (e.g., 100)
   └─ Exclude deprecated/experimental skills

4. Score Each Skill
   Composite formula:
   ```
   score = (capability_match × 0.4) +
           (popularity × 0.25) +
           (1/complexity × 0.2) +
           (success_rate × 0.15) -
           (constraint_penalty × 0.1)
   ```

   Where:
   - capability_match: 0.0-1.0 based on tag/category overlap
   - popularity: normalized install_count (0-100)
   - complexity: inverse (1/complexity_score), lower is better
   - success_rate: from execution_log historical data
   - constraint_penalty: 0 if meets all constraints, 0.5+ if violates

5. Rank & Select Top-N
   - Sort by score descending
   - Select top 3 skills per task
   - Ensure diversity (don't pick 3 similar skills)

6. Generate Dispatch Plan
   - Determine execution order
   - Identify parallelizable skills
   - Set up dependency resolution
   - Output: execution plan with skill IDs and order
```

## Capability Matching

**Tag overlap calculation:**
```
match_score = |task_tags ∩ skill_tags| / |task_tags ∪ skill_tags|
```
(Jaccard similarity of tag sets)

**Category match:**
- +0.3 if task.category == skill.category
- +0.1 if related categories (e.g., "web" and "frontend")
- 0 otherwise

## Example

Task: "optimize Next.js app performance"
- Task tags: ['performance', 'nextjs', 'bundling', 'optimization']
- Skill "nextjs-performance" tags: ['performance', 'nextjs']
  - match = 2/4 = 0.5
  - Other factors add up to final score
- Skill "general-performance" tags: ['performance']
  - match = 1/4 = 0.25 but higher popularity may compensate