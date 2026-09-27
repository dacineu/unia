# Formal Specification: Universal Resource (.ure) Format v1.0

## 1. Conceptual Model
A `.ure` file defines a **Resource Tuple**: 
$$\mathcal{U} = \langle \mathcal{I}, \mathcal{S}, \mathcal{A}, \mathcal{C} \rangle$$

Where:
- $\mathcal{I}$ (**Identity**): A unique, immutable identifier for the resource.
- $\mathcal{S}$ (**State Space**): The set of all possible valid states the resource can occupy.
- $\mathcal{A}$ (**Action Primitives**): The set of atomic transitions that can move the resource from one state to another.
- $\mathcal{C}$ (**Constraints**): The boundary conditions (security, physics, logic) that must be satisfied for an action to be valid.

---

## 2. Structural Grammar (Formal Definition)

The `.ure` format is a strictly typed, declarative language. Its structure follows this hierarchy:

### 2.1 The Header (Metadata)
Every `.ure` file must begin with a metadata block:
- `ure_version`: Semantic version of the specification.
- `resource_id`: Global unique ID (UUID).
- `category`: The functional class (e.g., `actuator`, `sensor`, `memory`, `compute`).

### 2.2 State Definitions ($\mathcal{S}$)
States are defined as a set of keys and value-types.
- **Example:** A `Light` resource has states: `{ "power": Boolean, "brightness": Range(0, 100) }`.
- **Formalism:** $\mathcal{S} = \{ (k_1, v_1), (k_2, v_2), \dots, (k_n, v_n) \}$ where $v \in \text{Types}$.

### 2.3 Action Primitives ($\mathcal{A}$)
Actions are defined as **Transitions**. An action does not specify "how" to change the state, but "what" the target state change is.
- **Syntax:** `ActionName(params) $\rightarrow$ StateChange`
- **Example:** `SetPower(Boolean) $\rightarrow$ power = Boolean`
- **Formalism:** $\mathcal{A} = \{ a_1, a_2, \dots, a_m \}$ where each $a$ is a function $a: \mathcal{S} \times \text{Params} \rightarrow \mathcal{S}'$.

### 2.4 Constraints ($\mathcal{C}$)
Constraints are predicates that must return `True` for an action to be dispatched to the Nucleus.
- **Example:** `SetBrightness(v) requires power == True`.
- **Formalism:** $\mathcal{C} = \{ p_1, p_2, \dots, p_k \}$ where $p$ is a predicate $p: \mathcal{S} \rightarrow \{0, 1\}$.

---

## 3. Implementation Representation (JSON-LD/YAML)

While the logic is formal, the implementation is represented in a machine-readable format (like YAML or JSON-LD) for the Primitive Bridge.

**Example: `smart_valve.ure`**
```yaml
ure_version: 1.0
resource_id: "valve-001-alpha"
category: actuator

state_space:
  flow_rate: { type: "float", range: [0.0, 1.0], unit: "percentage" }
  status: { type: "enum", values: ["open", "closed", "fault"] }

action_primitives:
  - id: "adjust_flow"
    params: { target: "float" }
    target_state: "flow_rate"
    constraints:
      - "status != 'fault'"

  - id: "emergency_shutdown"
    params: {}
    target_state: "flow_rate = 0.0"
    constraints: []
```

---

## 4. Interaction Lifecycle (The Bridge Logic)

When an agent expresses an intent, the **Primitive Bridge** follows this logic:

1. **Intent Parsing:** Agent says $\rightarrow$ "Shut down the valve."
2. **URE Lookup:** Bridge finds `smart_valve.ure` $\rightarrow$ matches "Shut down" to `emergency_shutdown`.
3. **Constraint Validation:** Bridge checks current state $\mathcal{S}$. If $\mathcal{C}$ is satisfied $\rightarrow$ Proceed.
4. **Primitive Dispatch:** Bridge emits a **Universal Primitive Packet**:
   `{ resource_id: "valve-001-alpha", primitive: "emergency_shutdown", params: {} }`
5. **Actuator Mapping:** The Nucleus finds the driver for `valve-001-alpha` and triggers the hardware-specific command (e.g., `WRITE_GPIO(PIN_4, LOW)`).

---

## 5. Complexity Summary

| Component | Complexity | Responsibility |
| :--- | :--- | :--- |
| **.ure Definition** | $O(1)$ | Defines the "What" (The Interface) |
| **Primitive Bridge** | $O(P)$ | Maps Intent $\rightarrow$ Primitive |
| **Actuator Nucleus** | $O(1)$ | Maps Primitive $\rightarrow$ Hardware |

By formalizing the `.ure` format this way, we ensure that adding a new piece of hardware only requires writing a new `.ure` file and a driver, **without ever touching the AI's cognitive weights.**
