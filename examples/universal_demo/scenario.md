# Universal Demo Scenario: The "Autonomous Infrastructure" Flow

## Objective
Demonstrate a single agent managing a heterogeneous environment (Industrial Hardware + System Software) using the unia pipeline without changing its cognitive context.

## Scenario Steps
1. **Phase 1: Industrial Control**
   - **Intent**: "Emergency shutdown the main water valve"
   - **Path**: Intent $\rightarrow$ Bridge (Semantic Alias) $\rightarrow$ Nucleus $\rightarrow$ `ValveDriver` (GPIO Low)
   - **Verification**: Flow rate = 0.0

2. **Phase 2: System Administration**
   - **Intent**: "Dump the system error logs to a file"
   - **Path**: Intent $\rightarrow$ Bridge (Semantic Mapping) $\rightarrow$ Nucleus $\rightarrow$ `FileSystemDriver` (Syscall Write)
   - **Verification**: File created at /var/log/system.log

3. **Phase 3: Environmental Sensing**
   - **Intent**: "Check the current ambient temperature"
   - **Path**: Intent $\rightarrow$ Bridge (Primitive Mapping) $\rightarrow$ Nucleus $\rightarrow$ `TempSensorDriver` (I2C Read)
   - **Verification**: Reading = 22.4C

## Success Criteria
- All three disparate tasks are handled by the same `MetaOrchestrator` instance.
- No custom code is written for the "intent" phase; only `.ure` manifests are used.
- Total execution time for the sequence remains sub-millisecond.
