// Can a peer address an artifact WITHOUT human vocabulary?
#[test]
fn probe_primitive_addressing() {
    use unia::mcp::store::Store;
    let s = Store::open("patterns");
    for q in [
        "pour some kibble",          // human vocabulary
        "toarna porumb",             // Romanian vocabulary
        "emergency_shutdown",        // canonical action id
        "emergency shutdown",        // its space form
        "SetValue_CheckSense",       // an induced primitive sequence
        "SetValue CheckSense",       // same, spaced
        "valve-001",                 // content address
    ] {
        let hits = s.search(q, 1);
        let top = hits.first()
            .map(|m| format!("{} @ {:.3}", m.id, m.score))
            .unwrap_or_else(|| "NO MATCH".into());
        println!("  {q:<22} -> {top}");
    }
}
