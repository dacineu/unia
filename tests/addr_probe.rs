// Does the artifact's identity depend on its surface form?
use serde_json::json;
use unia::identifiers::DuUuid;

#[test]
fn probe_address_language_sensitivity() {
    let english = json!({
        "resource_id": "valve-001",
        "action_primitives": [
            {"id":"emergency_shutdown","aliases":["emergency_shutdown","emergency shutdown","halt"]},
            {"id":"adjust_flow","aliases":["adjust_flow","adjust flow","set flow"]}
        ]
    });
    // The SAME capability, expressed in Romanian instead.
    let romanian = json!({
        "resource_id": "valve-001",
        "action_primitives": [
            {"id":"emergency_shutdown","aliases":["oprit de urgenta","inchide","deteneaza"]},
            {"id":"adjust_flow","aliases":["regleaza debitul","ajusteaza","set flow"]}
        ]
    });
    // The same capability, English, plus one more alias discovered in the wild.
    let english_plus_one = json!({
        "resource_id": "valve-001",
        "action_primitives": [
            {"id":"emergency_shutdown","aliases":["emergency_shutdown","emergency shutdown","halt","cut power"]},
            {"id":"adjust_flow","aliases":["adjust_flow","adjust flow","set flow"]}
        ]
    });

    let a = DuUuid::generate(&english, None).unwrap();
    let b = DuUuid::generate(&romanian, None).unwrap();
    let c = DuUuid::generate(&english_plus_one, None).unwrap();

    println!("  english                 = {}", a.as_simple());
    println!("  romanian                = {}", b.as_simple());
    println!("  english + 1 new alias   = {}", c.as_simple());
    println!();
    println!("  romanian == english?          {}", a == b);
    println!("  +1 alias == original english? {}", a == c);
}
