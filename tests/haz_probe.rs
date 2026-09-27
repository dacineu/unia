#[test]
fn probe_hazards() {
    use unia::mcp::store::{Store};
    use std::io::Write;
    let d = std::env::temp_dir().join("unia-haz");
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();

    // A valve whose shutdown action carries cross-lingual aliases, as proposed.
    let m = r#"{"ure_version":"1.0","resource_id":"valve-001","category":"actuator",
      "action_primitives":[
        {"id":"emergency_shutdown","aliases":["emergency_shutdown","emergency shutdown","halt",
          "inchide","oprit","deteneaza"],"params":{},"target_state":"","constraints":[]},
        {"id":"adjust_flow","aliases":["adjust_flow","adjust flow","set flow","regleaza","throttle"],
          "params":{},"target_state":"","constraints":[]}]}"#;
    std::fs::write(d.join("valve-001.ure"), m).unwrap();
    let s = Store::open(&d);

    for q in ["inchide supapa",           // close the valve (Romanian)
              "oprit supapa urgent",      // stop the valve urgently
              "regleaza debitul",         // regulate the flow
              "open the valve",           // the ANTONYM of inchide
              "close the valve",          // the ANTONYM of open
              "deschide supapa"] {       // open the valve (Romanian)
        let hits = s.search(q, 1);
        let top = hits.first().map(|m| format!("{} ({:.3})", m.id, m.score)).unwrap_or("-".into());
        println!("  {q:<26} -> {top}");
    }
    let _ = std::fs::remove_dir_all(&d);
    
}
