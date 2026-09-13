# unia (Universal Nucleus Interface Architecture)

**unia** is a high-performance orchestration system that decouples agent intent from implementation. Using the **.ure** universal resource format and a **Primitive Bridge**, it evolves from OS-dependence to an actuator-driven nucleus, ensuring total hardware independence and evolutionary capability growth.

It serves as the core implementation of the **amater** agent manager design, designed for universal resource exchange, hardware independence, and evolutionary capability growth.

## 🚀 Core Vision

The goal of `unia` is to decouple the *Intent* of an agent's action from its *Implementation*. By moving away from hardcoded OS paths and binary dependencies, `unia` creates a "Punctuation Layer" that allows the system to evolve from being OS-dependent to being **Actuator-driven**.

## 🛠 Key Architectural Pillars

### 1. Universal Resource Exchange (`.ure`)
The system uses the `.ure` format for universal resource identification, moving beyond simple file paths to a structured identity scheme:
- **Identifier Scheme**: `ure_RES_CAT_LOC_UNIQ` (using UUID v4).
- **7 Resource Types**: `identity`, `skill`, `config`, `job`, `hardware`, `tool`, `doc`.
- **Quality Metrics (A-E)**: Tracks product quality, service performance (execution time, success rate), resource requirements, and access policies.
- **Correlation Mechanism**: Maps relationships between resources (e.g., a `skill` requiring a specific `identity`) to optimize agent selection.

### 2. Hardware & OS Independence
`unia` ensures the nucleus remains universal through:
- **Primitive Bridge**: Maps universal primitive IDs (e.g., `PRIMITIVE_SEARCH`) to OS-specific implementations.
- **Weight Manager**: Abstracts hardware specifics (GPU/CPU memory, precision) to handle LoRA loading and memory allocation across different architectures (CUDA, ROCm, Metal).
- **Actuator Mesh**: Allows the system to replace bootstrap OS primitives with specialized, evolved `.ure` actuators.

### 3. Agent Profiling & Orchestration
Based on the **amater** design, `unia` dynamically profiles subagents using a comprehensive identity set:
- **Identity Files**: `SOUL.md`, `USER.md`, `AGENTS.md`, `TOOLS.md`, `HEARTBEAT.md`, `MEMORY.md`, `SKILL.md`.
- **Profiling-Driven Selection**: Uses a combination of `.ure` metrics and correlation strengths to assign the optimal agent identity to a specific task.

## 📂 Project Structure

- `src/`: Core Rust implementation.
  - `orchestrator/`: Logic for task decomposition and agent assignment.
  - `registry/`: Management of `.ure` resources and identities.
  - `bridge/`: The OS decoupling layer.
  - `wmis/`: Integration with Universal Management Interface Standards.
- `docs/`: Detailed design specifications for `.ure` and decoupling strategies.
- `database/`: SQLite schemas for agents, identities, and correlations.
- `tests/`: Integration and end-to-end demo tests.

## 📈 Evolution Path

`unia` is designed to evolve autonomously:
**OS-Dependent** $\rightarrow$ **Primitive Bridge** $\rightarrow$ **Actuator-Driven** $\rightarrow$ **Universal Nucleus**

## ⚖️ License
This project is licensed under the terms specified in the `LICENSE` file.
