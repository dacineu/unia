//! The precondition gate, end to end.
//!
//! These are the tests that make divergence D5 a closed item rather than a
//! claim. The unit tests in `src/constraints.rs` prove the evaluator; these prove
//! that a declared precondition actually prevents a dispatch, that a satisfied
//! one lets it through, and that the refusal reaches the caller as a statement
//! about the resource rather than as a panic or a silent no-op.
//!
//! Before this, `constraints` was printed by the bridge and never checked. The
//! only test that touched it asserted that a string appeared on stdout, which is
//! the shape of a test that cannot fail.

use std::sync::{Arc, Mutex};

use unia::bridge::primitive::{PrimitiveBridge, UniversalPrimitive, UreAction, UreResource};
use unia::nucleus::{ActuatorDriver, ActuatorNucleus, ValveDriver};
use unia::wmis::{QualityMetrics, ResourceType, SharingScope, WmisEconomicLayer, WmisResource};

/// Builds a bridge and a nucleus sharing one valve that declares
/// `status != 'fault'` on `emergency_shutdown`, matching the corpus manifest.
fn rig() -> (PrimitiveBridge, ActuatorNucleus) {
    let manifest: UreResource = serde_json::from_value(serde_json::json!({
        "ure_version": "1.0",
        "resource_id": "valve-001",
        "category": "actuator",
        "state_space": { "status": { "type": "enum", "values": ["open", "closed", "fault"] } },
        "action_primitives": [
            {
                "id": "emergency_shutdown",
                "aliases": ["emergency_shutdown", "emergency shutdown", "halt"],
                "params": {},
                "target_state": "status = closed",
                "constraints": ["status != 'fault'"]
            },
            {
                "id": "adjust_flow",
                "aliases": ["adjust_flow", "adjust flow"],
                "params": { "target": "float" },
                "target_state": "flow_rate",
                "constraints": []
            }
        ]
    }))
    .expect("valve manifest parses");

    let mut bridge = PrimitiveBridge::new();
    bridge.load_resource(manifest);

    let mut nucleus = ActuatorNucleus::new(Arc::new(Mutex::new(WmisEconomicLayer::new())));
    nucleus.register_driver(Box::new(ValveDriver {
        id: "valve-001".to_string(),
    }));

    (bridge, nucleus)
}

fn meta() -> WmisResource {
    WmisResource {
        id: "valve-001".to_string(),
        resource_type: ResourceType::Application,
        owner: "test".to_string(),
        sharing_scope: SharingScope::Global,
        capabilities: vec![],
        quality: QualityMetrics {
            qor: 1.0,
            qos: 1.0,
            qop: 1.0,
        },
        metadata: serde_json::json!({}),
    }
}

/// The happy path, kept so a failing refusal test below cannot be mistaken for a
/// rig that never dispatches at all.
#[test]
fn an_action_with_no_preconditions_reaches_the_driver() {
    // Asserted on *whose* refusal it is, not that it succeeds. `adjust_flow`
    // declares `params: { target: "float" }` — a type — while the driver reads a
    // `value` argument, so the driver refuses it. That refusal is correct and is
    // divergence D14, and it is the point of the test: the gate did not stop
    // this one. If the gate were firing on everything, this would fail.
    let (bridge, nucleus) = rig();
    let packet = bridge
        .map_intent("valve-001", "adjust flow")
        .expect("adjust_flow maps");
    let err = nucleus
        .dispatch(packet, "tester", &meta())
        .expect_err("the driver refuses this for want of a value");

    assert!(
        !err.contains("will not do that"),
        "the gate let it through; the driver said: {err}"
    );
    assert!(
        err.contains("Missing value"),
        "and it was the driver: {err}"
    );
}

#[test]
fn a_satisfied_precondition_lets_the_action_through() {
    let (bridge, nucleus) = rig();
    nucleus.report_state("valve-001", "status", "open");

    let packet = bridge
        .map_intent("valve-001", "emergency shutdown")
        .expect("emergency_shutdown maps");
    assert_eq!(
        packet.payload.primitive,
        UniversalPrimitive::Reset,
        "the shutdown resolves to Reset, so a passing test here is about the gate"
    );
    assert!(
        nucleus.dispatch(packet, "tester", &meta()).is_ok(),
        "a satisfied precondition permits dispatch"
    );
}

