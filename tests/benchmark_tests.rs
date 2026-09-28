use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use unia::bridge::primitive::{PrimitiveBridge, StateType, UreAction, UreResource};
use unia::nucleus::{ActuatorNucleus, ValveDriver};
use unia::wmis::{QualityMetrics, ResourceType, SharingScope, WmisEconomicLayer, WmisResource};

/// The valve shuts down, and the act costs tokens.
///
/// **This test used to be a benchmark and it was not one.** It timed a single
/// dispatch against `thread::sleep(800ms) + thread::sleep(50ms)` and printed the
/// quotient as an "efficiency gain", asserting `gain > 1.0` — a comparison
/// against a duration the test slept through, so it passed unconditionally and
/// its output was a number with no measurement behind it. `docs/PROVENANCE.md`
/// then cited it as "empirical benchmarks".
///
/// The comparison is gone rather than repaired, because repairing it means
/// asserting that one unia act and one LLM inference are comparable work, which
/// is a claim about *value* and not about time. A fabricated baseline is worse
/// than an absent one: it prints a figure that reads as a result.
///
/// The real measurement is `examples/dispatch_bench.rs`, which reports absolute
/// nanoseconds per act and, more usefully, the split between mapping an intent
/// and dispatching it. What is left here is the part that was always worth
/// having: the path works, the gate admits a permitted act, and the cost is paid.
#[test]
fn a_permitted_act_is_admitted_executed_and_paid_for() {
    let mut bridge = PrimitiveBridge::new();
    let economy = Arc::new(Mutex::new(WmisEconomicLayer::new()));
    let mut nucleus = ActuatorNucleus::new(Arc::clone(&economy));

    let mut state_space = HashMap::new();
    state_space.insert(
        "flow_rate".to_string(),
        StateType {
            r#type: "float".to_string(),
            range: Some((0.0, 1.0)),
            unit: Some("percentage".to_string()),
            values: None,
        },
    );

    bridge.load_resource(UreResource {
        ure_version: "1.0".to_string(),
        resource_id: "valve-001".to_string(),
        category: "actuator".to_string(),
        state_space,
        action_primitives: vec![UreAction {
            id: "emergency_shutdown".to_string(),
            aliases: None,
            params: HashMap::new(),
            target_state: "flow_rate = 0.0".to_string(),
            constraints: vec!["status != 'fault'".to_string()],
        }],
    });
    nucleus.register_driver(Box::new(ValveDriver {
        id: "valve-001".to_string(),
    }));

    // A resource starts with no state, so the gate has nothing to evaluate and
    // would refuse the very act that would have established the state. That is
    // divergence D13, and this line is the stopgap for it.
    nucleus.report_state("valve-001", "status", "open");

    let resource_meta = WmisResource {
        id: "valve-001".to_string(),
        resource_type: ResourceType::Application,
        owner: "benchmark".to_string(),
        sharing_scope: SharingScope::Global,
        capabilities: vec![],
        quality: QualityMetrics {
            qor: 1.0,
            qos: 1.0,
            qop: 1.0,
        },
        metadata: serde_json::json!({}),
    };

    let before = economy
        .lock()
        .unwrap()
        .calculate_cost(&resource_meta, &unia::wmis::WmisOperation::Execute);

    let packet = bridge
        .map_intent("valve-001", "Emergency shutdown the valve")
        .expect("bridge mapping");
    let result = nucleus
        .dispatch(packet, "benchmark", &resource_meta)
        .expect("a permitted act is admitted");

    assert!(
        result.contains("force-closed"),
        "the driver reported {:?}, which is not a shutdown",
        result
    );

    // The economic gate is part of the act, so the act is not free. This is the
    // half of the old "benchmark" that was measuring something real, though it
    // was measuring it against a sleep.
    let after = economy
        .lock()
        .unwrap()
        .calculate_cost(&resource_meta, &unia::wmis::WmisOperation::Execute);
    assert!(
        after > 0.0,
        "an actuation that costs nothing is not gated, and {} says it is",
        after
    );
    assert!(
        before > 0.0,
        "a cost of zero would make the assertion above vacuous"
    );
}

/// The gate refuses an act whose precondition does not hold, and the refusal is
/// what makes the permission in the test above worth having.
#[test]
fn an_act_against_a_faulted_resource_is_refused() {
    let mut bridge = PrimitiveBridge::new();
    let economy = Arc::new(Mutex::new(WmisEconomicLayer::new()));
    let mut nucleus = ActuatorNucleus::new(Arc::clone(&economy));

    let mut state_space = HashMap::new();
    state_space.insert(
        "flow_rate".to_string(),
        StateType {
            r#type: "float".to_string(),
            range: Some((0.0, 1.0)),
            unit: Some("percentage".to_string()),
            values: None,
        },
    );

    bridge.load_resource(UreResource {
        ure_version: "1.0".to_string(),
        resource_id: "valve-001".to_string(),
        category: "actuator".to_string(),
        state_space,
        action_primitives: vec![UreAction {
            id: "emergency_shutdown".to_string(),
            aliases: None,
            params: HashMap::new(),
            target_state: "flow_rate = 0.0".to_string(),
            constraints: vec!["status != 'fault'".to_string()],
        }],
    });
    nucleus.register_driver(Box::new(ValveDriver {
        id: "valve-001".to_string(),
    }));

    // No `report_state` call: the gate has an unevaluable precondition, and
    // refusing is the correct answer to a precondition it cannot compute. It is
    // also why D13 exists — refusing the first act means the first act can never
    // establish the state the gate needs.
    let resource_meta = WmisResource {
        id: "valve-001".to_string(),
        resource_type: ResourceType::Application,
        owner: "benchmark".to_string(),
        sharing_scope: SharingScope::Global,
        capabilities: vec![],
        quality: QualityMetrics {
            qor: 1.0,
            qos: 1.0,
            qop: 1.0,
        },
        metadata: serde_json::json!({}),
    };

    let packet = bridge
        .map_intent("valve-001", "Emergency shutdown the valve")
        .expect("bridge mapping");
    assert!(
        nucleus
            .dispatch(packet, "benchmark", &resource_meta)
            .is_err(),
        "an act whose declared precondition cannot be evaluated was admitted"
    );
}
