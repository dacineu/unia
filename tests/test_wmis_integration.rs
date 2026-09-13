#[cfg(test)]
mod tests {
    use askillify::wmis::{WmisAdapter, WmisDiscoveryProvider, DiscoveryQuery, SharingScope, WmisEconomicLayer, WmisPermissionEngine, WmisOperation};
    use serde_json::json;
    use uuid::Uuid;

    #[test]
    fn test_wmis_global_lifecycle() {
        // 1. Setup
        let mut registry = askillify::registry::ActuatorRegistry::new("simulated");
        let discovery = WmisDiscoveryProvider::new();
        let mut economy = WmisEconomicLayer::new();
        let mut permissions = WmisPermissionEngine::new();

        let manifest = json!({
            "resource_id": "global-actuator-1",
            "resource_type": "skill",
            "guidance": "Global Rust Expert",
            "complexity_score": 0.4
        });

        // 2. Convert to WMIS and Broadcast
        let wmis_res = WmisAdapter::from_ure(&manifest, "expert_user").unwrap();
        discovery.broadcast_actuator(wmis_res.clone());

        // 3. Discovery
        let query = DiscoveryQuery {
            seeker: "client_user".to_string(),
            allowed_scopes: vec![SharingScope::Global],
            min_qor: 0.7,
            tags: vec![],
        };
        let found = discovery.discover_resources(query);
        assert!(!found.is_empty());
        assert_eq!(found[0].id, "global-actuator-1");

        // 4. Permission & Economy
        let res_id = Uuid::new_v4();
        permissions.grant_permission(askillify::wmis::economy::ObjectivePermission {
            principal: "client_user".to_string(),
            resource_id: res_id,
            operations: vec![WmisOperation::Execute],
            percentage: 100.0,
        });

        assert!(permissions.check_permission("client_user", res_id, WmisOperation::Execute));
        
        // Charge for use
        assert!(economy.charge_actuation("client_user", 10.0).is_ok());
    }
}