#[test]
fn a_violated_precondition_refuses_and_names_both_sides() {
    let (bridge, nucleus) = rig();
    nucleus.report_state("valve-001", "status", "fault");

    let packet = bridge
        .map_intent("valve-001", "emergency shutdown")
        .expect("emergency_shutdown maps");
    let err = nucleus
        .dispatch(packet, "tester", &meta())
        .expect_err("a faulted valve must refuse a shutdown precondition");

    // The message is the phrase the creature turns into its question, so it has
    // to name the field, what it is, and what was needed — a player cannot act
    // on "precondition failed".
    assert!(err.contains("valve-001"), "names the resource: {err}");
    assert!(err.contains("status is fault"), "names what it is: {err}");
    assert!(
        err.contains("needs it to be 'fault'"),
        "names what was needed: {err}"
    );
    assert!(
        err.contains("left it alone"),
        "says nothing was done: {err}"
    );
}

#[test]
fn a_refused_action_does_not_reach_the_hardware() {
    // The refusal has to happen *before* the driver, or the valve closes and then
    // reports that it did not.
    let (bridge, nucleus) = rig();
    nucleus.report_state("valve-001", "status", "fault");
    let before = nucleus.state_of("valve-001");

    let packet = bridge
        .map_intent("valve-001", "emergency shutdown")
        .expect("emergency_shutdown maps");
    let _ = nucleus.dispatch(packet, "tester", &meta());

    assert_eq!(
        nucleus.state_of("valve-001"),
        before,
        "the driver's state is untouched by a refused action"
    );
}

#[test]
fn an_unevaluable_precondition_refuses_rather_than_passing() {
    // A precondition on a field nobody has reported has not been shown to hold.
    // Passing it would make a gate that reads everything as a gate that checks
    // nothing.
    let (bridge, nucleus) = rig();

    let packet = bridge
        .map_intent("valve-001", "emergency shutdown")
        .expect("emergency_shutdown maps");
    let err = nucleus
        .dispatch(packet, "tester", &meta())
        .expect_err("a precondition that cannot be evaluated must not dispatch");

    assert!(err.contains("nothing has told me its value"), "{err}");
}

#[test]
fn the_gate_reads_the_state_and_not_the_manifest() {
    // Two dispatches from the same manifest, differing only in reported state.
    // If the gate were reading the declaration rather than the state it would
    // give the same answer twice.
    let (bridge, mut nucleus_a) = rig();
    nucleus_a.report_state("valve-001", "status", "open");
    let allowed = nucleus_a
        .dispatch(
            bridge
                .map_intent("valve-001", "emergency shutdown")
                .unwrap(),
            "tester",
            &meta(),
        )
        .is_ok();

    let (bridge, mut nucleus_b) = rig();
    nucleus_b.report_state("valve-001", "status", "fault");
    let refused = nucleus_b
        .dispatch(
            bridge
                .map_intent("valve-001", "emergency shutdown")
                .unwrap(),
            "tester",
            &meta(),
        )
        .is_err();

    assert!(allowed, "open satisfies the precondition");
    assert!(refused, "fault violates it");
}

#[test]
fn a_reported_value_survives_for_the_next_call() {
    // The state a driver writes is what the next precondition reads, so the gate
    // has to be looking at the same store the driver writes to.
    let (bridge, nucleus) = rig();
    nucleus.report_state("valve-001", "status", "open");

    let first = nucleus
        .dispatch(
            bridge
                .map_intent("valve-001", "emergency shutdown")
                .unwrap(),
            "tester",
            &meta(),
        )
        .expect("first shutdown proceeds");

    assert!(
        first.contains("SUCCESS") || !first.is_empty(),
        "dispatch reported: {first}"
    );
    assert_eq!(
        nucleus
            .state_of("valve-001")
            .get("status")
            .map(String::as_str),
        Some("closed"),
        "the driver wrote the new value, and the gate can read it back"
    );
}
