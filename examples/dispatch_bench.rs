//! Measures what one interpreted act costs, in absolute terms.
//!
//! **This exists because the benchmark that was here before it was not a
//! measurement.** `tests/benchmark_tests.rs` compared one dispatch against
//! `thread::sleep(800ms) + thread::sleep(50ms)` and printed the quotient as an
//! "efficiency gain". The numerator was a duration the test itself slept
//! through, so the number measured `sleep` and passed unconditionally. A
//! comparison against a fictional baseline is worse than no comparison, because
//! it prints a figure that looks like a result.
//!
//! So this measures the only thing that can be measured without choosing what
//! the other side of the comparison *means*: the cost of one act, and where that
//! cost goes. The absolute figure is a fact about this machine. Interpreting it —
//! in particular against Lua — requires a claim that one unia act and one Lua
//! statement are the same work, and that claim is not this file's to make.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use unia::bridge::primitive::{PrimitiveBridge, StateType, UreAction, UreResource};
use unia::nucleus::{ActuatorNucleus, ValveDriver};
use unia::wmis::{QualityMetrics, ResourceType, SharingScope, WmisEconomicLayer, WmisResource};

/// How many dispatches to time. Large enough that the clock's own cost is
/// negligible, small enough that a test run stays fast.
const N: u32 = 50_000;

/// A valve that is already reported as `open`, so the precondition gate passes
/// and the measurement times a *permitted* act rather than a refusal.
fn fixture() -> (PrimitiveBridge, ActuatorNucleus, WmisResource) {
    let mut bridge = PrimitiveBridge::new();
    let economy = Arc::new(Mutex::new(WmisEconomicLayer::new()));
    // The economic gate charges every act, and a fresh balance is 100 against a
    // cost of 2, so a 50,000-act benchmark would run dry on act 51 and the panic
    // would be a refusal rather than a measurement. Funding it explicitly keeps
    // the timing on the *permitted* path, which is the one being measured: a
    // refused act never reaches the driver, so timing refusals would measure the
    // gate instead of the act.
    {
        let mut econ = economy.lock().unwrap();
        econ.reward_contributor("benchmark", (N as f64) * 4.0);
    }
    let mut nucleus = ActuatorNucleus::new(economy);

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
    (bridge, nucleus, resource_meta)
}

fn main() {
    let (bridge, nucleus, resource_meta) = fixture();
    let intent = "Emergency shutdown the valve";

    // 1. Mapping an intent to a packet — the front half of an act.
    let start = Instant::now();
    for _ in 0..N {
        let _ = bridge
            .map_intent("valve-001", intent)
            .expect("bridge mapping");
    }
    let map_only = start.elapsed();

    // 2. The whole act: map, gate, route, construct, execute.
    let start = Instant::now();
    for _ in 0..N {
        let packet = bridge
            .map_intent("valve-001", intent)
            .expect("bridge mapping");
        nucleus
            .dispatch(packet, "benchmark", &resource_meta)
            .expect("dispatch");
    }
    let whole = start.elapsed();

    // 3. The same two lines the act prints, timed on their own.
    //
    //    This is not a footnote. `charge_actuation` and `ValveDriver::execute`
    //    each `println!` unconditionally, so two formatted writes are inside
    //    every timed iteration above. Without this row the headline number is a
    //    measurement of stdout, and stdout is not the interpreter.
    let start = Instant::now();
    for i in 0..N {
        println!(
            "💰 Charged {:.2} tokens from {}. Resource: valve-001 (QoR: {:.2}). New balance: {:.2}",
            2.0,
            "benchmark",
            1.0,
            100000.0 - i as f64
        );
        println!("[Nucleus] SUCCESS: Valve valve-001 force-closed via Hardware GPIO Low");
    }
    let log_only = start.elapsed();

    // 4. Mapping and dispatch are both `O(1)` in `N`, so the difference is the
    //    dispatch alone rather than a per-iteration drift.
    let dispatch_only = whole.saturating_sub(map_only);

    let per = |d: std::time::Duration| d.as_secs_f64() / N as f64;
    println!("unia dispatch, {} acts, stdout to a pipe", N);
    println!("  map intent only : {:>9.0} ns/act", per(map_only) * 1e9);
    println!("  whole act       : {:>9.0} ns/act", per(whole) * 1e9);
    println!(
        "  of which logging: {:>9.0} ns/act  ({:.0}% of the act)",
        per(log_only) * 1e9,
        100.0 * per(log_only) / per(whole)
    );
    println!(
        "  act less logging: {:>9.0} ns/act",
        (per(whole) - per(log_only)) * 1e9
    );
    println!(
        "  dispatch only   : {:>9.0} ns/act",
        per(dispatch_only) * 1e9
    );
    println!(
        "  throughput      : {:>9.0} acts/s",
        N as f64 / whole.as_secs_f64()
    );
    println!();
    println!("Absolute figures only. No baseline is claimed, and none can be: the");
    println!("question \"is one unia act worth as much as one Lua statement\" is a");
    println!("claim about work, not a timing.");
}
