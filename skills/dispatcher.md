# askillify - Skill Dispatcher Mechanism

The system that takes matched skills and executes them in optimal order for project tasks.

## Dispatcher Workflow

```
┌─────────────────────────────────────────────────────┐
│  Task Input: "optimize Next.js performance"         │
└─────────────────────────────────────────────────────┘
                │
                ▼
┌─────────────────────────────────────────────────────┐
│  1. Task Decomposer                                 │
│  • Break complex tasks into subtasks                │
│  • Identify independent vs dependent subtasks       │
│  • Determine execution order (topological sort)     │
│  • Example:                                         │
│    - subtask A: "analyze bundle"                    │
│    - subtask B: "configure webpack plugins"         │
│    - subtask C: "test performance"                  │
│    - Order: A → B → C (B depends on A)             │
└─────────────────────────────────────────────────────┘
                │
                ▼
┌─────────────────────────────────────────────────────┐
│  2. Skill Matcher                                   │
│  • Query skill_registry for each subtask            │
│  • Score skills using algorithm                     │
│  • Select optimal skill per subtask                 │
│  • Return: skill_id, confidence, execution_order    │
└─────────────────────────────────────────────────────┘
                │
                ▼
┌─────────────────────────────────────────────────────┐
│  3. Dependency Resolver                             │
│  • Build dependency graph from selected skills      │
│  • Detect circular dependencies                     │
│  • Generate topological execution order             │
│  • Identify skills that can run in parallel         │
└─────────────────────────────────────────────────────┘
                │
                ▼
┌─────────────────────────────────────────────────────┐
│  4. Skill Executor                                  │
│  • Load skill metadata (capabilities, args)         │
│  • Invoke via /skill-manage or CLI wrapper          │
│  • Stream execution output to user                  │
│  • Track: execution_time_ms, tokens_used, success   │
│  • On completion: update skill_execution_log        │
└─────────────────────────────────────────────────────┘
                │
                ▼
┌─────────────────────────────────────────────────────┐
│  5. Performance Optimizer                           │
│  • After execution: update skill_scores in          │
│    skill_registry based on actual metrics           │
│  • Track trends: which skills succeed most often    │
│  • Suggest skill improvements or replacements        │
│  • Update complexity_score if execution was         │
│    unexpectedly complex/easy                       │
└─────────────────────────────────────────────────────┘
```

## Execution Plan Example

```json
{
  "task": "optimize Next.js performance",
  "subtasks": [
    {
      "id": "subtask-1",
      "description": "analyze bundle structure",
      "skill": "nextjs-performance-analysis",
      "order": 1,
      "parallelizable": false
    },
    {
      "id": "subtask-2", 
      "description": "configure webpack plugins",
      "skill": "webpack-optimization",
      "order": 2,
      "parallelizable": false,
      "depends_on": ["subtask-1"]
    },
    {
      "id": "subtask-3",
      "description": "run performance benchmarks",
      "skill": "performance-testing",
      "order": 3,
      "parallelizable": true,
      "depends_on": ["subtask-2"]
    }
  ],
  "execution_order": ["subtask-1", "subtask-2", "subtask-3"],
  "parallel_groups": [    ["subtask-1"],
    ["subtask-2"],
    ["subtask-3"]
  ]
}
```

## Key Features

1. **Dependency-aware** - Skills that depend on others wait their turn
2. **Parallel execution** - Independent skills run concurrently
3. **Failure handling** - If a skill fails, optionally retry or skip dependents
4. **Metrics tracking** - Every execution logs time, tokens, success/failure
5. **Continuous improvement** - Performance data feeds back into skill scoring