#[cfg(test)]
mod tests {
    use crate::bridge::primitive::{PrimitiveBridge, UreResource, UreAction, StateType, UniversalPrimitive};
    use std::collections::HashMap;

    #[test]
    fn test_primitive_mapping_flow() {
        let mut bridge = PrimitiveBridge::new();

        // 1. Setup a sample URE resource (Smart Valve)
        let mut state_space = HashMap::new();
        state_space.insert("flow_rate".to_string(), StateType {
            r#type: "float".to_string(),
            range: Some((0.0, 1.0)),
            unit: Some("percentage".to_string()),
            values: None,
        });

        let actions = vec![
            UreAction {
                id: "emergency_shutdown".to_string(),
                params: HashMap::new(),
                target_state: "flow_rate = 0.0".to_string(),
                constraints: vec!["status != 'fault'".to_string()],
            },
            UreAction {
                id: "adjust_flow".to_string(),
                params: {
                    let mut p = HashMap::new();
                    p.insert("target".to_string(), "float".to_string());
                    p
                },
                target_state: "flow_rate".to_string(),
                constraints: vec![],
            },
        ];

        let valve_ure = UreResource {
            ure_version: "1.0".to_string(),
            resource_id: "valve-001".to_string(),
            category: "actuator".to_string(),
            state_space,
            action_primitives: actions,
        };

        bridge.load_resource(valve_ure);

        // 2. Test Intent: "Emergency shutdown the valve"
        let intent = "Please perform an emergency_shutdown on the valve";
        let result = bridge.map_intent("valve-001", intent);

        assert!(result.is_ok(), "Bridge should map the intent to a primitive");
        let packet = result.unwrap();
        
        // Verify the mapping: emergency_shutdown -> UniversalPrimitive::Reset
        assert_eq!(packet.payload.primitive, UniversalPrimitive::Reset);
        assert_eq!(packet.payload.resource_id, "valve-001");
        println!("✅ Successfully mapped 'shutdown' intent to UniversalPrimitive::Reset");

        // 3. Test Intent: "Adjust the flow"
        let intent_2 = "adjust_flow the valve to 50%";
        let result_2 = bridge.map_intent("valve-001", intent_2);

        assert!(result_2.is_ok());
        let packet_2 = result_2.unwrap();
        
        // Verify the mapping: adjust_flow -> UniversalPrimitive::SetValue
        assert_eq!(packet_2.payload.primitive, UniversalPrimitive::SetValue);
        println!("✅ Successfully mapped 'adjust' intent to UniversalPrimitive::SetValue");
    }
}
